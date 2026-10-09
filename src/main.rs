mod cache;
mod config;
mod daemon;
mod hotkey;
mod location;
mod lookup;
mod notification;
mod weather;

use anyhow::Result;
use config::Config;

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
