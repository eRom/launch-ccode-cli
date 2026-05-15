//! Wrapper sync sur `launchctl` pour gérer le LaunchAgent du proxy.

use crate::proxy::{plist_path, LAUNCHD_LABEL};
use std::io::{self, ErrorKind};
use std::process::Command;

fn current_uid() -> io::Result<u32> {
    let output = Command::new("id").arg("-u").output()?;
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    s.parse::<u32>()
        .map_err(|e| io::Error::new(ErrorKind::Other, format!("uid parse: {e}")))
}

fn service_target() -> io::Result<String> {
    let uid = current_uid()?;
    Ok(format!("gui/{uid}/{LAUNCHD_LABEL}"))
}

pub fn load() -> io::Result<()> {
    let plist = plist_path();
    let status = Command::new("launchctl")
        .args(["load", "-w"])
        .arg(&plist)
        .status()?;
    if !status.success() {
        return Err(io::Error::new(
            ErrorKind::Other,
            format!("launchctl load failed (exit {status})"),
        ));
    }
    Ok(())
}

pub fn unload() -> io::Result<()> {
    let plist = plist_path();
    let _ = Command::new("launchctl")
        .args(["unload", "-w"])
        .arg(&plist)
        .status()?;
    Ok(())
}

pub fn kickstart() -> io::Result<()> {
    let target = service_target()?;
    let status = Command::new("launchctl")
        .args(["kickstart", "-k", &target])
        .status()?;
    if !status.success() {
        return Err(io::Error::new(
            ErrorKind::Other,
            format!("launchctl kickstart failed (exit {status})"),
        ));
    }
    Ok(())
}

/// Vérifie si le LaunchAgent est chargé. Retourne le PID si en cours.
pub fn pid() -> io::Result<Option<u32>> {
    let target = service_target()?;
    let output = Command::new("launchctl")
        .args(["print", &target])
        .output()?;
    if !output.status.success() {
        return Ok(None);
    }
    let s = String::from_utf8_lossy(&output.stdout);
    for line in s.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("pid = ") {
            return Ok(rest.parse::<u32>().ok());
        }
    }
    Ok(None)
}
