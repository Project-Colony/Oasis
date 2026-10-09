//! Runs the built binary the way the release workflow smoke-tests it: no
//! display, no stdin, and it must return at once.

use std::process::Command;

#[test]
fn version_prints_the_version_and_exits_zero() {
    let output = Command::new(env!("CARGO_BIN_EXE_oasis"))
        .arg("--version")
        .output()
        .expect("run oasis --version");

    assert!(output.status.success(), "exit status: {}", output.status);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout.trim(),
        format!("oasis {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn unknown_argument_exits_two_instead_of_starting_the_daemon() {
    let output = Command::new(env!("CARGO_BIN_EXE_oasis"))
        .arg("--bogus")
        .output()
        .expect("run oasis --bogus");

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Usage: oasis"));
}
