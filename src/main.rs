#![warn(
    clippy::correctness,
    clippy::suspicious,
    clippy::complexity,
    clippy::style,
    clippy::pedantic
)]

use std::io::Write as _;
use std::path::Path;
use std::process::{Command, ExitCode};
use std::time::{Duration, Instant};
use std::{fs, io, thread};

use anyhow::{Context as _, bail};
use colored::Colorize as _;
use tracing::{error, warn};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

#[derive(Debug, Clone, Copy)]
enum Theme {
    Light,
    Dark,
}

fn main() -> ExitCode {
    try_main().unwrap_or_else(|err| {
        let mut stderr = io::stderr().lock();
        _ = writeln!(stderr, "{}", "tmux-knight failed".bold().red());

        for cause in err.chain() {
            _ = writeln!(stderr, "  {}: {}", "Cause:".bold(), cause);
        }

        ExitCode::FAILURE
    })
}

fn try_main() -> anyhow::Result<ExitCode> {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=trace", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let tmux_themes = {
        let mut path = dirs::config_local_dir().context("failed to get configuration directory")?;
        path.extend(["tmux", "themes"]);
        path
    };
    let light_theme = tmux_themes.join("light.conf");
    let dark_theme = tmux_themes.join("dark.conf");
    let current_theme = tmux_themes.join("current.conf");
    let mut last_warning = Option::<Instant>::None;
    let interval = Duration::from_millis(1500);

    loop {
        let next_tick = Instant::now() + interval;
        let system_theme = get_current_theme()
            .inspect_err(|err| error!("failed to get current theme: {err}"))
            .unwrap_or(Theme::Light);

        match system_theme {
            Theme::Light => {
                if is_target_theme(&current_theme, &light_theme) {
                    maybe_sleep(next_tick);
                    continue;
                }

                if let Err(err) = fs::remove_file(&current_theme) {
                    error!("failed to unlink previous theme: {err}");
                    maybe_sleep(next_tick);
                    continue;
                }

                if let Err(err) = symlink::symlink_file(&light_theme, &current_theme)
                    && last_warning
                        .is_none_or(|w| Instant::now() >= (w + Duration::from_secs(60 * 10)))
                {
                    warn!("failed to symlink to light theme: {err}");
                    last_warning = Some(Instant::now());
                }

                match Command::new("tmux")
                    .args(["source-file", &current_theme.to_string_lossy()])
                    .status()
                {
                    Ok(status) => match status.code() {
                        Some(0) => {}
                        Some(code) => warn!("failed with exit code: {code}"),
                        None => warn!("process terminated due to signal"),
                    },
                    Err(err) => error!("failed to execute tmux: {err}"),
                }
            }
            Theme::Dark => {
                // Check if the current theme is already dark mode.
                if let (Ok(linked), Ok(dark)) =
                    (current_theme.canonicalize(), dark_theme.canonicalize())
                    && linked == dark
                {
                    maybe_sleep(next_tick);
                    continue;
                }

                if let Err(err) = fs::remove_file(&current_theme) {
                    error!("failed to unlink previous theme: {err}");
                    maybe_sleep(next_tick);
                    continue;
                }

                if let Err(err) = symlink::symlink_file(&dark_theme, &current_theme)
                    && last_warning
                        .is_none_or(|w| Instant::now() >= (w + Duration::from_secs(60 * 10)))
                {
                    warn!("failed to symlink to new theme: {err}");
                    last_warning = Some(Instant::now());
                }

                match Command::new("tmux")
                    .args(["source-file", &current_theme.to_string_lossy()])
                    .status()
                {
                    Ok(status) => match status.code() {
                        Some(0) => {}
                        Some(code) => warn!("failed with exit code: {code}"),
                        None => warn!("process terminated due to signal"),
                    },
                    Err(err) => error!("failed to execute tmux: {err}"),
                }
            }
        }

        maybe_sleep(next_tick);
    }
}

fn maybe_sleep(next_tick: Instant) {
    let now = Instant::now();
    if now < next_tick {
        thread::sleep(next_tick - now);
    }
}

fn is_target_theme(current: &Path, target: &Path) -> bool {
    matches!(
        (current.canonicalize(), target.canonicalize()),
        (Ok(linked), Ok(target)) if linked == target
    )
}

fn get_current_theme() -> anyhow::Result<Theme> {
    let output = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output()
        .context("failed to execute gsettings")?;

    let stdout =
        String::from_utf8(output.stdout).context("expected gsettings output to be valid UTF-8")?;
    let value = stdout.trim().trim_matches('\'').to_lowercase();

    Ok(match value.as_ref() {
        "default" => Theme::Light,
        "prefer-dark" => Theme::Dark,
        other => bail!("unknown gsettings color-scheme value: {other}"),
    })
}
