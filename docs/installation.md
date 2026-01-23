# Installation

Ce guide décrit comment installer et utiliser Oasis Weather Notify sur chaque OS.

## Prérequis communs
- Rust + Cargo installés si vous compilez depuis les sources.
- Un accès réseau sortant pour les appels IP + météo.

## Linux
### Installation depuis les sources
1. Cloner le dépôt.
2. Compiler en release :
   ```bash
   ./scripts/build.sh
   ```
3. Lancer l'application :
   ```bash
   ./target/release/oasis-weather-notify
   ```

### Notifications
- Les notifications reposent sur `notify-rust` (souvent via `libnotify`).
- Sur certaines distributions, installez `libnotify` si nécessaire.

## macOS
### Installation depuis les sources
1. Cloner le dépôt.
2. Compiler en release :
   ```bash
   ./scripts/build.sh
   ```
3. Lancer l'application :
   ```bash
   ./target/release/oasis-weather-notify
   ```

### Notifications
- Les notifications utilisent `mac-notification-sys`.
- macOS peut demander l'autorisation d'afficher des notifications.

## Windows
### Installation depuis les sources (PowerShell)
1. Cloner le dépôt.
2. Compiler en release :
   ```powershell
   .\scripts\release.ps1
   ```
3. Lancer l'application :
   ```powershell
   .\target\release\oasis-weather-notify.exe
   ```

### Notifications
- Les notifications utilisent `winrt-notification` avec fallback MessageBox.
- Vérifiez que l'application est autorisée à afficher des notifications.

## Configuration
Un fichier TOML optionnel permet de personnaliser les raccourcis et le cache. Voir le README pour les détails et l'exemple de configuration.
