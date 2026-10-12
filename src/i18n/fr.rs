//! French strings, translated from `en.rs` and kept in the same order.

pub(super) const STRINGS: &[(&str, &str)] = &[
    // Command line
    (
        "usage",
        "\
Oasis affiche la météo actuelle dans une notification de bureau.

Utilisation : oasis [OPTION]

Sans option, Oasis tourne en tâche de fond et attend le raccourci global.

Options :
      --trigger  Affiche une notification, puis quitte
  -V, --version  Affiche la version, puis quitte
  -h, --help     Affiche cette aide, puis quitte

Environnement :
  OASIS_CONFIG_PATH  Chemin du fichier de configuration TOML à lire
",
    ),
    ("unknown_argument", "oasis : argument inconnu « {arg} »"),
    // Config file
    ("config_warning", "Attention : {error}"),
    (
        "config_defaults",
        "Oasis continue avec les réglages par défaut.",
    ),
    (
        "config_copied",
        "Oasis a copié sa configuration de {old} vers {new}.",
    ),
    (
        "config_copy_failed",
        "Attention : impossible de copier la configuration de {old} vers {new} : {error}. Oasis continue de lire l'ancien fichier.",
    ),
    (
        "config_unknown_language",
        "Attention : [general] language doit valoir « auto », « en » ou « fr ». Oasis suit la langue du système.",
    ),
    // Daemon
    (
        "wayland_hint",
        "\
Attention : Wayland détecté sans serveur X11.
Les raccourcis globaux ne fonctionnent pas sous Wayland pur.
Associez plutôt le mode ponctuel à un raccourci de votre compositeur :

  oasis --trigger

Exemple pour Hyprland (~/.config/hypr/hyprland.conf) :
  bind = SUPER SHIFT, W, exec, oasis --trigger

Exemple pour Sway (~/.config/sway/config) :
  bindsym Mod4+Shift+w exec oasis --trigger

Oasis démarre quand même en tâche de fond (les raccourcis risquent de ne pas fonctionner)...",
    ),
    ("daemon_ready", "Oasis Weather Notify - prêt."),
    (
        "daemon_hotkeys",
        "Raccourcis : {primary} (AZERTY) ou {secondary} (QWERTY).",
    ),
    (
        "daemon_manual_hint",
        "Astuce : appuyez sur Entrée (ou tapez « w ») pour lancer une recherche à la main.",
    ),
    ("daemon_quit_hint", "Tapez « quit » pour quitter."),
    ("active_hotkey", "Raccourci actif : {hotkey}"),
    (
        "active_hotkeys",
        "Raccourcis actifs : {primary} et {secondary}",
    ),
    (
        "hotkey_pressed",
        "Raccourci détecté, récupération de la météo...",
    ),
    (
        "manual_trigger",
        "Déclenchement manuel, récupération de la météo...",
    ),
    (
        "lookup_failed",
        "Erreur lors de la récupération de la météo : {error}",
    ),
    (
        "invalid_hotkey",
        "Raccourci « {hotkey} » invalide, celui par défaut est utilisé : {error}",
    ),
    // Lookups
    (
        "ip_lookup_fallback",
        "Géolocalisation IP principale indisponible, essai du service de secours : {error}",
    ),
    (
        "stale_location",
        "Impossible d'actualiser la position, le cache expiré est utilisé : {error}",
    ),
    (
        "stale_weather",
        "Impossible d'actualiser la météo, le cache expiré est utilisé : {error}",
    ),
    // Notification
    ("weather_unavailable", "Météo indisponible"),
    (
        "could_not_get_location",
        "Impossible de déterminer votre position",
    ),
    ("could_not_get_weather", "Impossible de récupérer la météo"),
    ("current_weather", "Météo actuelle - {city}"),
    ("unknown_location", "Lieu inconnu"),
    (
        "windows_toast_failed",
        "Échec de la notification Windows, affichage d'une boîte de dialogue à la place : {error}",
    ),
    // WMO weather codes
    ("weather_clear_sky", "Ciel dégagé"),
    ("weather_mainly_clear", "Plutôt dégagé"),
    ("weather_overcast", "Couvert"),
    ("weather_fog", "Brouillard"),
    ("weather_drizzle", "Bruine"),
    ("weather_freezing_drizzle", "Bruine verglaçante"),
    ("weather_rain", "Pluie"),
    ("weather_freezing_rain", "Pluie verglaçante"),
    ("weather_snow", "Neige"),
    ("weather_snow_grains", "Neige en grains"),
    ("weather_showers", "Averses"),
    ("weather_snow_showers", "Averses de neige"),
    ("weather_thunderstorm", "Orage"),
    ("weather_thunderstorm_with_hail", "Orage avec grêle"),
    ("weather_unknown", "Conditions inconnues"),
];
