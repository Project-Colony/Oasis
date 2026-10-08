# Oasis Weather Notify

Oasis is a small Rust app that shows the current weather in a native desktop
notification when you press a global hotkey. It finds your approximate location
from your IP address, then fetches the current conditions from Open-Meteo.

## Status

Early development (0.1.0). There are no prebuilt releases yet: build it from
source. Console messages are currently in French.

## How it works

1. A global hotkey (default `Super+Shift+W`) or the `--trigger` flag starts a lookup.
2. The location comes from IP geolocation: ipapi.co, with ipwho.is as a fallback.
3. The weather comes from Open-Meteo, which needs no API key.
4. A native notification shows the place, the temperature and a weather icon.

Notifications use `notify-rust` on Linux, `winrt-notification` with a MessageBox
fallback on Windows, and `mac-notification-sys` on macOS.

## Build from source

You need a recent stable Rust toolchain (edition 2024) and network access.

```bash
cargo build --release
./target/release/oasis-weather-notify
```

`scripts/build.sh`, `scripts/release.sh` and `scripts/release.ps1` wrap the
release build and packaging. See `docs/installation.md` for per-OS notes.

## Usage

- Daemon mode (default): listens for the global hotkey. Press Enter or type `w`
  in the terminal to trigger a notification by hand, and type `quit` to exit.
- One-shot mode: `oasis-weather-notify --trigger` shows one notification and exits.

Global hotkeys need X11 or XWayland. On pure Wayland, bind the one-shot mode in
your compositor instead, for example on Hyprland:

```
bind = SUPER SHIFT, W, exec, oasis-weather-notify --trigger
```

## Configuration

Oasis reads an optional TOML file and falls back to defaults when it is missing.
It looks for the file in this order:

- `OASIS_CONFIG_PATH`
- `$XDG_CONFIG_HOME/oasis/config.toml`
- `$HOME/.config/oasis/config.toml`
- `%APPDATA%\Oasis\config.toml` (Windows)

```toml
[hotkeys]
primary = "Super+Shift+W"
secondary = "Super+Shift+W"

[cache]
location_ttl_seconds = 86400
weather_ttl_seconds = 300

[weather]
temperature_unit = "celsius" # or "fahrenheit"
```

## License

Oasis is free software, released under the GNU General Public License v3.0 or
later. See [LICENSE](LICENSE).
