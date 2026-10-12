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

- Open-Meteo: `https://api.open-meteo.com/v1/forecast` (free for non-commercial
  use, no API key). Weather data by [Open-Meteo.com](https://open-meteo.com/),
  under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).

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
- In daemon mode, the location is cached for a long time (one day by default)
  and the weather for a short time (five minutes by default). `--trigger` runs
  one lookup per process, so it caches nothing.

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
| `config.rs` | The TOML config file, its location, its defaults and the one-time move from the old location |
| `hotkey.rs` | Parsing the hotkey strings from the config |
| `notification.rs` | The native notification on each OS |

## Security and privacy

- Oasis makes network requests only when you trigger a lookup. Each request
  goes to the services listed above, and each shows them your IP address.
- In daemon mode, both caches live in memory and are gone when Oasis exits.
  Within their lifetime a lookup reuses them instead of making the requests
  again, and a failed request falls back to the expired entry when there is
  one. `--trigger` caches nothing, so every press makes the requests.
- The only thing Oasis writes to disk is the one-time copy of a config file
  from an earlier version, plus its `.migrated` marker.
- There is no way yet to turn IP geolocation off or to set a fixed location.

## Installation

See `docs/installation.md` for building and running Oasis on each OS. Releases
are built, signed and published by `.github/workflows/release.yml` when a
release-please pull request is merged; the README's Code signing policy shows
how to check a download.
