//! Orchestration de `lcc proxy install`.
//!
//! Étapes :
//! 1. Vérifier `uv` présent
//! 2. `uv tool install litellm[proxy]`
//! 3. Générer master_key + Keychain
//! 4. Générer yaml + plist + wrapper
//! 5. Copier le callback Python dans le venv
//! 6. launchctl load
//! 7. Attendre health check OK
//!
//! Cette task implémente l'étape 1+2.

use std::io::{self, ErrorKind};
use std::process::Command;

pub fn ensure_uv_installed() -> io::Result<()> {
    let status = Command::new("uv").arg("--version").status();
    match status {
        Ok(s) if s.success() => {
            println!("✓ uv détecté");
            Ok(())
        }
        _ => Err(io::Error::new(
            ErrorKind::NotFound,
            "uv introuvable. Installe-le : `brew install uv`",
        )),
    }
}

pub fn install_litellm_via_uv() -> io::Result<()> {
    println!("→ uv tool install litellm[proxy] (peut prendre ~30s)…");
    let status = Command::new("uv")
        .args(["tool", "install", "--force", "litellm[proxy]"])
        .status()?;
    if !status.success() {
        return Err(io::Error::new(
            ErrorKind::Other,
            "uv tool install litellm a échoué",
        ));
    }
    println!("✓ litellm installé");
    Ok(())
}

/// Retourne le path absolu du binaire `litellm` installé par uv.
pub fn litellm_bin_path() -> io::Result<std::path::PathBuf> {
    let output = Command::new("uv").args(["tool", "dir"]).output()?;
    if !output.status.success() {
        return Err(io::Error::new(
            ErrorKind::Other,
            "uv tool dir a échoué",
        ));
    }
    let dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let bin = std::path::PathBuf::from(dir).join("litellm/bin/litellm");
    if !bin.exists() {
        return Err(io::Error::new(
            ErrorKind::NotFound,
            format!("litellm binary introuvable à {}", bin.display()),
        ));
    }
    Ok(bin)
}
