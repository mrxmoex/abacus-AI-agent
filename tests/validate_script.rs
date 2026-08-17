#![cfg(unix)]

use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf, process::Command};

use tempfile::tempdir;

fn script() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/validate.sh")
}

fn dry_run(mode: &[&str]) -> (bool, String, String) {
    let output = Command::new("sh")
        .arg(script())
        .args(mode)
        .arg("--dry-run")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn quick_checks_formatting_and_compilation_without_running_tests() {
    let (ok, stdout, stderr) = dry_run(&["quick"]);
    assert!(ok, "{stderr}");
    assert!(stdout.contains("cargo fmt --all -- --check"), "{stdout}");
    assert!(stdout.contains("cargo check --all-targets"), "{stdout}");
    assert!(!stdout.contains("cargo test"), "{stdout}");
}

#[test]
fn smoke_runs_the_headless_cli_test_that_needs_no_api_key() {
    let (ok, stdout, stderr) = dry_run(&["smoke"]);
    assert!(ok, "{stderr}");
    assert!(
        stdout.contains("cargo test --test cli_headless"),
        "{stdout}"
    );
}

#[test]
fn full_covers_every_ci_gate() {
    let (ok, stdout, stderr) = dry_run(&["full"]);
    assert!(ok, "{stderr}");
    assert!(stdout.contains("cargo fmt --all -- --check"), "{stdout}");
    assert!(
        stdout.contains("cargo clippy --all-targets -- -D warnings"),
        "{stdout}"
    );
    assert!(stdout.contains("cargo test --all-targets"), "{stdout}");
}

#[test]
fn no_argument_defaults_to_quick() {
    let (ok, stdout, stderr) = dry_run(&[]);
    assert!(ok, "{stderr}");
    assert_eq!(stdout, dry_run(&["quick"]).1);
}

#[test]
fn a_failing_step_stops_the_run_and_keeps_its_exit_code() {
    let directory = tempdir().unwrap();
    let calls = directory.path().join("calls");
    let cargo = directory.path().join("cargo");
    fs::write(
        &cargo,
        format!(
            "#!/bin/sh\necho \"$@\" >> {}\nexit 3\n",
            calls.to_str().unwrap()
        ),
    )
    .unwrap();
    fs::set_permissions(&cargo, fs::Permissions::from_mode(0o755)).unwrap();

    let output = Command::new("sh")
        .arg(script())
        .arg("quick")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env(
            "PATH",
            format!(
                "{}:{}",
                directory.path().display(),
                std::env::var("PATH").unwrap()
            ),
        )
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(3));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("failed"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(&calls).unwrap().lines().count(),
        1,
        "the run should stop at the first failing step"
    );
}

#[test]
fn pre_push_hook_runs_quick_validation() {
    let hook = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".githooks/pre-push");
    let mode = fs::metadata(&hook).unwrap().permissions().mode();
    assert_ne!(mode & 0o111, 0, "hook must be executable");

    // The hook honours ABACUS_HOOK_DRY_RUN so it can be exercised without
    // spawning a nested cargo build.
    let output = Command::new("sh")
        .arg(&hook)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("ABACUS_HOOK_DRY_RUN", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cargo fmt --all -- --check"), "{stdout}");
    assert!(stdout.contains("cargo check --all-targets"), "{stdout}");
}

#[test]
fn unknown_mode_fails_and_prints_the_supported_modes() {
    let output = Command::new("sh")
        .arg(script())
        .arg("everything")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("quick"), "{stderr}");
    assert!(stderr.contains("smoke"), "{stderr}");
    assert!(stderr.contains("full"), "{stderr}");
}
