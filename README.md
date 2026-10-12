<div align="center">

# Oasis

**Current weather in a native notification, one hotkey away, for Linux, Windows and macOS.**

</div>

[![License: GPL-3.0-or-later](https://img.shields.io/badge/License-GPL--3.0--or--later-blue.svg)](LICENSE)
[![Colony app](https://img.shields.io/badge/Colony-utility-purple)](https://github.com/Project-Colony/Colony)
[![Platforms](https://img.shields.io/badge/platforms-linux%20%7C%20windows%20%7C%20macOS-lightgrey)](#installation)

Checking the weather usually means opening a browser tab, a phone app or a
desktop widget that stays on screen all day. Oasis is a small background program
with no window: press `Super+Shift+W` and the current conditions for where you
are appear in a native notification, then go away.

> **Status:** early. On every change, CI builds Oasis and runs its tests on
> Linux, Windows and macOS (Apple Silicon); the Intel macOS binary is only
> cross-built by the release workflow. CI does not show a notification, so the
> per-OS notification code is wired but not exercised there, and the Windows
> and macOS notifications have not been checked by hand yet. On Windows the
> toast is sent under the PowerShell AppID, so it appears as coming from
> Windows PowerShell, with a MessageBox as the fallback. Oasis is a console
> program there: launched outside a terminal, it can open a console window that
> stays open while the daemon runs. Until the first release appears on the
> [releases page](../../releases), [building from source](#build-from-source)
> is the only way to install Oasis.

## Why Oasis

- **One key press, no window.** No tab to open, no widget taking screen space:
  the answer arrives where your other notifications do.
- **No account, no API key.** The location comes from your IP address and the
  weather from [Open-Meteo](https://open-meteo.com/), which needs no key.
- **Native on each OS.** It uses the notification service of the desktop you
  already run, not a window of its own.
- **Works on Wayland compositors too.** Where global hotkeys cannot work,
  `oasis --trigger` shows one notification from a compositor key binding.

## What it does

Each lookup finds your approximate location from your IP address, through
ipapi.co with ipwho.is as a fallback, asks Open-Meteo for the current
temperature and conditions there, and shows the place, the temperature and a
weather icon in a notification.

### Daemon mode

Run `oasis` with no option. It listens for the global hotkey (default
`Super+Shift+W`) and looks up the weather on each press. In the terminal it was
started from, press Enter or type `w` to trigger a lookup by hand, and type
`quit` to stop it. By default the location is kept in memory for a day and the
weather for five minutes, so repeated presses reuse them.

Global hotkeys need X11 or XWayland. Under a pure Wayland session Oasis prints a
warning and starts anyway, but the hotkey may never fire.

### One notification, for Wayland compositors

`oasis --trigger` shows one notification and exits. Bind it in your compositor
instead of using the daemon. On Hyprland (`~/.config/hypr/hyprland.conf`):

```
bind = SUPER SHIFT, W, exec, oasis --trigger
```

On Sway (`~/.config/sway/config`):

```
bindsym Mod4+Shift+w exec oasis --trigger
```

Use the full path of the binary if `oasis` is not on your `PATH`.

`oasis --version` prints the version and `oasis --help` lists the options. Any
other argument is an error (exit code 2).

### Configuration

Oasis reads an optional TOML file and falls back to defaults when it is missing.
If the file exists but cannot be read or parsed, Oasis prints a warning with the
path and the error, then runs with the defaults. `OASIS_CONFIG_PATH` points it
at another file. Otherwise the file lives in the shared Colony folder:

| OS | Path |
|---|---|
| Linux | `~/.config/Colony/Oasis/preferences/config.toml` |
| Windows | `%LOCALAPPDATA%\Colony\Oasis\preferences\config.toml` |
| macOS | `~/Library/Application Support/Colony/Oasis/preferences/config.toml` |

Earlier versions read `$XDG_CONFIG_HOME/oasis/config.toml`,
`~/.config/oasis/config.toml` or `%APPDATA%\Oasis\config.toml`. On the first
launch without a new file, Oasis copies the old one to the new path, prints both
paths and writes a `.migrated` marker next to it, so the copy happens once. The
old file is left in place. If the copy fails, Oasis keeps reading the old file.

Console messages and notifications are in English or French. `language` picks
one; `auto`, the default, follows the system language and gives English unless
it is French. `--help` and the messages printed while the config loads always
follow the system language, because they come before the file is read.

```toml
[general]
language = "auto" # or "en", "fr"

[hotkeys]
primary = "Super+Shift+W"
secondary = "Super+Shift+W"

[cache]
location_ttl_seconds = 86400
weather_ttl_seconds = 300

[weather]
temperature_unit = "celsius" # or "fahrenheit"
```

## Installation

### Via Colony (recommended)

Search for **Oasis** in [Colony](https://github.com/Project-Colony/Colony) and
install it. Updates arrive through the launcher.

### Direct binary download

Grab the asset for your platform from the
[latest release](../../releases/latest):

| Platform | Asset |
|---|---|
| Linux | `oasis-linux` |
| Windows | `oasis-windows.exe` |
| macOS (Apple Silicon) | `oasis-macos` |
| macOS (Intel) | `oasis-macos-x86` |

```bash
chmod +x oasis-linux && ./oasis-linux
```

Each asset comes with `.sig`, `.meta` and `.meta.sig` files. The
[Code signing policy](#code-signing-policy) shows how to check them.

### Build from source

```bash
git clone https://github.com/Project-Colony/Oasis
cd Oasis
cargo build --release
./target/release/oasis
```

Requires Rust 1.89 or newer. On Linux you also need the GTK 3 development files:
`libgtk-3-dev` on Debian and Ubuntu, `gtk3` on Arch Linux, `gtk3-devel` on
Fedora.

## Documentation

- [docs/docs.md](docs/docs.md): how a lookup works, the services used and a
  map of the source modules.
- [docs/installation.md](docs/installation.md): building and running on each
  OS, and how its notifications are shown.

## Code signing policy

Every release asset is signed by the Project Colony organisation in CI, never on
a developer machine. Next to each asset on the release page:

| File | What it is |
|---|---|
| `<asset>.sig` | an ed25519 signature over the asset, made with the organisation's release key |
| `<asset>.meta` | three lines binding the asset to its file name, its sha256 and the release version |
| `<asset>.meta.sig` | an ed25519 signature over the `.meta` |

The private key is an organisation secret, used only by the shared
[sign-and-publish workflow](https://github.com/Project-Colony/Project-Colony-Resources/blob/main/.github/workflows/sign-and-publish.yml)
in a job that builds nothing; the jobs that compile Oasis never see it.
Colony checks all three files before it installs or updates Oasis, and
refuses a release older than the one installed. To check a download yourself
with OpenSSL 3 (the same commands work for every asset):

```bash
cat > colony-release.pub <<'EOF'
-----BEGIN PUBLIC KEY-----
MCowBQYDK2VwAyEARNjg3Nn8H6/aBg1unwGjkUTcrdTxERNefVaqU8cFu0s=
-----END PUBLIC KEY-----
EOF
a=oasis-linux
openssl pkeyutl -verify -pubin -inkey colony-release.pub -rawin -in "$a" -sigfile "$a.sig"
openssl pkeyutl -verify -pubin -inkey colony-release.pub -rawin -in "$a.meta" -sigfile "$a.meta.sig"
cat "$a.meta"     # version=<tag>, asset=<file name>, sha256=<digest>
sha256sum "$a"    # the digest must equal the sha256 line
```

How releases are built, signed and published:
[design/releases.md](https://github.com/Project-Colony/Project-Colony-Resources/blob/main/design/releases.md#5-signing).

The Windows build is not Authenticode-signed yet. Signing it through SignPath
Foundation is planned, and this section will say so once it applies.

## Privacy

Oasis sends no telemetry, no analytics and no crash reports. It has no account,
no API key and no update check of its own. It makes network requests only when
you trigger a lookup.

| Data | Stored or sent | Where, and why |
|---|---|---|
| Settings | stored | `Colony/Oasis/preferences/config.toml` in this machine's config directory. Oasis only reads it, except for the one-time copy of a config from an earlier version, plus its `.migrated` marker. |
| Your IP address | sent | to [ipapi.co](https://ipapi.co/privacy/) (`https://ipapi.co/json/`) on each lookup the location cache cannot answer (see the cache row below), and to [ipwho.is](https://ipwhois.io/privacy) (`https://ipwho.is/`) only when ipapi.co fails. Oasis uses the approximate city, region, country, latitude and longitude they return. |
| Approximate latitude and longitude, temperature unit | sent | to [Open-Meteo](https://open-meteo.com/en/terms) (`https://api.open-meteo.com/v1/forecast`) on each lookup the weather cache cannot answer, which answers with the current temperature and a weather code. Like any HTTPS request, it also shows your IP address. |
| Location and weather answers | kept in memory | in daemon mode only, and only until Oasis exits: the location for 24 hours and the weather for 5 minutes by default (`[cache]` in the config). `oasis --trigger` keeps nothing, so every press makes the requests again. |
| Place name and weather | shown | in a notification, through the operating system's notification service, which may keep it in its notification history. |

Oasis connects to no server except the ones in this table, and only for the
lookup named there. Nothing else is written to disk.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).

Weather data by [Open-Meteo.com](https://open-meteo.com/), under
[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). Oasis uses the free
Open-Meteo API, which is for non-commercial use.
