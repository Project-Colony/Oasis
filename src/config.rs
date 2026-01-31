use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;

const DEFAULT_PRIMARY_HOTKEY: &str = "Super+Shift+W";
const DEFAULT_SECONDARY_HOTKEY: &str = "Super+Shift+W";
pub const DEFAULT_LOCATION_TTL_SECONDS: u64 = 60 * 60 * 24;
pub const DEFAULT_WEATHER_TTL_SECONDS: u64 = 60 * 5;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub hotkeys: HotkeyConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub weather: WeatherConfig,
}

#[derive(Debug, Deserialize)]
pub struct HotkeyConfig {
    pub primary: Option<String>,
    pub secondary: Option<String>,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            primary: Some(DEFAULT_PRIMARY_HOTKEY.to_string()),
            secondary: Some(DEFAULT_SECONDARY_HOTKEY.to_string()),
        }
    }
}

impl HotkeyConfig {
    pub fn primary_str(&self) -> &str {
        self.primary.as_deref().unwrap_or(DEFAULT_PRIMARY_HOTKEY)
    }

    pub fn secondary_str(&self) -> &str {
        self.secondary.as_deref().unwrap_or(DEFAULT_SECONDARY_HOTKEY)
    }
}

#[derive(Debug, Deserialize)]
pub struct CacheConfig {
    pub location_ttl_seconds: Option<u64>,
    pub weather_ttl_seconds: Option<u64>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            location_ttl_seconds: Some(DEFAULT_LOCATION_TTL_SECONDS),
            weather_ttl_seconds: Some(DEFAULT_WEATHER_TTL_SECONDS),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct WeatherConfig {
    #[serde(default)]
    pub temperature_unit: TemperatureUnit,
}

#[derive(Debug, Deserialize, Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum TemperatureUnit {
    #[default]
    Celsius,
    Fahrenheit,
}

impl TemperatureUnit {
    pub fn as_query_param(self) -> &'static str {
        match self {
            TemperatureUnit::Celsius => "celsius",
            TemperatureUnit::Fahrenheit => "fahrenheit",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            TemperatureUnit::Celsius => "°C",
            TemperatureUnit::Fahrenheit => "°F",
        }
    }
}

pub fn load_config() -> Result<Config> {
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
