//! English strings, the canonical ones.
//!
//! `fr.rs` holds the same keys in the same order. A key in one file and not the
//! other fails `fr_and_en_have_identical_key_sets`.

pub(super) const STRINGS: &[(&str, &str)] = &[
    // Command line
    (
        "usage",
        "\
Oasis shows the current weather in a desktop notification.

Usage: oasis [OPTION]

With no option, Oasis runs as a daemon and listens for the global hotkey.

Options:
      --trigger  Show one notification, then exit
  -V, --version  Print the version, then exit
  -h, --help     Print this help, then exit

Environment:
  OASIS_CONFIG_PATH  Path of the TOML config file to read
",
    ),
    ("unknown_argument", "oasis: unknown argument '{arg}'"),
    // Config file
    ("config_warning", "Warning: {error}"),
    (
        "config_defaults",
        "Oasis continues with the default settings.",
    ),
    (
        "config_copied",
        "Oasis copied its config from {old} to {new}.",
    ),
    (
        "config_copy_failed",
        "Warning: cannot copy the config from {old} to {new}: {error}. Oasis keeps reading the old file.",
    ),
    // Daemon
    (
        "wayland_hint",
        "\
Warning: Wayland detected without an X11 server.
Global hotkeys do not work on pure Wayland.
Bind the one-shot mode in your compositor instead:

  oasis --trigger

Example for Hyprland (~/.config/hypr/hyprland.conf):
  bind = SUPER SHIFT, W, exec, oasis --trigger

Example for Sway (~/.config/sway/config):
  bindsym Mod4+Shift+w exec oasis --trigger

Starting the daemon anyway (the hotkeys may not work)...",
    ),
    ("daemon_ready", "Oasis Weather Notify - ready."),
    (
        "daemon_hotkeys",
        "Hotkeys: {primary} (AZERTY) or {secondary} (QWERTY).",
    ),
    (
        "daemon_manual_hint",
        "Tip: press Enter (or type 'w') to trigger a lookup by hand.",
    ),
    ("daemon_quit_hint", "Type 'quit' to exit."),
    ("active_hotkey", "Active hotkey: {hotkey}"),
    (
        "active_hotkeys",
        "Active hotkeys: {primary} and {secondary}",
    ),
    ("hotkey_pressed", "Hotkey pressed, fetching the weather..."),
    ("manual_trigger", "Manual trigger, fetching the weather..."),
    ("lookup_failed", "Error while fetching the weather: {error}"),
    (
        "invalid_hotkey",
        "Invalid hotkey '{hotkey}', using the default one: {error}",
    ),
    // Lookups
    (
        "ip_lookup_fallback",
        "Primary IP geolocation unavailable, trying the fallback: {error}",
    ),
    (
        "stale_location",
        "Could not refresh the location, using the expired cache: {error}",
    ),
    (
        "stale_weather",
        "Could not refresh the weather, using the expired cache: {error}",
    ),
    // Notification
    ("weather_unavailable", "Weather unavailable"),
    ("could_not_get_location", "Could not get your location"),
    ("could_not_get_weather", "Could not get the weather"),
    ("current_weather", "Current weather - {city}"),
    ("unknown_location", "Unknown location"),
    (
        "windows_toast_failed",
        "Windows notification failed, showing a message box instead: {error}",
    ),
    // WMO weather codes
    ("weather_clear_sky", "Clear sky"),
    ("weather_mainly_clear", "Mainly clear"),
    ("weather_overcast", "Overcast"),
    ("weather_fog", "Fog"),
    ("weather_drizzle", "Drizzle"),
    ("weather_freezing_drizzle", "Freezing drizzle"),
    ("weather_rain", "Rain"),
    ("weather_freezing_rain", "Freezing rain"),
    ("weather_snow", "Snow"),
    ("weather_snow_grains", "Snow grains"),
    ("weather_showers", "Showers"),
    ("weather_snow_showers", "Snow showers"),
    ("weather_thunderstorm", "Thunderstorm"),
    ("weather_thunderstorm_with_hail", "Thunderstorm with hail"),
    ("weather_unknown", "Unknown conditions"),
];
