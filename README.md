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

## Prochaines étapes
- Implémenter le cache (localisation et météo).
- Ajouter un fichier de configuration (raccourci, unités, clés API optionnelles).
- Ajouter une icône météo dans la notification.

## Arborescence
- `src/` : code Rust.
- `tasks/` : tâches à faire.
- `docs/` : documentation détaillée.

## Lancement
```bash
cargo run
```

Une fois lancé, utilisez le raccourci global (Alt+A ou Alt+Q) ou appuyez sur Entrée
(ou tapez `w`) dans le terminal pour déclencher manuellement la notification. Tapez
`quit` pour quitter.

## Licence
MIT
