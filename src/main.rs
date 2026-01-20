use anyhow::{Context, Result, anyhow};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use serde::Deserialize;
use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tokio::runtime::Builder;
use tokio::sync::{Mutex, mpsc};

const IP_GEOLOCATION_URL: &str = "https://ipapi.co/json/";
const IP_GEOLOCATION_FALLBACK_URL: &str = "https://ipwho.is/";
const OPEN_METEO_BASE_URL: &str = "https://api.open-meteo.com/v1/forecast";

const DEFAULT_PRIMARY_HOTKEY: &str = "Alt+A";
const DEFAULT_SECONDARY_HOTKEY: &str = "Alt+Q";
const DEFAULT_LOCATION_TTL_SECONDS: u64 = 60 * 60 * 24;
const DEFAULT_WEATHER_TTL_SECONDS: u64 = 60 * 5;

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(default)]
    hotkeys: HotkeyConfig,
    #[serde(default)]
    cache: CacheConfig,
    #[serde(default)]
    weather: WeatherConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkeys: HotkeyConfig::default(),
            cache: CacheConfig::default(),
            weather: WeatherConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct HotkeyConfig {
    primary: Option<String>,
    secondary: Option<String>,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            primary: Some(DEFAULT_PRIMARY_HOTKEY.to_string()),
            secondary: Some(DEFAULT_SECONDARY_HOTKEY.to_string()),
        }
    }
}

#[derive(Debug, Deserialize)]
struct CacheConfig {
    location_ttl_seconds: Option<u64>,
    weather_ttl_seconds: Option<u64>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            location_ttl_seconds: Some(DEFAULT_LOCATION_TTL_SECONDS),
            weather_ttl_seconds: Some(DEFAULT_WEATHER_TTL_SECONDS),
        }
    }
}

#[derive(Debug, Deserialize)]
struct WeatherConfig {
    #[serde(default)]
    temperature_unit: TemperatureUnit,
}

impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            temperature_unit: TemperatureUnit::Celsius,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum TemperatureUnit {
    Celsius,
    Fahrenheit,
}

impl Default for TemperatureUnit {
    fn default() -> Self {
        Self::Celsius
    }
}

impl TemperatureUnit {
    fn as_query_param(self) -> &'static str {
        match self {
            TemperatureUnit::Celsius => "celsius",
            TemperatureUnit::Fahrenheit => "fahrenheit",
        }
    }

    fn label(self) -> &'static str {
        match self {
            TemperatureUnit::Celsius => "°C",
            TemperatureUnit::Fahrenheit => "°F",
        }
    }
}

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

#[derive(Debug, Deserialize, Clone)]
struct OpenMeteoCurrent {
    temperature_2m: f64,
    weather_code: i32,
}

#[derive(Debug, Clone)]
struct Cached<T> {
    value: T,
    fetched_at: Instant,
}

#[derive(Debug, Clone)]
struct WeatherCache {
    value: OpenMeteoCurrent,
    fetched_at: Instant,
    latitude: f64,
    longitude: f64,
}

#[derive(Debug, Default)]
struct AppCache {
    location: Option<Cached<Location>>,
    weather: Option<WeatherCache>,
}

#[derive(Debug)]
enum Trigger {
    Hotkey,
    Manual,
}

