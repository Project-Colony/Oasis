# Installation

This guide covers building and running Oasis on each OS. To install a prebuilt
binary instead, through Colony or from the release page, see the Installation
section of the README.

## Common requirements

- Rust and Cargo, version 1.89 or newer, to build from source.
- Outbound network access for the IP and weather lookups.

## Linux

### Build from source

1. Install the GTK 3 development files, which the event loop links against:
   `libgtk-3-dev` on Debian and Ubuntu, `gtk3` on Arch Linux, `gtk3-devel` on
   Fedora.
2. Clone the repository.
3. Build in release mode:
   ```bash
   cargo build --release
   ```
4. Run the application:
   ```bash
   ./target/release/oasis
   ```

### Notifications

- Notifications go through `notify-rust`, which talks to your desktop's
  notification daemon over D-Bus.
- Make sure a notification daemon is running (most desktops ship one).

## macOS

### Build from source

1. Clone the repository.
2. Build in release mode:
   ```bash
   cargo build --release
   ```
3. Run the application:
   ```bash
   ./target/release/oasis
   ```

### Notifications

- Notifications use `mac-notification-sys`.
- macOS may ask for permission to show notifications.

## Windows

### Build from source (PowerShell)

1. Clone the repository.
2. Build in release mode:
   ```powershell
   cargo build --release
   ```
3. Run the application:
   ```powershell
   .\target\release\oasis.exe
   ```

### Notifications

- Notifications use `tauri-winrt-notification`, with a MessageBox fallback.
- Oasis sends its toasts under the Windows PowerShell AppID, so they appear as
  coming from Windows PowerShell. Check that notifications are allowed for
  Windows PowerShell in Settings, System, Notifications.
- Oasis is a console program: launched outside a terminal, it can open a
  console window, which stays open while the daemon runs.

## Configuration

An optional TOML file sets the language (`auto`, `en` or `fr`), the hotkeys, the
cache lifetimes and the temperature unit:

- Linux: `~/.config/Colony/Oasis/preferences/config.toml`
- Windows: `%LOCALAPPDATA%\Colony\Oasis\preferences\config.toml`
- macOS: `~/Library/Application Support/Colony/Oasis/preferences/config.toml`

A config from an earlier version is copied there once, on the first launch. See
the README for the details and an example.
