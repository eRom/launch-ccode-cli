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
use std::thread;
use std::time::{Duration, Instant};

use crate::config::load_settings;
use crate::proxy::{
    generators::{plist::generate_plist, wrapper::generate_wrapper, yaml::generate_litellm_yaml},
    keychain, health, launchctl, logs_dir, plist_path, wrapper_path, yaml_path, DEFAULT_PORT,
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
    println!("→ uv tool install litellm[proxy] + prisma (peut prendre ~30s)…");
    // `--with prisma` : LiteLLM appelle `import prisma` dans son handler
    // d'exception meme sans DB configuree. Sans prisma installe, le daemon
    // crashe au startup avec ModuleNotFoundError.
    let status = Command::new("uv")
        .args([
            "tool", "install", "--force",
            "--with", "prisma",
            "litellm[proxy]",
        ])
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

/// Copie le callback Python embarqué dans les assets vers le répertoire du yaml config.
/// LiteLLM résout `callbacks: ["foo.bar"]` comme `<config_dir>/foo.py` → le fichier
/// doit être à côté du litellm.yaml, pas dans site-packages.
pub fn install_python_callback() -> io::Result<()> {
    const CALLBACK_PY: &str = include_str!("../../assets/lcc_strip_thinking.py");

    let yaml = yaml_path();
    let dest_dir = yaml.parent().ok_or_else(|| {
        io::Error::new(ErrorKind::Other, "yaml_path n'a pas de parent")
    })?;
    fs::create_dir_all(dest_dir)?;
    let dest = dest_dir.join("lcc_strip_thinking.py");
    fs::write(&dest, CALLBACK_PY)?;
    println!("✓ callback Python installé : {}", dest.display());
    Ok(())
}

/// Lance `prisma generate` dans le venv litellm pour creer le client prisma.
/// LiteLLM appelle `from prisma import Prisma` dans son startup event meme
/// sans DB configuree (cf litellm/proxy/utils.py:2562). Sans ce generate,
/// le daemon crashe avec "Unable to find Prisma binaries".
pub fn run_prisma_generate() -> io::Result<()> {
    println!("→ prisma generate…");
    let tool_dir = uv_tool_dir_path()?;
    let schema = tool_dir
        .join("lib/python3.13/site-packages/litellm/proxy/schema.prisma");
    if !schema.exists() {
        return Err(io::Error::new(
            ErrorKind::NotFound,
            format!("schema.prisma introuvable a {}", schema.display()),
        ));
    }
    let prisma_bin = tool_dir.join("bin/prisma");
    let bin_dir = tool_dir.join("bin");
    let new_path = match std::env::var("PATH") {
        Ok(p) => format!("{}:{p}", bin_dir.display()),
        Err(_) => bin_dir.display().to_string(),
    };
    let status = Command::new(&prisma_bin)
        .args(["generate", "--schema"])
        .arg(&schema)
        .env("PATH", &new_path)
        .current_dir(schema.parent().unwrap())
        .status()?;
    if !status.success() {
        return Err(io::Error::new(
            ErrorKind::Other,
            "prisma generate a échoué",
        ));
    }
    println!("✓ prisma client généré");
    Ok(())
}

/// Place une sentinelle `.env` dans le tool dir uv pour court-circuiter la
/// remontee de python-dotenv. Sans ca, dotenv (appele par LiteLLM) remonte
/// depuis le `__file__` du module litellm jusqu'a `~/.env` et y trouve
/// potentiellement DATABASE_URL, ce qui fait crasher LiteLLM au demarrage.
pub fn install_dotenv_sentinel() -> io::Result<()> {
    let tool_dir = uv_tool_dir_path()?;
    let sentinel = tool_dir.join(".env");
    let content = "# Sentinelle placee par lcc proxy install: court-circuite la remontee\n\
                   # de python-dotenv pour empecher le chargement de ~/.env (qui pourrait\n\
                   # contenir DATABASE_URL et faire crasher LiteLLM au demarrage).\n\
                   LCC_DOTENV_SENTINEL=1\n";
    fs::write(&sentinel, content)?;
    println!("✓ sentinelle dotenv installée : {}", sentinel.display());
    Ok(())
}

fn uv_tool_dir_path() -> io::Result<std::path::PathBuf> {
    let output = Command::new("uv").args(["tool", "dir"]).output()?;
    if !output.status.success() {
        return Err(io::Error::new(ErrorKind::Other, "uv tool dir a échoué"));
    }
    let dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(std::path::PathBuf::from(dir).join("litellm"))
}

/// Charge le LaunchAgent et attend que le proxy soit opérationnel.
pub fn load_and_wait() -> io::Result<()> {
    println!("→ launchctl load…");
    launchctl::load()?;
    println!("→ attente health check (max 10s)…");
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if health::is_alive(DEFAULT_PORT, Duration::from_millis(500)) {
            println!("✓ proxy ready on :{DEFAULT_PORT}");
            return Ok(());
        }
        thread::sleep(Duration::from_millis(500));
    }
    Err(io::Error::new(
        ErrorKind::TimedOut,
        format!(
            "proxy non répondant après 10s. Vérifie : tail ~/Library/Logs/lcc/litellm.err.log"
        ),
    ))
}

/// Orchestrateur principal : enchaîne toutes les étapes d'installation.
pub fn run_install() -> io::Result<()> {
    println!("=== lcc proxy install ===\n");
    ensure_uv_installed()?;
    install_litellm_via_uv()?;
    run_prisma_generate()?;
    install_dotenv_sentinel()?;
    write_master_key_to_keychain()?;
    write_yaml()?;
    write_wrapper()?;
    write_plist()?;
    install_python_callback()?;
    load_and_wait()?;
    println!("\n✓ tout est prêt. `lcc start --profil <X>` devrait marcher.");
    Ok(())
}
