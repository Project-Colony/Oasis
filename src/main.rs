mod cache;
mod config;
mod daemon;
mod hotkey;
mod i18n;
mod location;
mod lookup;
mod notification;
mod weather;

use anyhow::Result;
use config::{Config, Language};
use i18n::{t, t_fmt};

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

fn main() -> Result<()> {
    // The system language until the config is read: --help and the messages
    // printed while the config loads use it.
    i18n::set_language(Language::Auto);

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
            print!("{}", t("usage"));
            return Ok(());
        }
        Cli::Unknown(arg) => {
            eprintln!("{}\n", t_fmt("unknown_argument", &[("arg", &arg)]));
            eprint!("{}", t("usage"));
            std::process::exit(2);
        }
        Cli::Trigger => true,
        Cli::Daemon => false,
    };

    let config = config::load_config().unwrap_or_else(|error| {
        eprintln!(
            "{}",
            t_fmt("config_warning", &[("error", &format!("{error:#}"))])
        );
        eprintln!("{}", t("config_defaults"));
        Config::default()
    });
    i18n::set_language(config.general.language);

    if trigger_mode {
        return lookup::run_trigger(&config);
    }

    daemon::run_daemon(config)
}

#[cfg(test)]
mod tests {
    use super::*;

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
