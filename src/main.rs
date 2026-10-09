mod cache;
mod config;
mod hotkey;
mod location;
mod notification;
mod weather;

use anyhow::{Context, Result};
use cache::AppCache;
use config::{Config, DEFAULT_LOCATION_TTL_SECONDS, DEFAULT_WEATHER_TTL_SECONDS};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use std::io::{self, BufRead};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tao::event::Event;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tokio::runtime::{Builder, Runtime};
use tokio::sync::{Mutex, mpsc};

const HTTP_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
enum Trigger {
    Hotkey,
    Manual,
}

/// Event sent to the tao loop from other threads.
#[derive(Debug)]
enum UserEvent {
    Quit,
}

/// A command typed on stdin in daemon mode.
#[derive(Debug, PartialEq)]
enum Command {
    Quit,
    Trigger,
}

/// Enter or `w` triggers a lookup, `quit` stops the daemon, anything else is ignored.
fn parse_command(line: &str) -> Option<Command> {
    match line.trim().to_lowercase().as_str() {
        "quit" => Some(Command::Quit),
        "" | "w" => Some(Command::Trigger),
        _ => None,
    }
}

const USAGE: &str = "\
Oasis shows the current weather in a desktop notification.

Usage: oasis [OPTION]

With no option, Oasis runs as a daemon and listens for the global hotkey.

Options:
      --trigger  Show one notification, then exit
  -V, --version  Print the version, then exit
  -h, --help     Print this help, then exit

Environment:
  OASIS_CONFIG_PATH  Path of the TOML config file to read
";

/// What the command line asks Oasis to do.
#[derive(Debug, PartialEq)]
enum Cli {
    Daemon,
    Trigger,
    Version,
    Help,
    Unknown(String),
}

/// Parses the arguments after the program name. The parse stops at the first
/// `--version`, `--help` or unknown argument.
fn parse_args(args: impl IntoIterator<Item = String>) -> Cli {
    let mut cli = Cli::Daemon;
    for arg in args {
        match arg.as_str() {
            "-V" | "--version" => return Cli::Version,
            "-h" | "--help" => return Cli::Help,
            "--trigger" => cli = Cli::Trigger,
            _ => return Cli::Unknown(arg),
        }
    }
    cli
}

fn is_wayland() -> bool {
    std::env::var("WAYLAND_DISPLAY").is_ok() && std::env::var("DISPLAY").is_err()
}

