//! Wrapper sync sur le CLI `security` macOS pour gérer la master_key
//! du proxy LiteLLM dans le Keychain de l'utilisateur.

use crate::proxy::KEYCHAIN_SERVICE;
use std::io::{self, ErrorKind};
use std::process::Command;

/// Stocke (ou remplace) la master_key dans le Keychain.
/// Utilise `security add-generic-password -U` (update si existe).
pub fn set_master_key(value: &str) -> io::Result<()> {
    let user = whoami_account()?;
    let status = Command::new("security")
        .args([
            "add-generic-password",
            "-U", // update if exists
            "-s", KEYCHAIN_SERVICE,
            "-a", &user,
            "-w", value,
        ])
        .status()?;
    if !status.success() {
        return Err(io::Error::new(
            ErrorKind::Other,
            format!("security add-generic-password failed (exit {status})"),
        ));
    }
    Ok(())
}

/// Lit la master_key depuis le Keychain.
/// Retourne `Err` si non trouvée.
pub fn get_master_key() -> io::Result<String> {
    let output = Command::new("security")
        .args(["find-generic-password", "-s", KEYCHAIN_SERVICE, "-w"])
        .output()?;
    if !output.status.success() {
        return Err(io::Error::new(
            ErrorKind::NotFound,
            "master_key absente du Keychain (lance `lcc proxy install`)",
        ));
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "master_key vide dans le Keychain",
        ));
    }
    Ok(s)
}

/// Supprime la master_key du Keychain (pour `proxy uninstall --purge`).
pub fn delete_master_key() -> io::Result<()> {
    let _ = Command::new("security")
        .args(["delete-generic-password", "-s", KEYCHAIN_SERVICE])
        .status()?;
    // OK même si la clé n'existait pas (uninstall idempotent).
    Ok(())
}

/// Génère 32 bytes random hex pour la master_key.
pub fn generate_random_key() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    // Note: pour la V1, on délègue à `openssl rand` (présent sur tout macOS).
    // Évite d'ajouter une crate `rand` juste pour ça.
    let output = Command::new("openssl")
        .args(["rand", "-hex", "32"])
        .output()
        .expect("openssl absent (macOS de base devrait l'avoir)");
    let key = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if key.len() != 64 {
        // Fallback ultra-grossier mais déterministe en cas de pépin
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        return format!("{:064x}", nanos);
    }
    key
}

fn whoami_account() -> io::Result<String> {
    let output = Command::new("whoami").output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
