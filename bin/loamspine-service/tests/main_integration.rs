// SPDX-License-Identifier: AGPL-3.0-or-later

//! Integration tests for the `loamspine` binary entry point.
//!
//! Exercises CLI argument parsing, subcommands, capabilities output,
//! socket path resolution, and server startup/shutdown.

#![allow(missing_docs)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;

/// Helper to get the loamspine binary for testing.
fn loamspine_cmd() -> Command {
    Command::cargo_bin("loamspine").expect("loamspine binary not found")
}

// ──────────────────────────────────────────────────────────────────────────────
// CLI help and version
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn cli_help_succeeds() {
    loamspine_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("LoamSpine"))
        .stdout(predicate::str::contains("Permanence"))
        .stdout(predicate::str::contains("server"))
        .stdout(predicate::str::contains("capabilities"))
        .stdout(predicate::str::contains("socket"));
}

#[test]
fn cli_version_succeeds() {
    loamspine_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("loamspine"));
}

#[test]
fn server_subcommand_help_succeeds() {
    loamspine_cmd()
        .args(["server", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tarpc-port"))
        .stdout(predicate::str::contains("jsonrpc-port"))
        .stdout(predicate::str::contains("bind-address"));
}

// ──────────────────────────────────────────────────────────────────────────────
// Capabilities subcommand
// ──────────────────────────────────────────────────────────────────────────────

#[test]
#[serial]
fn capabilities_outputs_valid_json() {
    let output = loamspine_cmd()
        .arg("capabilities")
        .output()
        .expect("capabilities subcommand failed");

    assert!(output.status.success(), "capabilities should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();

    // Must parse as valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(trimmed).expect("capabilities output must be valid JSON");

    // UniBin structure: primal, version, capabilities, methods
    assert!(parsed.get("primal").is_some(), "must have primal field");
    assert!(parsed.get("version").is_some(), "must have version field");
    assert!(
        parsed.get("capabilities").is_some(),
        "must have capabilities field"
    );
    assert!(parsed.get("methods").is_some(), "must have methods field");

    assert_eq!(
        parsed["primal"].as_str(),
        Some("loamspine"),
        "primal name must be loamspine"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Socket subcommand
// ──────────────────────────────────────────────────────────────────────────────

#[test]
#[serial]
fn socket_outputs_path() {
    let output = loamspine_cmd()
        .arg("socket")
        .output()
        .expect("socket subcommand failed");

    assert!(output.status.success(), "socket should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let path = stdout.trim();

    assert!(!path.is_empty(), "socket path must not be empty");
    assert!(
        path.contains("loamspine"),
        "socket path must contain 'loamspine', got: {path}"
    );
}

#[test]
#[serial]
fn socket_respects_loamspine_socket_env() {
    temp_env::with_vars(
        [("LOAMSPINE_SOCKET", Some("/custom/loamspine.sock"))],
        || {
            let output = loamspine_cmd()
                .arg("socket")
                .output()
                .expect("socket subcommand failed");

            assert!(output.status.success());
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert_eq!(stdout.trim(), "/custom/loamspine.sock");
        },
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Server subcommand
// ──────────────────────────────────────────────────────────────────────────────

#[test]
#[serial]
fn server_starts_and_shuts_down_via_signal() {
    use std::thread;
    use std::time::{Duration, Instant};

    let bin_path = assert_cmd::cargo::cargo_bin("loamspine");
    let mut child = std::process::Command::new(&bin_path)
        .args([
            "server",
            "--tarpc-port",
            "19001",
            "--jsonrpc-port",
            "18080",
            "--bind-address",
            "127.0.0.1",
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn loamspine server");

    thread::sleep(Duration::from_millis(1500));

    #[cfg(unix)]
    {
        use nix::sys::signal::{Signal, kill};
        use nix::unistd::Pid;

        let pid = child.id();
        kill(
            Pid::from_raw(i32::try_from(pid).expect("pid overflow")),
            Signal::SIGINT,
        )
        .expect("failed to send SIGINT to server");
    }

    #[cfg(not(unix))]
    {
        child.kill().expect("failed to kill server");
    }

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match child.try_wait() {
            Ok(Some(_status)) => {
                break;
            }
            Ok(None) => {}
            Err(e) => panic!("error waiting for server: {e}"),
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            panic!("server did not exit within timeout");
        }
        thread::sleep(Duration::from_millis(50));
    }
}

#[test]
#[serial]
fn server_with_invalid_bind_address_fails() {
    let output = loamspine_cmd()
        .args([
            "server",
            "--tarpc-port",
            "19002",
            "--jsonrpc-port",
            "18081",
            "--bind-address",
            "not-a-valid-ip",
        ])
        .timeout(std::time::Duration::from_secs(2))
        .output();

    let output = output.expect("server should have run");
    assert!(
        !output.status.success(),
        "server with invalid bind address should fail"
    );
}
