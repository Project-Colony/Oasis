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

## Security and privacy

- Oasis writes nothing to disk: both caches live in memory and are gone when
  it exits.
- It only makes network requests when you trigger a lookup, and the caches
  avoid repeating them.
- There is no way yet to turn IP geolocation off or to set a fixed location.

## Installation

See `docs/installation.md` for per-OS instructions and the build and release
scripts.
