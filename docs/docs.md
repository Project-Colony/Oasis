# Docs

## Description
Oasis Weather Notify est un outil desktop multiplateforme qui affiche la météo actuelle sur demande via un raccourci clavier global. La localisation est dérivée de l'adresse IP (avec fallback), puis la météo est récupérée via Open-Meteo.

## API utilisées
### Géolocalisation IP
- Primaire : `https://ipapi.co/json/`
- Fallback : `https://ipwho.is/`

### Météo
- Open-Meteo (gratuit, sans clé)

## Fonctionnement détaillé
1. Le service s'exécute en arrière-plan.
2. L'utilisateur déclenche un raccourci clavier.
3. L'app obtient la localisation via IP.
4. L'app appelle l'API météo avec lat/long.
5. L'app affiche une notification système.

## Choix techniques validés
- Hotkeys globales : `global-hotkey`.
- Notifications par OS : `notify-rust` (Linux), `winrt-notification` + fallback MessageBox (Windows), `mac-notification-sys` (macOS).

## Choix techniques finalisés
- Configuration via fichier TOML (avec valeurs par défaut).
- Cache de la localisation (TTL long) + cache météo (TTL court).

## Sécurité & vie privée
- Ne stocker aucune donnée personnelle.
- Minimiser les requêtes réseau.
- Permettre l'opt-out de la géolocalisation.

## Installation
Voir `docs/installation.md` pour les instructions par OS et les scripts de build/release.
