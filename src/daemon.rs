use crate::cache::AppCache;
use crate::config::Config;
use crate::i18n::{t, t_fmt};
use crate::{hotkey, lookup};
use anyhow::{Context, Result};
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

fn is_wayland() -> bool {
    std::env::var("WAYLAND_DISPLAY").is_ok() && std::env::var("DISPLAY").is_err()
}

/// Builds the runtime and HTTP client up front, so a failure exits with an error.
pub fn build_runtime() -> Result<(Runtime, reqwest::Client)> {
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

/// Daemon mode: listens for the global hotkeys until `quit` (X11/XWayland).
pub fn run_daemon(config: Config) -> Result<()> {
    if is_wayland() {
        eprintln!("{}", t("wayland_hint"));
    }

    let (runtime, client) = build_runtime()?;
    let cache = Arc::new(Mutex::new(AppCache::default()));

    let primary_str = config.hotkeys.primary_str();
    let secondary_str = config.hotkeys.secondary_str();

    println!("{}", t("daemon_ready"));
    println!(
        "{}",
        t_fmt(
            "daemon_hotkeys",
            &[("primary", primary_str), ("secondary", secondary_str)]
        )
    );
    println!("{}", t("daemon_manual_hint"));
    println!("{}", t("daemon_quit_hint"));

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let manager = GlobalHotKeyManager::new().context("cannot start the global hotkey manager")?;

    let default_hotkey = HotKey::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyW);
    let hotkey_azerty = hotkey::parse_hotkey(primary_str, default_hotkey);
    let hotkey_qwerty = hotkey::parse_hotkey(secondary_str, default_hotkey);

    if hotkey_azerty == hotkey_qwerty {
        manager
            .register(hotkey_azerty)
            .context("cannot register the hotkey")?;
        let hotkey = format!("{hotkey_azerty:?}");
        println!("{}", t_fmt("active_hotkey", &[("hotkey", &hotkey)]));
    } else {
        manager
            .register(hotkey_azerty)
            .context("cannot register the primary hotkey")?;
        manager
            .register(hotkey_qwerty)
            .context("cannot register the secondary hotkey")?;
        let (primary, secondary) = (format!("{hotkey_azerty:?}"), format!("{hotkey_qwerty:?}"));
        println!(
            "{}",
            t_fmt(
                "active_hotkeys",
                &[("primary", &primary), ("secondary", &secondary)]
            )
        );
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
                    Trigger::Hotkey => println!("{}", t("hotkey_pressed")),
                    Trigger::Manual => println!("{}", t("manual_trigger")),
                }
                if let Err(error) = lookup::show_weather(&client, &config, &cache_for_runtime).await
                {
                    let error = format!("{error:#}");
                    eprintln!("{}", t_fmt("lookup_failed", &[("error", &error)]));
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
}