fn main() -> Result<()> {
    let config = load_config().unwrap_or_default();
    let cache = Arc::new(Mutex::new(AppCache::default()));

    println!("Oasis Weather Notify - prêt.");
    println!(
        "Raccourcis: {} (AZERTY) ou {} (QWERTY).",
        config
            .hotkeys
            .primary
            .as_deref()
            .unwrap_or(DEFAULT_PRIMARY_HOTKEY),
        config
            .hotkeys
            .secondary
            .as_deref()
            .unwrap_or(DEFAULT_SECONDARY_HOTKEY)
    );
    println!("Astuce: appuyez sur Entrée (ou tapez 'w') pour déclencher manuellement.");
    println!("Tapez 'quit' pour quitter.");

    let event_loop = EventLoopBuilder::new().build();
    let manager = GlobalHotKeyManager::new().context("Échec init manager hotkey")?;
    let hotkey_azerty = parse_hotkey(
        config
            .hotkeys
            .primary
            .as_deref()
            .unwrap_or(DEFAULT_PRIMARY_HOTKEY),
        HotKey::new(Some(Modifiers::ALT), Code::KeyA),
    );
    let hotkey_qwerty = parse_hotkey(
        config
            .hotkeys
            .secondary
            .as_deref()
            .unwrap_or(DEFAULT_SECONDARY_HOTKEY),
        HotKey::new(Some(Modifiers::ALT), Code::KeyQ),
    );
    if hotkey_azerty == hotkey_qwerty {
        manager
            .register(hotkey_azerty)
            .context("Échec enregistrement raccourci")?;
        println!("Raccourci actif: {hotkey_azerty:?}");
    } else {
        manager
            .register(hotkey_azerty)
            .context("Échec enregistrement Alt+A")?;
        manager
            .register(hotkey_qwerty)
            .context("Échec enregistrement Alt+Q")?;
        println!("Raccourcis actifs: {hotkey_azerty:?} et {hotkey_qwerty:?}");
    }

    let receiver = GlobalHotKeyEvent::receiver();
    let (tx, mut rx) = mpsc::unbounded_channel::<Trigger>();

    let tx_manual = tx.clone();
    std::thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let input = match line {
                Ok(text) => text.trim().to_lowercase(),
                Err(_) => continue,
            };
            if input == "quit" {
                break;
            }
            if input.is_empty() || input == "w" {
                let _ = tx_manual.send(Trigger::Manual);
            }
        }
    });

    let config_for_runtime = config;
    let cache_for_runtime = Arc::clone(&cache);
    std::thread::spawn(move || {
        let runtime = Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Échec init runtime tokio");
        runtime.block_on(async move {
            while let Some(trigger) = rx.recv().await {
                match trigger {
                    Trigger::Hotkey => println!("Raccourci détecté, récupération météo..."),
                    Trigger::Manual => println!("Déclenchement manuel, récupération météo..."),
                }
                if let Err(error) = handle_hotkey(&config_for_runtime, &cache_for_runtime).await {
                    eprintln!("Erreur lors de la récupération météo: {error:#}");
                }
            }
        });
    });

    let tx_hotkey = tx.clone();
    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Ok(event) = receiver.try_recv() {
            if event.state == HotKeyState::Pressed {
                let _ = tx_hotkey.send(Trigger::Hotkey);
            }
        }
    });
}

async fn handle_hotkey(config: &Config, cache: &Arc<Mutex<AppCache>>) -> Result<()> {
    let location = match get_location_with_cache(
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
            send_notification(
                "Météo indisponible",
                &format!("Impossible de récupérer la localisation.\n{error:#}"),
            )?;
            return Ok(());
        }
    };

    let weather = match get_weather_with_cache(
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
        Ok(weather) => weather,
        Err(error) => {
            send_notification(
                "Météo indisponible",
                &format!("Impossible de récupérer la météo.\n{error:#}"),
            )?;
            return Ok(());
        }
    };

    let icon = weather_icon(weather.weather_code);
    let title = format!(
        "{icon} Météo actuelle - {}",
        location.city.as_deref().unwrap_or("Localisation inconnue")
    );
    let description = weather_description(weather.weather_code);
    let body = format!(
        "{description} • {:.0}{}",
        weather.temperature_2m,
        config.weather.temperature_unit.label()
    );
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
    match fetch_ip_location_ipapi().await {
        Ok(location) => Ok(location),
        Err(error) => {
            eprintln!("Géolocalisation IP principale indisponible, fallback: {error:#}");
            fetch_ip_location_ipwho().await
        }
    }
}

