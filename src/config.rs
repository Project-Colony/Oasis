use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_PRIMARY_HOTKEY: &str = "Super+Shift+W";
const DEFAULT_SECONDARY_HOTKEY: &str = "Super+Shift+W";
pub const DEFAULT_LOCATION_TTL_SECONDS: u64 = 60 * 60 * 24;
pub const DEFAULT_WEATHER_TTL_SECONDS: u64 = 60 * 5;

/// The directory name under `Colony/`, spelled as the program spells it.
const PROGRAM: &str = "Oasis";
/// Written next to the new config once the legacy file has been copied.
const MIGRATION_MARKER: &str = ".migrated";

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
        self.secondary
            .as_deref()
            .unwrap_or(DEFAULT_SECONDARY_HOTKEY)
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

/// Missing file gives the defaults. A file that cannot be read or parsed is an error
/// that names the path.
pub fn load_config() -> Result<Config> {
    match config_path() {
        Some(path) => load_config_from(&path),
        None => Ok(Config::default()),
    }
}

fn load_config_from(path: &Path) -> Result<Config> {
    if !path.exists() {
        return Ok(Config::default());
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("cannot read the config file {}", path.display()))?;
    toml::from_str(&content)
        .with_context(|| format!("cannot parse the config file {}", path.display()))
}

/// The file Oasis reads: `OASIS_CONFIG_PATH` when set, otherwise
/// `<config>/Colony/Oasis/preferences/config.toml`, after a one-time copy from
/// the location earlier releases used.
fn config_path() -> Option<PathBuf> {
    if let Some(path) = env::var_os("OASIS_CONFIG_PATH") {
        return Some(PathBuf::from(path));
    }

    let legacy = legacy_config_path();
    // `locate` creates nothing: a normal launch leaves no empty directory behind.
    let Ok(dir) = colony_ui::paths::locate::config_dir(PROGRAM) else {
        return legacy;
    };
    Some(resolve_config_path(
        &dir.join("preferences").join("config.toml"),
        legacy.as_deref(),
    ))
}

/// The file releases before the Colony layout read, chosen the same way they did.
fn legacy_config_path() -> Option<PathBuf> {
    if let Some(xdg) = env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg).join("oasis").join("config.toml"));
    }
    if let Some(home) = env::var_os("HOME") {
        return Some(
            PathBuf::from(home)
                .join(".config")
                .join("oasis")
                .join("config.toml"),
        );
    }
    env::var_os("APPDATA").map(|appdata| PathBuf::from(appdata).join("Oasis").join("config.toml"))
}

/// Picks the file to read and copies the legacy file to `new` on first run.
///
/// A marker next to `new` records the copy, so a user who later deletes the new
/// file gets the defaults rather than the old file again. The legacy file is
/// never deleted. If the copy fails, Oasis keeps reading the legacy file.
fn resolve_config_path(new: &Path, legacy: Option<&Path>) -> PathBuf {
    let marker = new.with_file_name(MIGRATION_MARKER);
    if new.exists() || marker.exists() {
        return new.to_path_buf();
    }
    let Some(legacy) = legacy.filter(|path| path.is_file()) else {
        return new.to_path_buf();
    };

    match migrate(legacy, new, &marker) {
        Ok(()) => {
            println!(
                "Oasis copied its config from {} to {}.",
                legacy.display(),
                new.display()
            );
            new.to_path_buf()
        }
        Err(error) => {
            eprintln!(
                "Warning: cannot copy the config from {} to {}: {error}. Oasis keeps reading the old file.",
                legacy.display(),
                new.display()
            );
            legacy.to_path_buf()
        }
    }
}

fn migrate(legacy: &Path, new: &Path, marker: &Path) -> std::io::Result<()> {
    if let Some(dir) = new.parent() {
        fs::create_dir_all(dir)?;
    }
    if let Err(error) = fs::copy(legacy, new) {
        // A half-written copy would win over the legacy file on the next launch.
        let _ = fs::remove_file(new);
        return Err(error);
    }
    fs::write(marker, legacy.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        env::temp_dir().join(format!("oasis-{}-{name}", std::process::id()))
    }

    #[test]
    fn missing_file_gives_defaults() {
        let config = load_config_from(&temp_path("missing.toml")).unwrap();
        assert_eq!(config.hotkeys.primary_str(), DEFAULT_PRIMARY_HOTKEY);
        assert_eq!(
            config.cache.location_ttl_seconds,
            Some(DEFAULT_LOCATION_TTL_SECONDS)
        );
    }

    #[test]
    fn malformed_toml_is_an_error_naming_the_path() {
        let path = temp_path("malformed.toml");
        fs::write(&path, "[hotkeys\nprimary = ").unwrap();
        let result = load_config_from(&path);
        fs::remove_file(&path).unwrap();
        let error = format!("{:#}", result.unwrap_err());
        assert!(error.contains(&path.display().to_string()), "{error}");
    }

    /// A fresh, empty directory per test, so no real home is touched.
    fn temp_dir(name: &str) -> PathBuf {
        let dir = temp_path(name);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn new_config(root: &Path) -> PathBuf {
        root.join("Colony/Oasis/preferences/config.toml")
    }

    fn legacy_config(root: &Path, content: &str) -> PathBuf {
        let legacy = root.join("oasis/config.toml");
        fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        fs::write(&legacy, content).unwrap();
        legacy
    }

    #[test]
    fn new_config_wins_over_legacy() {
        let root = temp_dir("new-wins");
        let legacy = legacy_config(&root, "old");
        let new = new_config(&root);
        fs::create_dir_all(new.parent().unwrap()).unwrap();
        fs::write(&new, "new").unwrap();

        assert_eq!(resolve_config_path(&new, Some(&legacy)), new);
        assert_eq!(fs::read_to_string(&new).unwrap(), "new");
        assert!(!new.with_file_name(MIGRATION_MARKER).exists());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn legacy_config_is_copied_once_and_kept() {
        let root = temp_dir("migrate");
        let legacy = legacy_config(&root, "[weather]\ntemperature_unit = \"fahrenheit\"\n");
        let new = new_config(&root);

        assert_eq!(resolve_config_path(&new, Some(&legacy)), new);
        assert_eq!(
            fs::read_to_string(&new).unwrap(),
            fs::read_to_string(&legacy).unwrap()
        );
        assert!(new.with_file_name(MIGRATION_MARKER).exists());
        assert!(legacy.exists());
        let config = load_config_from(&new).unwrap();
        assert!(matches!(
            config.weather.temperature_unit,
            TemperatureUnit::Fahrenheit
        ));

        // The user deletes the new file: the marker stops a second copy.
        fs::remove_file(&new).unwrap();
        assert_eq!(resolve_config_path(&new, Some(&legacy)), new);
        assert!(!new.exists());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn failed_copy_falls_back_to_legacy() {
        let root = temp_dir("copy-fails");
        let legacy = legacy_config(&root, "old");
        // A file where the Colony directory should be makes create_dir_all fail.
        fs::write(root.join("Colony"), "").unwrap();
        let new = new_config(&root);

        assert_eq!(resolve_config_path(&new, Some(&legacy)), legacy);
        assert!(legacy.exists());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn no_legacy_config_gives_the_new_path_untouched() {
        let root = temp_dir("fresh");
        let new = new_config(&root);

        assert_eq!(
            resolve_config_path(&new, Some(&root.join("missing.toml"))),
            new
        );
        assert_eq!(resolve_config_path(&new, None), new);
        assert!(!root.join("Colony").exists());
        fs::remove_dir_all(&root).unwrap();
    }
}
