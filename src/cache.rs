use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use crate::config::TemperatureUnit;
use crate::location::{self, Location};
use crate::weather::{self, OpenMeteoCurrent};

const COORD_EPSILON: f64 = 0.0001;

#[derive(Debug, Clone)]
pub struct Cached<T> {
    pub value: T,
    pub fetched_at: Instant,
}

#[derive(Debug, Clone)]
pub struct WeatherCache {
    pub value: OpenMeteoCurrent,
    pub fetched_at: Instant,
    pub latitude: f64,
    pub longitude: f64,
}

impl WeatherCache {
    pub fn matches_location(&self, lat: f64, lon: f64) -> bool {
        (self.latitude - lat).abs() < COORD_EPSILON && (self.longitude - lon).abs() < COORD_EPSILON
    }
}

#[derive(Debug, Default)]
pub struct AppCache {
    pub location: Option<Cached<Location>>,
    pub weather: Option<WeatherCache>,
}

pub async fn get_location_with_cache(
    client: &reqwest::Client,
    cache: &Arc<Mutex<AppCache>>,
    ttl: Duration,
) -> Result<Location> {
    let cached = {
        let cache = cache.lock().await;
        cache.location.clone()
    };

    if let Some(entry) = cached.as_ref()
        && entry.fetched_at.elapsed() <= ttl
    {
        return Ok(entry.value.clone());
    }

    match location::fetch_ip_location(client).await {
        Ok(location) => {
            let mut cache = cache.lock().await;
            cache.location = Some(Cached {
                value: location.clone(),
                fetched_at: Instant::now(),
            });
            Ok(location)
        }
        Err(error) => {
            if let Some(entry) = cached {
                eprintln!("Échec récupération localisation, cache expiré utilisé: {error:#}");
                Ok(entry.value)
            } else {
                Err(error)
            }
        }
    }
}

pub async fn get_weather_with_cache(
    client: &reqwest::Client,
    cache: &Arc<Mutex<AppCache>>,
    latitude: f64,
    longitude: f64,
    temperature_unit: TemperatureUnit,
    ttl: Duration,
) -> Result<OpenMeteoCurrent> {
    let cached = {
        let cache = cache.lock().await;
        cache.weather.clone()
    };

    if let Some(entry) = cached.as_ref()
        && entry.matches_location(latitude, longitude)
        && entry.fetched_at.elapsed() <= ttl
    {
        return Ok(entry.value.clone());
    }

    match weather::fetch_weather(client, latitude, longitude, temperature_unit).await {
        Ok(weather) => {
            let mut cache = cache.lock().await;
            cache.weather = Some(WeatherCache {
                value: weather.clone(),
                fetched_at: Instant::now(),
                latitude,
                longitude,
            });
            Ok(weather)
        }
        Err(error) => {
            if let Some(entry) = cached
                && entry.matches_location(latitude, longitude)
            {
                eprintln!("Échec récupération météo, cache expiré utilisé: {error:#}");
                return Ok(entry.value);
            }
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_location_exact() {
        let cache = WeatherCache {
            value: OpenMeteoCurrent {
                temperature_2m: 20.0,
                weather_code: 0,
            },
            fetched_at: Instant::now(),
            latitude: 48.8566,
            longitude: 2.3522,
        };
        assert!(cache.matches_location(48.8566, 2.3522));
    }

    #[test]
    fn test_matches_location_within_epsilon() {
        let cache = WeatherCache {
            value: OpenMeteoCurrent {
                temperature_2m: 20.0,
                weather_code: 0,
            },
            fetched_at: Instant::now(),
            latitude: 48.8566,
            longitude: 2.3522,
        };
        assert!(cache.matches_location(48.85665, 2.35225));
    }

    #[test]
    fn test_matches_location_different() {
        let cache = WeatherCache {
            value: OpenMeteoCurrent {
                temperature_2m: 20.0,
                weather_code: 0,
            },
            fetched_at: Instant::now(),
            latitude: 48.8566,
            longitude: 2.3522,
        };
        assert!(!cache.matches_location(40.7128, -74.0060));
    }
}
