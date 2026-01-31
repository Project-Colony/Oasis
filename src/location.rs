use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

const IP_GEOLOCATION_URL: &str = "https://ipapi.co/json/";
const IP_GEOLOCATION_FALLBACK_URL: &str = "https://ipwho.is/";

#[derive(Debug, Deserialize)]
struct IpApiResponse {
    city: Option<String>,
    region: Option<String>,
    country_name: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct IpWhoIsResponse {
    success: bool,
    message: Option<String>,
    city: Option<String>,
    region: Option<String>,
    country: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct Location {
    pub city: Option<String>,
    pub region: Option<String>,
    pub country_name: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
}

pub async fn fetch_ip_location(client: &reqwest::Client) -> Result<Location> {
    match fetch_ip_location_ipapi(client).await {
        Ok(location) => Ok(location),
        Err(error) => {
            eprintln!("Géolocalisation IP principale indisponible, fallback: {error:#}");
            fetch_ip_location_ipwho(client).await
        }
    }
}

async fn fetch_ip_location_ipapi(client: &reqwest::Client) -> Result<Location> {
    let response = client
        .get(IP_GEOLOCATION_URL)
        .send()
        .await
        .context("Échec requête géolocalisation IP")?
        .error_for_status()
        .context("Erreur HTTP géolocalisation IP")?
        .json::<IpApiResponse>()
        .await
        .context("Échec parsing géolocalisation IP")?;

    let latitude = response.latitude.context("Latitude manquante")?;
    let longitude = response.longitude.context("Longitude manquante")?;

    Ok(Location {
        city: response.city,
        region: response.region,
        country_name: response.country_name,
        latitude,
        longitude,
    })
}

async fn fetch_ip_location_ipwho(client: &reqwest::Client) -> Result<Location> {
    let response = client
        .get(IP_GEOLOCATION_FALLBACK_URL)
        .send()
        .await
        .context("Échec requête géolocalisation IP (fallback)")?
        .error_for_status()
        .context("Erreur HTTP géolocalisation IP (fallback)")?
        .json::<IpWhoIsResponse>()
        .await
        .context("Échec parsing géolocalisation IP (fallback)")?;

    if !response.success {
        let message = response
            .message
            .unwrap_or_else(|| "Réponse fallback invalide".to_string());
        return Err(anyhow!(message)).context("Géolocalisation IP fallback refusée");
    }

    let latitude = response.latitude.context("Latitude manquante (fallback)")?;
    let longitude = response
        .longitude
        .context("Longitude manquante (fallback)")?;

    Ok(Location {
        city: response.city,
        region: response.region,
        country_name: response.country,
        latitude,
        longitude,
    })
}