async fn get_location_with_cache(cache: &Arc<Mutex<AppCache>>, ttl: Duration) -> Result<Location> {
    let cached = {
        let cache = cache.lock().await;
        cache.location.clone()
    };

    if let Some(entry) = cached.as_ref() {
        if entry.fetched_at.elapsed() <= ttl {
            return Ok(entry.value.clone());
        }
    }

    match fetch_ip_location().await {
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

async fn fetch_ip_location_ipapi() -> Result<Location> {
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

async fn fetch_ip_location_ipwho() -> Result<Location> {
    let response = reqwest::get(IP_GEOLOCATION_FALLBACK_URL)
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

async fn fetch_weather(
    latitude: f64,
    longitude: f64,
    temperature_unit: TemperatureUnit,
) -> Result<OpenMeteoCurrent> {
    let url = format!(
        "{OPEN_METEO_BASE_URL}?latitude={latitude}&longitude={longitude}&current=temperature_2m,weather_code&temperature_unit={}",
        temperature_unit.as_query_param()
    );

    let response = reqwest::get(url)
        .await
        .context("Échec requête météo Open-Meteo")?
        .error_for_status()
        .context("Erreur HTTP météo Open-Meteo")?
        .json::<OpenMeteoResponse>()
        .await
        .context("Échec parsing météo Open-Meteo")?;

    response.current.context("Réponse météo incomplète")
}

async fn get_weather_with_cache(
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

    if let Some(entry) = cached.as_ref() {
        let same_location = (entry.latitude - latitude).abs() < 0.0001
            && (entry.longitude - longitude).abs() < 0.0001;
        if same_location && entry.fetched_at.elapsed() <= ttl {
            return Ok(entry.value.clone());
        }
    }

    match fetch_weather(latitude, longitude, temperature_unit).await {
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
            if let Some(entry) = cached {
                let same_location = (entry.latitude - latitude).abs() < 0.0001
                    && (entry.longitude - longitude).abs() < 0.0001;
                if same_location {
                    eprintln!("Échec récupération météo, cache expiré utilisé: {error:#}");
                    return Ok(entry.value);
                }
            }
            Err(error)
        }
    }
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

fn weather_icon(code: i32) -> &'static str {
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
        80 | 81 | 82 => "🌧️",
        85 | 86 => "🌨️",
        95 | 96 | 99 => "⛈️",
        _ => "🌡️",
    }
}

fn load_config() -> Result<Config> {
    let Some(path) = config_path() else {
        return Ok(Config::default());
    };

    if !path.exists() {
        return Ok(Config::default());
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("Lecture config impossible: {}", path.display()))?;
    toml::from_str(&content).context("Parsing config TOML impossible")
}

fn config_path() -> Option<PathBuf> {
    if let Ok(path) = env::var("OASIS_CONFIG_PATH") {
        return Some(PathBuf::from(path));
    }

    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg).join("oasis").join("config.toml"));
    }

    if let Ok(home) = env::var("HOME") {
        return Some(
            PathBuf::from(home)
                .join(".config")
                .join("oasis")
                .join("config.toml"),
        );
    }

    if let Ok(appdata) = env::var("APPDATA") {
        return Some(PathBuf::from(appdata).join("Oasis").join("config.toml"));
    }

    None
}

fn parse_hotkey(configured: &str, fallback: HotKey) -> HotKey {
    match parse_hotkey_inner(configured) {
        Ok(hotkey) => hotkey,
        Err(error) => {
            eprintln!("Hotkey invalide '{configured}', fallback utilisé: {error:#}");
            fallback
        }
    }
}

