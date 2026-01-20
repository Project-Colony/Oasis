# Oasis Weather Notify

Une application Rust + Cargo qui affiche la météo actuelle via une notification native (Windows/Linux/macOS) lorsqu'un raccourci clavier global est déclenché. La localisation est dérivée de l'adresse IP, puis la météo est récupérée via une API météo.

## Objectif
- Raccourci clavier global → déclenche une notification système.
- Localisation via IP (ville + coordonnées).
- Météo actuelle basée sur latitude/longitude.
- Multiplateforme (Windows, Linux, macOS).

## Flux envisagé
1. Détection du raccourci clavier.
2. Récupération de la localisation via IP.
3. Appel d'une API météo (température + description).
4. Affichage d'une notification native avec lieu + météo.

## Prochaines étapes
- Choisir l'API météo (Open-Meteo, OpenWeather, etc.).
- Choisir la librairie de raccourcis globaux.
- Choisir la librairie de notifications par OS.
- Implémenter le cache (localisation et météo).

## Arborescence
- `src/` : code Rust.
- `tasks/` : tâches à faire.
- `docs/` : documentation détaillée.

## Lancement (placeholder)
```bash
cargo run
```

## Licence
MIT
