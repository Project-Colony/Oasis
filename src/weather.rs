use anyhow::{Context, Result};
use serde::Deserialize;

use crate::config::TemperatureUnit;

const OPEN_METEO_BASE_URL: &str = "https://api.open-meteo.com/v1/forecast";

#[derive(Debug, Deserialize)]
struct OpenMeteoResponse {
    current: Option<OpenMeteoCurrent>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OpenMeteoCurrent {
    pub temperature_2m: f64,
    pub weather_code: i32,
}

pub async fn fetch_weather(
    client: &reqwest::Client,
    latitude: f64,
    longitude: f64,
    temperature_unit: TemperatureUnit,
) -> Result<OpenMeteoCurrent> {
    let url = format!(
        "{OPEN_METEO_BASE_URL}?latitude={latitude}&longitude={longitude}&current=temperature_2m,weather_code&temperature_unit={}",
        temperature_unit.as_query_param()
    );

    let response = client
        .get(url)
        .send()
        .await
        .context("Échec requête météo Open-Meteo")?
        .error_for_status()
        .context("Erreur HTTP météo Open-Meteo")?
        .json::<OpenMeteoResponse>()
        .await
        .context("Échec parsing météo Open-Meteo")?;

    response.current.context("Réponse météo incomplète")
}

pub fn weather_description(code: i32) -> &'static str {
    match code {
        0 => "Ciel dégagé",
        1 | 2 => "Plutôt clair",
        3 => "Couvert",
        45 | 48 => "Brouillard",
        51 | 53 | 55 => "Bruine",
        56 | 57 => "Bruine verglaçante",
        61 | 63 | 65 => "Pluie",
        66 | 67 => "Pluie verglaçante",
        71 | 73 | 75 => "Neige",
        77 => "Grains de neige",
        80..=82 => "Averses",
        85 | 86 => "Averses de neige",
        95 => "Orage",
        96 | 99 => "Orage avec grêle",
        _ => "Conditions inconnues",
    }
}

pub fn weather_icon(code: i32) -> &'static str {
    match code {
        0 => "☀️",
        1 | 2 => "🌤️",
        3 => "☁️",
        45 | 48 => "🌫️",
        51 | 53 | 55 => "🌦️",
        56 | 57 => "🧊",
        61 | 63 | 65 => "🌧️",
        66 | 67 => "🌧️",
        71 | 73 | 75 | 77 => "❄️",
        80..=82 => "🌧️",
        85 | 86 => "🌨️",
        95 | 96 | 99 => "⛈️",
        _ => "🌡️",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_description_known_codes() {
        assert_eq!(weather_description(0), "Ciel dégagé");
        assert_eq!(weather_description(1), "Plutôt clair");
        assert_eq!(weather_description(2), "Plutôt clair");
        assert_eq!(weather_description(3), "Couvert");
        assert_eq!(weather_description(45), "Brouillard");
        assert_eq!(weather_description(61), "Pluie");
        assert_eq!(weather_description(71), "Neige");
        assert_eq!(weather_description(95), "Orage");
        assert_eq!(weather_description(99), "Orage avec grêle");
    }

    #[test]
    fn test_weather_description_unknown_code() {
        assert_eq!(weather_description(999), "Conditions inconnues");
        assert_eq!(weather_description(-1), "Conditions inconnues");
    }

    #[test]
    fn test_weather_icon_known_codes() {
        assert_eq!(weather_icon(0), "☀️");
        assert_eq!(weather_icon(3), "☁️");
        assert_eq!(weather_icon(71), "❄️");
        assert_eq!(weather_icon(95), "⛈️");
    }

    #[test]
    fn test_weather_icon_unknown_code() {
        assert_eq!(weather_icon(999), "🌡️");
    }
}