fn parse_hotkey_inner(configured: &str) -> Result<HotKey> {
    let parts: Vec<&str> = configured
        .split('+')
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .collect();

    if parts.is_empty() {
        return Err(anyhow!("Hotkey vide"));
    }

    let mut modifiers = Modifiers::empty();
    let mut code: Option<Code> = None;

    for part in parts {
        match part.to_lowercase().as_str() {
            "alt" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "super" | "meta" | "cmd" | "command" | "win" => modifiers |= Modifiers::SUPER,
            key => {
                if code.is_some() {
                    return Err(anyhow!("Trop de touches principales"));
                }
                code = Some(parse_key_code(key)?);
            }
        }
    }

    let code = code.context("Touche principale manquante")?;
    Ok(HotKey::new(Some(modifiers), code))
}

fn parse_key_code(key: &str) -> Result<Code> {
    let upper = key.to_uppercase();
    let mut chars = upper.chars();
    if let (Some(single), None) = (chars.next(), chars.next()) {
        if single.is_ascii_alphabetic() {
            return match single {
                'A' => Ok(Code::KeyA),
                'B' => Ok(Code::KeyB),
                'C' => Ok(Code::KeyC),
                'D' => Ok(Code::KeyD),
                'E' => Ok(Code::KeyE),
                'F' => Ok(Code::KeyF),
                'G' => Ok(Code::KeyG),
                'H' => Ok(Code::KeyH),
                'I' => Ok(Code::KeyI),
                'J' => Ok(Code::KeyJ),
                'K' => Ok(Code::KeyK),
                'L' => Ok(Code::KeyL),
                'M' => Ok(Code::KeyM),
                'N' => Ok(Code::KeyN),
                'O' => Ok(Code::KeyO),
                'P' => Ok(Code::KeyP),
                'Q' => Ok(Code::KeyQ),
                'R' => Ok(Code::KeyR),
                'S' => Ok(Code::KeyS),
                'T' => Ok(Code::KeyT),
                'U' => Ok(Code::KeyU),
                'V' => Ok(Code::KeyV),
                'W' => Ok(Code::KeyW),
                'X' => Ok(Code::KeyX),
                'Y' => Ok(Code::KeyY),
                'Z' => Ok(Code::KeyZ),
                _ => Err(anyhow!("Touche invalide: {key}")),
            };
        }
        if single.is_ascii_digit() {
            return match single {
                '0' => Ok(Code::Digit0),
                '1' => Ok(Code::Digit1),
                '2' => Ok(Code::Digit2),
                '3' => Ok(Code::Digit3),
                '4' => Ok(Code::Digit4),
                '5' => Ok(Code::Digit5),
                '6' => Ok(Code::Digit6),
                '7' => Ok(Code::Digit7),
                '8' => Ok(Code::Digit8),
                '9' => Ok(Code::Digit9),
                _ => Err(anyhow!("Touche invalide: {key}")),
            };
        }
    }

    Err(anyhow!("Touche invalide: {key}"))
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
    if let Err(error) = send_windows_toast(title, body) {
        eprintln!("Échec notification Windows, fallback MessageBox: {error:#}");
        show_windows_message_box(title, body);
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn send_windows_toast(title: &str, body: &str) -> Result<()> {
    winrt_notification::Toast::new(winrt_notification::Toast::POWERSHELL_APP_ID)
        .title(title)
        .text1(body)
        .show()
        .context("Notification Windows (winrt-notification) échouée")
}

#[cfg(target_os = "windows")]
fn show_windows_message_box(title: &str, body: &str) {
    use std::ffi::OsStr;
    use std::iter;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONINFORMATION, MB_OK, MessageBoxW};

    let title_wide: Vec<u16> = OsStr::new(title)
        .encode_wide()
        .chain(iter::once(0))
        .collect();
    let body_wide: Vec<u16> = OsStr::new(body)
        .encode_wide()
        .chain(iter::once(0))
        .collect();

    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            body_wide.as_ptr(),
            title_wide.as_ptr(),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}

#[cfg(target_os = "macos")]
fn send_notification(title: &str, body: &str) -> Result<()> {
    mac_notification_sys::Notification::new(title)
        .subtitle(body)
        .send()
        .context("Notification macOS (mac-notification-sys) échouée")?;
    Ok(())
}
