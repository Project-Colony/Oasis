# Docs

## Description
Oasis Weather Notify est un outil desktop multiplateforme qui affiche la météo actuelle sur demande via un raccourci clavier global. La localisation est dérivée de l'adresse IP, puis la météo est récupérée via une API en temps réel.

## API recommandées
### Géolocalisation IP
- `https://ipapi.co/json/`
- `https://ipinfo.io/json`
- `https://ipwho.is/`

### Météo
- Open-Meteo (gratuit, sans clé)
- OpenWeather (clé requise)
- WeatherAPI (clé requise)

## Fonctionnement détaillé
1. Le service s'exécute en arrière-plan.
2. L'utilisateur déclenche un raccourci clavier.
3. L'app obtient la localisation via IP.
4. L'app appelle l'API météo avec lat/long.
5. L'app affiche une notification système.

## Choix techniques à valider
- Librairie de hotkey globale.
- Librairie de notification par OS.
- Modèle de configuration (fichier TOML).
- Stratégie de cache et fréquence de mise à jour.

## Sécurité & vie privée
- Ne stocker aucune donnée personnelle.
- Minimiser les requêtes réseau.
- Permettre l'opt-out de la géolocalisation.
