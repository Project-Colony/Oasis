# Oasis Weather Notify

Une application Rust + Cargo qui affiche la météo actuelle via une notification native (Windows/Linux/macOS) lorsqu'un raccourci clavier global est déclenché. La localisation est dérivée de l'adresse IP (avec fallback), puis la météo est récupérée via l'API Open-Meteo.

## Objectif
- Raccourci clavier global → déclenche une notification système.
- Localisation via IP (ville + coordonnées).
- Météo actuelle basée sur latitude/longitude.
- Multiplateforme (Windows, Linux, macOS).

## Flux actuel
1. Détection du raccourci clavier.
2. Récupération de la localisation via IP.
3. Appel d'une API météo (température + description).
4. Affichage d'une notification native avec lieu + météo.

## Composants utilisés
- Géolocalisation IP : ipapi.co (fallback ipwho.is).
- Météo : Open-Meteo (sans clé).
- Hotkeys globales : `global-hotkey`.
- Notifications : `notify-rust` (Linux), `winrt-notification` + fallback MessageBox (Windows), `mac-notification-sys` (macOS).

## Notifications
Les notifications affichent désormais une icône météo (emoji) basée sur le code Open-Meteo.

## Arborescence
- `src/` : code Rust.
- `tasks/` : tâches à faire.
- `docs/` : documentation détaillée.

## Lancement
```bash
cargo run
```

## Configuration
Oasis charge un fichier TOML optionnel (valeurs par défaut si absent). Le chemin est
déterminé dans l'ordre suivant :
- `OASIS_CONFIG_PATH`
- `${XDG_CONFIG_HOME}/oasis/config.toml`
- `${HOME}/.config/oasis/config.toml`
- `%APPDATA%\\Oasis\\config.toml` (Windows)

Exemple de configuration :
```toml
[hotkeys]
primary = "Alt+A"
secondary = "Alt+Q"

[cache]
location_ttl_seconds = 86400
weather_ttl_seconds = 600

[weather]
temperature_unit = "celsius" # ou "fahrenheit"
```

Une fois lancé, utilisez le raccourci global (Alt+A ou Alt+Q) ou appuyez sur Entrée
(ou tapez `w`) dans le terminal pour déclencher manuellement la notification. Tapez
`quit` pour quitter.

## Licence
MIT
