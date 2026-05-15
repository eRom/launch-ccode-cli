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
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;

use crate::config::load_settings;
use crate::proxy::{
    generators::{plist::generate_plist, wrapper::generate_wrapper, yaml::generate_litellm_yaml},
    keychain, logs_dir, plist_path, wrapper_path, yaml_path, DEFAULT_PORT,
};

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

pub fn write_master_key_to_keychain() -> io::Result<()> {
    let key = keychain::generate_random_key();
    keychain::set_master_key(&key)?;
    println!("✓ master_key générée + stockée dans le Keychain");
    Ok(())
}

pub fn write_yaml() -> io::Result<()> {
    let settings = load_settings().map_err(|e| {
        io::Error::new(ErrorKind::Other, format!("settings.json invalide: {e}"))
    })?;
    let yaml = generate_litellm_yaml(&settings);
    let path = yaml_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, yaml)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    println!("✓ litellm.yaml généré : {}", path.display());
    Ok(())
}

pub fn write_wrapper() -> io::Result<()> {
    let bin = litellm_bin_path()?;
    let yaml = yaml_path();
    let wrapper_content = generate_wrapper(
        &bin.to_string_lossy(),
        &yaml.to_string_lossy(),
        DEFAULT_PORT,
    );
    let path = wrapper_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, wrapper_content)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
    println!("✓ wrapper généré : {}", path.display());
    Ok(())
}

pub fn write_plist() -> io::Result<()> {
    // Snapshot des env vars provider connues depuis le shell courant.
    let env_keys = [
        "OPENROUTER_API_KEY",
        "DEEPSEEK_API_KEY",
        "GROQ_API_KEY",
        "TOGETHER_API_KEY",
        "MISTRAL_API_KEY",
    ];
    let mut env: HashMap<String, String> = HashMap::new();
    for k in env_keys {
        if let Ok(v) = std::env::var(k) {
            env.insert(k.to_string(), v);
        }
    }
    if env.is_empty() {
        eprintln!("⚠ aucune clé provider trouvée dans le shell — vérifie ton .zshrc");
    }

    let logs = logs_dir();
    fs::create_dir_all(&logs)?;
    let stdout = logs.join("litellm.out.log");
    let stderr = logs.join("litellm.err.log");

    let plist_content = generate_plist(
        &wrapper_path().to_string_lossy(),
        &stdout.to_string_lossy(),
        &stderr.to_string_lossy(),
        &env,
    );
    let path = plist_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, plist_content)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    println!("✓ plist généré : {}", path.display());
    Ok(())
}
