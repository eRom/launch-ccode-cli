//! `lcc proxy status / logs / doctor` — V1 basique.

use crate::config::load_settings;
use crate::proxy::{
    health, keychain, launchctl, logs_dir, plist_path, wrapper_path, yaml_path, DEFAULT_PORT,
};
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::process::Command;
use std::time::Duration;

pub fn run_status() -> io::Result<()> {
    println!("=== lcc proxy status ===\n");
    let pid = launchctl::pid().ok().flatten();
    let alive = health::is_alive(DEFAULT_PORT, Duration::from_secs(1));
    println!(
        "Daemon       : {}",
        match pid {
            Some(p) => format!("up (PID {p})"),
            None => "down".to_string(),
        }
    );
    println!("Health check : {}", if alive { "OK" } else { "fail" });
    println!("Port         : {DEFAULT_PORT}");
    if yaml_path().exists() {
        let settings = load_settings().ok();
        let count = settings.map(|s| s.profiles.len()).unwrap_or(0);
        println!("Profils      : {count}");
    }
    Ok(())
}

pub fn run_logs(follow: bool) -> io::Result<()> {
    let log = logs_dir().join("litellm.err.log");
    if !log.exists() {
        println!("Pas de log à {} (daemon jamais lancé ?)", log.display());
        return Ok(());
    }
    if follow {
        // Délégation à `tail -f` du système (plus simple qu'une impl Rust).
        Command::new("tail")
            .args(["-f", &log.to_string_lossy()])
            .status()?;
    } else {
        let f = fs::File::open(&log)?;
        let reader = BufReader::new(f);
        for line in reader.lines().take(100) {
            println!("{}", line?);
        }
    }
    Ok(())
}

pub fn run_doctor() -> io::Result<()> {
    println!("=== lcc proxy doctor ===\n");
    check("uv installé", Command::new("uv").arg("--version").status().is_ok_and(|s| s.success()));
    check("plist présent", plist_path().exists());
    check("wrapper présent", wrapper_path().exists());
    check("yaml présent", yaml_path().exists());
    check("master_key dans Keychain", keychain::get_master_key().is_ok());
    check("daemon en cours", launchctl::pid().ok().flatten().is_some());
    check(
        "health check OK",
        health::is_alive(DEFAULT_PORT, Duration::from_secs(1)),
    );
    Ok(())
}

fn check(label: &str, ok: bool) {
    println!("[{}] {label}", if ok { "✓" } else { "✗" });
}
