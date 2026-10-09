# Docs

## Description

Oasis Weather Notify is a cross-platform desktop tool that shows the current
weather on demand, from a global hotkey. It derives your location from your IP
address (with a fallback service), then fetches the weather from Open-Meteo.

## Services used

### IP geolocation

- Primary: `https://ipapi.co/json/`
- Fallback: `https://ipwho.is/`

### Weather

- Open-Meteo (free, no API key)

## How it works

1. Oasis runs in the background.
2. You press the global hotkey.
3. Oasis looks up your location from your IP address.
4. Oasis asks Open-Meteo for the weather at that latitude and longitude.
5. Oasis shows a native notification.

## Technical choices

- Global hotkeys: `global-hotkey`.
- Notifications per OS: `notify-rust` (Linux), `tauri-winrt-notification` with
  a MessageBox fallback (Windows), `mac-notification-sys` (macOS).
- Configuration from an optional TOML file, with defaults.
- The location is cached for a long time (one day by default) and the weather
  for a short time (five minutes by default).

### Module map

All the code is one binary crate, one flat file per module under `src/`:

| Module | What lives there |
|---|---|
| `main.rs` | Command-line parsing and dispatch to the daemon or one lookup |
| `daemon.rs` | Daemon mode: hotkey registration, the event loop, stdin commands, the Wayland hint |
| `lookup.rs` | One lookup: location, weather, notification text, then the notification (`--trigger` mode too) |
| `location.rs` | IP geolocation, with the fallback service |
| `weather.rs` | The Open-Meteo request and the weather code descriptions and icons |
| `cache.rs` | The in-memory location and weather caches |
| `config.rs` | The TOML config file and its defaults |
| `hotkey.rs` | Parsing the hotkey strings from the config |
| `notification.rs` | The native notification on each OS |

## Security and privacy

- Oasis writes nothing to disk: both caches live in memory and are gone when
  it exits.
- It only makes network requests when you trigger a lookup, and the caches
  avoid repeating them.
- There is no way yet to turn IP geolocation off or to set a fixed location.

## Installation

See `docs/installation.md` for per-OS instructions and the build and release
scripts.
