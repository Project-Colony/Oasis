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
        .context("the Open-Meteo weather request failed")?
        .error_for_status()
        .context("Open-Meteo returned an HTTP error")?
        .json::<OpenMeteoResponse>()
        .await
        .context("cannot parse the Open-Meteo response")?;

    response
        .current
        .context("the Open-Meteo response has no current weather")
}

/// The i18n key of the WMO weather code's description.
pub fn weather_description(code: i32) -> &'static str {
    match code {
        0 => "weather_clear_sky",
        1 | 2 => "weather_mainly_clear",
        3 => "weather_overcast",
        45 | 48 => "weather_fog",
        51 | 53 | 55 => "weather_drizzle",
        56 | 57 => "weather_freezing_drizzle",
        61 | 63 | 65 => "weather_rain",
        66 | 67 => "weather_freezing_rain",
        71 | 73 | 75 => "weather_snow",
        77 => "weather_snow_grains",
        80..=82 => "weather_showers",
        85 | 86 => "weather_snow_showers",
        95 => "weather_thunderstorm",
        96 | 99 => "weather_thunderstorm_with_hail",
        _ => "weather_unknown",
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
    use crate::i18n::t_in;
    use colony_ui::i18n::Locale;

    fn describe(locale: Locale, code: i32) -> &'static str {
        t_in(locale, weather_description(code))
    }

    #[test]
    fn test_weather_description_known_codes() {
        assert_eq!(weather_description(0), "weather_clear_sky");
        assert_eq!(weather_description(1), "weather_mainly_clear");
        assert_eq!(weather_description(2), "weather_mainly_clear");
        assert_eq!(weather_description(3), "weather_overcast");
        assert_eq!(weather_description(45), "weather_fog");
        assert_eq!(weather_description(61), "weather_rain");
        assert_eq!(weather_description(71), "weather_snow");
        assert_eq!(weather_description(95), "weather_thunderstorm");
        assert_eq!(weather_description(99), "weather_thunderstorm_with_hail");

        assert_eq!(describe(Locale::En, 0), "Clear sky");
        assert_eq!(describe(Locale::En, 3), "Overcast");
        assert_eq!(describe(Locale::En, 99), "Thunderstorm with hail");
        assert_eq!(describe(Locale::Fr, 3), "Couvert");
        assert_eq!(describe(Locale::Fr, 61), "Pluie");
        assert_eq!(describe(Locale::Fr, 71), "Neige");
    }

    #[test]
    fn test_weather_description_unknown_code() {
        assert_eq!(weather_description(999), "weather_unknown");
        assert_eq!(weather_description(-1), "weather_unknown");
        assert_eq!(describe(Locale::En, 999), "Unknown conditions");
        assert_eq!(describe(Locale::Fr, 999), "Conditions inconnues");
    }

    /// A description key missing from a table would render as the key itself.
    #[test]
    fn every_weather_code_has_a_string_in_both_locales() {
        for code in -1..=100 {
            let key = weather_description(code);
            for locale in [Locale::En, Locale::Fr] {
                assert_ne!(t_in(locale, key), key, "{key} in {locale:?}");
            }
        }
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
