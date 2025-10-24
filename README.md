# tmux-knight

Automatically switch between light and dark theme on tmux.

I wrote a similar program, [Knight], for automatically switching the system
theme on Linux based on the time of day. This is an extension to that project.

## Installation

Install directly from Git using cargo:

```bash
cargo install --git https://github.com/nicdgonzalez/tmux-knight
```

## Requirements

**Tested with**:

- cargo 1.87.0
- tmux 3.5a

Other recent versions may also work.

## Usage

This program expects you to have two themes available: one for light, one for
dark. The themes themselves can be anywhere, but they need to be symlinked into
`$XDG_CONFIG_HOME/tmux/themes`.

```bash
# Light theme
ln --symbolic </path/to/light_theme.conf> "${XDG_CONFIG_HOME:-$HOME/.config}/tmux/light.conf"

# Dark theme
ln --symbolic </path/to/dark_theme.conf> "${XDG_CONFIG_HOME:-$HOME/.config}/tmux/dark.conf"
```

Then, run the program:

```bash
tmux-knight
```

Change the theme on your system and watch tmux automatically match it.

This program is designed to run forever; you aren't meant to run it directly.
To quit, press <kbd>Ctrl</kbd>+<kbd>C</kbd>.

### Intended usage

To automatically run `tmux-knight` when the system boots, run:

> [!TIP]\
> Piping directly to bash can be risky because it prevents you from reading the
> code that will run on your system. Always inspect scripts before executing
> them to ensure they are safe.
>
> You can inspect the script used below [here](./scripts/systemd.sh).

```bash
curl -SsL https://raw.githubusercontent.com/nicdgonzalez/tmux-knight/refs/heads/main/scripts/systemd.sh | bash
```

Or create a new file at `$HOME/.config/systemd/user/tmux-knight.service` with
the following contents:

```ini
[Unit]
Description=Automatically switch between light and dark theme on tmux

[Service]
ExecStart=/usr/bin/env tmux-knight start

[Install]
WantedBy=default.target
```

Then run the following commands:

```bash
# Make systemd aware of our changes
systemctl --user daemon-reload

# Start the service
systemctl --user start tmux-knight.service

# Persist after reboots
systemctl --user enable tmux-knight.service

# Check if the service is running
systemctl --user status tmux-knight.service
```

### Stopping the program

To stop the program until the next reboot:

```bash
systemctl --user stop tmux-knight.service
```

To stop the program indefinitely:

```bash
systemctl --user disable tmux-knight.service
systemctl --user stop tmux-knight.service
```

To uninstall:

```bash
cargo uninstall tmux-knight
```

## Roadmap

- [ ] Use `dbus-send` to better query the current theme.

[knight]: https://github.com/nicdgonzalez/knight
