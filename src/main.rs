use anyhow::{Context, Result};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use serde::Deserialize;
use tokio::sync::mpsc;

const IP_GEOLOCATION_URL: &str = "https://ipapi.co/json/";
const OPEN_METEO_BASE_URL: &str = "https://api.open-meteo.com/v1/forecast";

#[derive(Debug, Deserialize)]
struct IpApiResponse {
    city: Option<String>,
    region: Option<String>,
    country_name: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

#[derive(Debug)]
struct Location {
    city: Option<String>,
    region: Option<String>,
    country_name: Option<String>,
    latitude: f64,
    longitude: f64,
}

#[derive(Debug, Deserialize)]
struct OpenMeteoResponse {
    current: Option<OpenMeteoCurrent>,
}

#[derive(Debug, Deserialize)]
struct OpenMeteoCurrent {
    temperature_2m: f64,
    weather_code: i32,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Oasis Weather Notify - prêt.");
    println!("Raccourcis: Alt+A (AZERTY) ou Alt+Q (QWERTY).");

    let manager = GlobalHotKeyManager::new().context("Échec init manager hotkey")?;
    let hotkey_azerty = HotKey::new(Some(Modifiers::ALT), Code::KeyA);
    let hotkey_qwerty = HotKey::new(Some(Modifiers::ALT), Code::KeyQ);
    manager.register(hotkey_azerty).context("Échec enregistrement Alt+A")?;
    manager.register(hotkey_qwerty).context("Échec enregistrement Alt+Q")?;

    let receiver = GlobalHotKeyEvent::receiver();
    let (tx, mut rx) = mpsc::unbounded_channel();

    std::thread::spawn(move || {
        while let Ok(event) = receiver.recv() {
            let _ = tx.send(event);
        }
    });

    while let Some(event) = rx.recv().await {
        if event.state != HotKeyState::Pressed {
            continue;
        }

        if let Err(error) = handle_hotkey().await {
            eprintln!("Erreur lors de la récupération météo: {error:#}");
        }
    }

    Ok(())
}

async fn handle_hotkey() -> Result<()> {
    let location = fetch_ip_location().await?;
    let weather = fetch_weather(location.latitude, location.longitude).await?;

    let title = format!(
        "Météo actuelle - {}",
        location.city.as_deref().unwrap_or("Localisation inconnue")
    );
    let description = weather_description(weather.weather_code);
    let body = format!("{description} • {:.0}°C", weather.temperature_2m);
    let footer = format!(
        "{}, {}",
        location.region.as_deref().unwrap_or(""),
        location.country_name.as_deref().unwrap_or("")
    );
    let full_body = if footer.trim().is_empty() {
        body
    } else {
        format!("{body}\n{footer}")
    };

    send_notification(&title, &full_body)?;
    Ok(())
}

async fn fetch_ip_location() -> Result<Location> {
    let response = reqwest::get(IP_GEOLOCATION_URL)
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

async fn fetch_weather(latitude: f64, longitude: f64) -> Result<OpenMeteoCurrent> {
    let url = format!(
        "{OPEN_METEO_BASE_URL}?latitude={latitude}&longitude={longitude}&current=temperature_2m,weather_code&temperature_unit=celsius"
    );

    let response = reqwest::get(url)
        .await
        .context("Échec requête météo Open-Meteo")?
        .error_for_status()
        .context("Erreur HTTP météo Open-Meteo")?
        .json::<OpenMeteoResponse>()
        .await
        .context("Échec parsing météo Open-Meteo")?;

    response
        .current
        .context("Réponse météo incomplète")
}

fn weather_description(code: i32) -> &'static str {
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
        80 | 81 | 82 => "Averses",
        85 | 86 => "Averses de neige",
        95 => "Orage",
        96 | 99 => "Orage avec grêle",
        _ => "Conditions inconnues",
    }
}

#[cfg(target_os = "linux")]
fn send_notification(title: &str, body: &str) -> Result<()> {
    notify_rust::Notification::new()
        .summary(title)
        .body(body)
        .show()
        .context("Notification Linux (notify-rust) échouée")?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn send_notification(title: &str, body: &str) -> Result<()> {
    winrt_notification::Toast::new(winrt_notification::Toast::POWERSHELL_APP_ID)
        .title(title)
        .text1(body)
        .show()
        .context("Notification Windows (winrt-notification) échouée")?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn send_notification(title: &str, body: &str) -> Result<()> {
    mac_notification_sys::Notification::new(title)
        .subtitle(body)
        .send()
        .context("Notification macOS (mac-notification-sys) échouée")?;
    Ok(())
}