fn main() -> Result<()> {
    // Before any config load, GUI init or network access, so `--version` works
    // headless and returns at once (the release workflow smoke-tests it).
    let args = std::env::args_os()
        .skip(1)
        .map(|arg| arg.to_string_lossy().into_owned());
    let trigger_mode = match parse_args(args) {
        Cli::Version => {
            println!("oasis {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Cli::Help => {
            print!("{USAGE}");
            return Ok(());
        }
        Cli::Unknown(arg) => {
            eprintln!("oasis: unknown argument '{arg}'\n");
            eprint!("{USAGE}");
            std::process::exit(2);
        }
        Cli::Trigger => true,
        Cli::Daemon => false,
    };

    let config = config::load_config().unwrap_or_else(|error| {
        eprintln!("Warning: {error:#}");
        eprintln!("Oasis continues with the default settings.");
        Config::default()
    });

    if trigger_mode {
        return run_trigger(&config);
    }

    if is_wayland() {
        eprintln!("Attention: Wayland détecté sans serveur X11.");
        eprintln!("Les raccourcis globaux ne fonctionnent pas sous Wayland pur.");
        eprintln!("Utilisez le mode one-shot avec votre compositeur:");
        eprintln!();
        eprintln!("  oasis --trigger");
        eprintln!();
        eprintln!("Exemple pour Hyprland (~/.config/hypr/hyprland.conf):");
        eprintln!("  bind = SUPER SHIFT, W, exec, oasis --trigger");
        eprintln!();
        eprintln!("Exemple pour Sway (~/.config/sway/config):");
        eprintln!("  bindsym Mod4+Shift+w exec oasis --trigger");
        eprintln!();
        eprintln!(
            "Lancement en mode daemon malgré tout (les raccourcis pourraient ne pas fonctionner)..."
        );
    }

    run_daemon(config)
}

/// Builds the runtime and HTTP client up front, so a failure exits with an error.
fn build_runtime() -> Result<(Runtime, reqwest::Client)> {
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to start the async runtime")?;
    let client = reqwest::Client::builder()
        .timeout(HTTP_TIMEOUT)
        .build()
        .context("failed to build the HTTP client")?;
    Ok((runtime, client))
}

/// Mode one-shot: récupère la météo, affiche la notification, quitte.
fn run_trigger(config: &Config) -> Result<()> {
    let (runtime, client) = build_runtime()?;
    let cache = Arc::new(Mutex::new(AppCache::default()));
    runtime.block_on(handle_hotkey(&client, config, &cache))
}

/// Mode daemon: écoute les raccourcis globaux en continu (X11/XWayland).
fn run_daemon(config: Config) -> Result<()> {
    let (runtime, client) = build_runtime()?;
    let cache = Arc::new(Mutex::new(AppCache::default()));

    let primary_str = config.hotkeys.primary_str();
    let secondary_str = config.hotkeys.secondary_str();

    println!("Oasis Weather Notify - prêt.");
    println!("Raccourcis: {primary_str} (AZERTY) ou {secondary_str} (QWERTY).");
    println!("Astuce: appuyez sur Entrée (ou tapez 'w') pour déclencher manuellement.");
    println!("Tapez 'quit' pour quitter.");

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let manager = GlobalHotKeyManager::new().context("Échec init manager hotkey")?;

    let default_hotkey = HotKey::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyW);
    let hotkey_azerty = hotkey::parse_hotkey(primary_str, default_hotkey);
    let hotkey_qwerty = hotkey::parse_hotkey(secondary_str, default_hotkey);

    if hotkey_azerty == hotkey_qwerty {
        manager
            .register(hotkey_azerty)
            .context("Échec enregistrement raccourci")?;
        println!("Raccourci actif: {hotkey_azerty:?}");
    } else {
        manager
            .register(hotkey_azerty)
            .context("Échec enregistrement raccourci primaire")?;
        manager
            .register(hotkey_qwerty)
            .context("Échec enregistrement raccourci secondaire")?;
        println!("Raccourcis actifs: {hotkey_azerty:?} et {hotkey_qwerty:?}");
    }

    let receiver = GlobalHotKeyEvent::receiver();
    let (tx, mut rx) = mpsc::unbounded_channel::<Trigger>();

    let tx_manual = tx.clone();
    let proxy = event_loop.create_proxy();
    std::thread::spawn(move || {
        // EOF or a read error only ends this thread: under Colony, or with
        // stdin from /dev/null, EOF arrives at once and the hotkeys must keep working.
        for line in io::stdin().lock().lines() {
            let Ok(line) = line else { break };
            match parse_command(&line) {
                Some(Command::Quit) => {
                    let _ = proxy.send_event(UserEvent::Quit);
                    break;
                }
                Some(Command::Trigger) => {
                    let _ = tx_manual.send(Trigger::Manual);
                }
                None => {}
            }
        }
    });

    let cache_for_runtime = Arc::clone(&cache);
    std::thread::spawn(move || {
        runtime.block_on(async move {
            while let Some(trigger) = rx.recv().await {
                match trigger {
                    Trigger::Hotkey => println!("Raccourci détecté, récupération météo..."),
                    Trigger::Manual => println!("Déclenchement manuel, récupération météo..."),
                }
                if let Err(error) = handle_hotkey(&client, &config, &cache_for_runtime).await {
                    eprintln!("Erreur lors de la récupération météo: {error:#}");
                }
            }
        });
    });

    let tx_hotkey = tx.clone();
    event_loop.run(move |event, _, control_flow| {
        if let Event::UserEvent(UserEvent::Quit) = event {
            *control_flow = ControlFlow::Exit;
            return;
        }
        *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(50));
        if let Ok(event) = receiver.try_recv()
            && event.state == HotKeyState::Pressed
        {
            let _ = tx_hotkey.send(Trigger::Hotkey);
        }
    });
}

async fn handle_hotkey(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_command_maps_stdin_lines() {
        assert_eq!(parse_command(""), Some(Command::Trigger));
        assert_eq!(parse_command("w"), Some(Command::Trigger));
        assert_eq!(parse_command("W "), Some(Command::Trigger));
        assert_eq!(parse_command("quit"), Some(Command::Quit));
        assert_eq!(parse_command("QUIT"), Some(Command::Quit));
        assert_eq!(parse_command("x"), None);
    }

    fn parse(args: &[&str]) -> Cli {
        parse_args(args.iter().map(|arg| arg.to_string()))
    }

    #[test]
    fn parse_args_maps_the_command_line() {
        assert_eq!(parse(&[]), Cli::Daemon);
        assert_eq!(parse(&["--trigger"]), Cli::Trigger);
        assert_eq!(parse(&["--version"]), Cli::Version);
        assert_eq!(parse(&["-V"]), Cli::Version);
        assert_eq!(parse(&["--help"]), Cli::Help);
        assert_eq!(parse(&["-h"]), Cli::Help);
        assert_eq!(parse(&["--trigger", "--version"]), Cli::Version);
        assert_eq!(parse(&["--bogus"]), Cli::Unknown("--bogus".into()));
        assert_eq!(parse(&["--trigger", "extra"]), Cli::Unknown("extra".into()));
    }
}
