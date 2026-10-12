use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

use crate::i18n::t_fmt;

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
            let error = format!("{error:#}");
            eprintln!("{}", t_fmt("ip_lookup_fallback", &[("error", &error)]));
            fetch_ip_location_ipwho(client).await
        }
    }
}

async fn fetch_ip_location_ipapi(client: &reqwest::Client) -> Result<Location> {
    let response = client
        .get(IP_GEOLOCATION_URL)
        .send()
        .await
        .context("the IP geolocation request failed")?
        .error_for_status()
        .context("the IP geolocation service returned an HTTP error")?
        .json::<IpApiResponse>()
        .await
        .context("cannot parse the IP geolocation response")?;

    let latitude = response
        .latitude
        .context("the IP geolocation response has no latitude")?;
    let longitude = response
        .longitude
        .context("the IP geolocation response has no longitude")?;

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
        .context("the fallback IP geolocation request failed")?
        .error_for_status()
        .context("the fallback IP geolocation service returned an HTTP error")?
        .json::<IpWhoIsResponse>()
        .await
        .context("cannot parse the fallback IP geolocation response")?;

    if !response.success {
        let message = response
            .message
            .unwrap_or_else(|| "no reason given".to_string());
        return Err(anyhow!(message))
            .context("the fallback IP geolocation service refused the lookup");
    }

    let latitude = response
        .latitude
        .context("the fallback IP geolocation response has no latitude")?;
    let longitude = response
        .longitude
        .context("the fallback IP geolocation response has no longitude")?;

    Ok(Location {
        city: response.city,
        region: response.region,
        country_name: response.country,
        latitude,
        longitude,
    })
}
