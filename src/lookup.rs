use crate::cache::{self, AppCache};
use crate::config::{Config, DEFAULT_LOCATION_TTL_SECONDS, DEFAULT_WEATHER_TTL_SECONDS};
use crate::{daemon, notification, weather};
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// One-shot mode: fetches the weather, shows the notification, then returns.
pub fn run_trigger(config: &Config) -> Result<()> {
    let (runtime, client) = daemon::build_runtime()?;
    let cache = Arc::new(Mutex::new(AppCache::default()));
    runtime.block_on(show_weather(&client, config, &cache))
}

/// Looks up the location and the weather, then shows them in a notification.
/// A failed lookup is reported in the notification, not returned as an error.
pub async fn show_weather(
    client: &reqwest::Client,
    config: &Config,
    cache: &Arc<Mutex<AppCache>>,
) -> Result<()> {
    let location = match cache::get_location_with_cache(
        client,
        cache,
        Duration::from_secs(
            config
                .cache
                .location_ttl_seconds
                .unwrap_or(DEFAULT_LOCATION_TTL_SECONDS),
        ),
    )
    .await
    {
        Ok(location) => location,
        Err(error) => {
            notification::send_notification(
                "Météo indisponible",
                &format!("Impossible de récupérer la localisation.\n{error:#}"),
            )?;
            return Ok(());
        }
    };

    let weather_data = match cache::get_weather_with_cache(
        client,
        cache,
        location.latitude,
        location.longitude,
        config.weather.temperature_unit,
        Duration::from_secs(
            config
                .cache
                .weather_ttl_seconds
                .unwrap_or(DEFAULT_WEATHER_TTL_SECONDS),
        ),
    )
    .await
    {
        Ok(w) => w,
        Err(error) => {
            notification::send_notification(
                "Météo indisponible",
                &format!("Impossible de récupérer la météo.\n{error:#}"),
            )?;
            return Ok(());
        }
    };

    let icon = weather::weather_icon(weather_data.weather_code);
    let title = format!(
        "{icon} Météo actuelle - {}",
        location.city.as_deref().unwrap_or("Localisation inconnue")
    );
    let description = weather::weather_description(weather_data.weather_code);
    let body = format!(
        "{description} • {:.0}{}",
        weather_data.temperature_2m,
        config.weather.temperature_unit.label()
    );

    let footer_parts: Vec<&str> = [location.region.as_deref(), location.country_name.as_deref()]
        .into_iter()
        .flatten()
        .filter(|s| !s.is_empty())
        .collect();
    let footer = footer_parts.join(", ");

    let full_body = if footer.is_empty() {
        body
    } else {
        format!("{body}\n{footer}")
    };

    notification::send_notification(&title, &full_body)?;
    Ok(())
}
