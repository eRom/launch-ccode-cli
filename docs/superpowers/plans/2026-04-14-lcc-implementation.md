# lcc — Launch Claude Code CLI — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust CLI wrapper (`lcc`) that launches `claude` with des profils de modèles configurables (local/cloud).

**Architecture:** CLI clap avec 3 commandes (`start`, `list`, `settings`). Config JSON chargée depuis `~/.config/launch-claude-code/settings.json`. Le runner résout le binaire claude, injecte les env vars du profil, et exec.

**Tech Stack:** Rust, clap (derive), serde, serde_json, dirs

---

## File Structure

```
Cargo.toml              — projet Rust, dépendances
src/
  main.rs               — point d'entrée, CLI clap, dispatch commandes
  config.rs             — types serde (Settings, Profile), chargement, validation
  runner.rs             — résolution binaire claude, construction env, exec
tests/
  config_test.rs        — tests unitaires config
  runner_test.rs        — tests unitaires runner
```

---

### Task 1: Scaffold du projet Rust

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`

- [ ] **Step 1: Init le projet Cargo**

```bash
cd /Users/recarnot/dev/launch-ccode-cli
cargo init --name lcc
```

- [ ] **Step 2: Ajouter les dépendances dans Cargo.toml**

Modifier `Cargo.toml` pour ajouter :

```toml
[package]
name = "lcc"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
dirs = "6"
```

- [ ] **Step 3: Écrire le squelette CLI dans main.rs**

```rust
use clap::{Parser, Subcommand};

mod config;
mod runner;

#[derive(Parser)]
#[command(name = "lcc", about = "Launch Claude Code with custom profiles")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch claude with a profile
    Start {
        /// Profile name from settings.json
        #[arg(long)]
        profil: String,
        /// Extra arguments passed to claude
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// List available profiles
    List,
    /// Open or validate settings
    Settings {
        /// Validate the settings.json structure
        #[arg(long)]
        validate: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Start { profil, args } => {
            if let Err(e) = runner::run(&profil, &args) {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        Commands::List => {
            if let Err(e) = config::list_profiles() {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        Commands::Settings { validate } => {
            if let Err(e) = config::settings_command(validate) {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
    }
}
```

- [ ] **Step 4: Créer les fichiers modules vides**

`src/config.rs` :
```rust
pub fn list_profiles() -> Result<(), Box<dyn std::error::Error>> {
    todo!()
}

pub fn settings_command(_validate: bool) -> Result<(), Box<dyn std::error::Error>> {
    todo!()
}
```

`src/runner.rs` :
```rust
pub fn run(_profil: &str, _args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    todo!()
}
```

- [ ] **Step 5: Vérifier que ça compile**

```bash
cargo build
```

Expected: BUILD SUCCESS

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml src/
git commit -m "feat: scaffold projet Rust avec CLI clap"
```

---

### Task 2: Module config — types et chargement

**Files:**
- Create: `tests/config_test.rs`
- Modify: `src/config.rs`

- [ ] **Step 1: Écrire les tests pour le parsing de config**

`tests/config_test.rs` :
```rust
use std::io::Write;
use tempfile::NamedTempFile;

// On teste le parsing directement via serde
#[test]
fn test_parse_valid_settings() {
    let json = r#"{
        "profiles": {
            "gemma4": {
                "model": "gemma4",
                "base_url": "http://localhost:11434/v1",
                "api_key": "",
                "auth_token": "ollama"
            }
        }
    }"#;

    let settings: lcc::config::Settings = serde_json::from_str(json).unwrap();
    assert_eq!(settings.profiles.len(), 1);
    let profile = settings.profiles.get("gemma4").unwrap();
    assert_eq!(profile.model, "gemma4");
    assert_eq!(profile.base_url, "http://localhost:11434/v1");
    assert_eq!(profile.api_key, "");
    assert_eq!(profile.auth_token, "ollama");
    assert!(profile.env.is_none());
}

#[test]
fn test_parse_profile_with_env() {
    let json = r#"{
        "profiles": {
            "test": {
                "model": "test-model",
                "base_url": "https://api.example.com/v1",
                "api_key": "sk-xxx",
                "auth_token": "bearer",
                "env": {
                    "CLAUDE_CODE_AUTO_COMPACT_WINDOW": "50000"
                }
            }
        }
    }"#;

    let settings: lcc::config::Settings = serde_json::from_str(json).unwrap();
    let profile = settings.profiles.get("test").unwrap();
    let env = profile.env.as_ref().unwrap();
    assert_eq!(env.get("CLAUDE_CODE_AUTO_COMPACT_WINDOW").unwrap(), "50000");
}

#[test]
fn test_parse_missing_required_field() {
    let json = r#"{
        "profiles": {
            "bad": {
                "model": "test"
            }
        }
    }"#;

    let result: Result<lcc::config::Settings, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
fn test_get_profile_not_found() {
    let json = r#"{"profiles": {}}"#;
    let settings: lcc::config::Settings = serde_json::from_str(json).unwrap();
    assert!(settings.profiles.get("nope").is_none());
}
```

- [ ] **Step 2: Ajouter tempfile aux dev-dependencies**

Dans `Cargo.toml` :
```toml
[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 3: Rendre le module config public (lib.rs)**

Créer `src/lib.rs` :
```rust
pub mod config;
pub mod runner;
```

- [ ] **Step 4: Vérifier que les tests échouent**

```bash
cargo test --test config_test
```

Expected: FAIL — types pas définis

- [ ] **Step 5: Implémenter les types et le chargement dans config.rs**

`src/config.rs` :
```rust
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub profiles: HashMap<String, Profile>,
}

#[derive(Debug, Deserialize)]
pub struct Profile {
    pub model: String,
    pub base_url: String,
    pub api_key: String,
    pub auth_token: String,
    pub env: Option<HashMap<String, String>>,
}

pub fn settings_path() -> PathBuf {
    let config_dir = dirs::config_dir().expect("cannot resolve config directory");
    config_dir.join("launch-claude-code").join("settings.json")
}

pub fn load_settings() -> Result<Settings, Box<dyn std::error::Error>> {
    let path = settings_path();
    if !path.exists() {
        return Err(format!(
            "Settings file not found: {}\nCreate it with your profiles.",
            path.display()
        )
        .into());
    }
    let content = std::fs::read_to_string(&path)?;
    let settings: Settings = serde_json::from_str(&content)?;
    Ok(settings)
}

pub fn list_profiles() -> Result<(), Box<dyn std::error::Error>> {
    let settings = load_settings()?;
    if settings.profiles.is_empty() {
        println!("No profiles configured.");
        return Ok(());
    }
    println!("{:<20} {:<30} {}", "PROFILE", "MODEL", "BASE URL");
    println!("{:<20} {:<30} {}", "-------", "-----", "--------");
    let mut names: Vec<_> = settings.profiles.keys().collect();
    names.sort();
    for name in names {
        let p = &settings.profiles[name];
        println!("{:<20} {:<30} {}", name, p.model, p.base_url);
    }
    Ok(())
}

pub fn settings_command(validate: bool) -> Result<(), Box<dyn std::error::Error>> {
    let path = settings_path();
    if validate {
        match load_settings() {
            Ok(settings) => {
                println!("OK — {} profile(s) found.", settings.profiles.len());
                Ok(())
            }
            Err(e) => Err(format!("Validation failed: {e}").into()),
        }
    } else {
        if !path.exists() {
            return Err(format!("Settings file not found: {}", path.display()).into());
        }
        std::process::Command::new("open")
            .arg(&path)
            .status()?;
        Ok(())
    }
}
```

- [ ] **Step 6: Vérifier que les tests passent**

```bash
cargo test --test config_test
```

Expected: ALL PASS

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml src/lib.rs src/config.rs tests/config_test.rs
git commit -m "feat: module config — types serde, chargement et validation"
```

---

### Task 3: Module runner — résolution claude et exec

**Files:**
- Create: `tests/runner_test.rs`
- Modify: `src/runner.rs`

- [ ] **Step 1: Écrire les tests pour le runner**

`tests/runner_test.rs` :
```rust
#[test]
fn test_build_env_vars_basic() {
    let profile = lcc::config::Profile {
        model: "gemma4".to_string(),
        base_url: "http://localhost:11434/v1".to_string(),
        api_key: "".to_string(),
        auth_token: "ollama".to_string(),
        env: None,
    };

    let env = lcc::runner::build_env_vars(&profile);

    assert_eq!(env.get("ANTHROPIC_BASE_URL").unwrap(), "http://localhost:11434/v1");
    assert_eq!(env.get("ANTHROPIC_API_KEY").unwrap(), "");
    assert_eq!(env.get("ANTHROPIC_AUTH_TOKEN").unwrap(), "ollama");
    assert_eq!(env.get("ANTHROPIC_DEFAULT_OPUS_MODEL").unwrap(), "gemma4");
    assert_eq!(env.get("ANTHROPIC_DEFAULT_SONNET_MODEL").unwrap(), "gemma4");
    assert_eq!(env.get("ANTHROPIC_DEFAULT_HAIKU_MODEL").unwrap(), "gemma4");
    assert_eq!(env.get("CLAUDE_CODE_SUBAGENT_MODEL").unwrap(), "gemma4");
    assert_eq!(env.get("CLAUDE_CODE_ATTRIBUTION_HEADER").unwrap(), "0");
}

#[test]
fn test_build_env_vars_with_custom_env() {
    let mut custom = std::collections::HashMap::new();
    custom.insert("MY_VAR".to_string(), "hello".to_string());

    let profile = lcc::config::Profile {
        model: "test".to_string(),
        base_url: "https://api.example.com/v1".to_string(),
        api_key: "sk-xxx".to_string(),
        auth_token: "bearer".to_string(),
        env: Some(custom),
    };

    let env = lcc::runner::build_env_vars(&profile);
    assert_eq!(env.get("MY_VAR").unwrap(), "hello");
    assert_eq!(env.get("ANTHROPIC_API_KEY").unwrap(), "sk-xxx");
}

#[test]
fn test_build_claude_args() {
    let extra = vec!["--dangerously-skip-permissions".to_string(), "-p".to_string(), "hello".to_string()];
    let args = lcc::runner::build_claude_args("gemma4", &extra);
    assert_eq!(args, vec!["--model", "gemma4", "--dangerously-skip-permissions", "-p", "hello"]);
}

#[test]
fn test_build_claude_args_no_extra() {
    let args = lcc::runner::build_claude_args("gemma4", &[]);
    assert_eq!(args, vec!["--model", "gemma4"]);
}
```

- [ ] **Step 2: Vérifier que les tests échouent**

```bash
cargo test --test runner_test
```

Expected: FAIL — fonctions pas définies

- [ ] **Step 3: Implémenter le runner**

`src/runner.rs` :
```rust
use crate::config::{load_settings, Profile};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

pub fn find_claude() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Check PATH first
    if let Ok(output) = Command::new("which").arg("claude").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Ok(PathBuf::from(path));
        }
    }

    // Fallback: ~/.claude/local/claude
    let home = dirs::home_dir().ok_or("cannot resolve home directory")?;
    let fallback = home.join(".claude").join("local").join("claude");
    if fallback.exists() {
        return Ok(fallback);
    }

    Err(format!(
        "claude not found in PATH or at {}",
        fallback.display()
    )
    .into())
}

pub fn build_env_vars(profile: &Profile) -> HashMap<String, String> {
    let mut env = HashMap::new();
    env.insert("ANTHROPIC_BASE_URL".into(), profile.base_url.clone());
    env.insert("ANTHROPIC_API_KEY".into(), profile.api_key.clone());
    env.insert("ANTHROPIC_AUTH_TOKEN".into(), profile.auth_token.clone());
    env.insert("ANTHROPIC_DEFAULT_OPUS_MODEL".into(), profile.model.clone());
    env.insert("ANTHROPIC_DEFAULT_SONNET_MODEL".into(), profile.model.clone());
    env.insert("ANTHROPIC_DEFAULT_HAIKU_MODEL".into(), profile.model.clone());
    env.insert("CLAUDE_CODE_SUBAGENT_MODEL".into(), profile.model.clone());
    env.insert("CLAUDE_CODE_ATTRIBUTION_HEADER".into(), "0".into());

    if let Some(custom) = &profile.env {
        for (k, v) in custom {
            env.insert(k.clone(), v.clone());
        }
    }

    env
}

pub fn build_claude_args(model: &str, extra: &[String]) -> Vec<String> {
    let mut args = vec!["--model".to_string(), model.to_string()];
    args.extend(extra.iter().cloned());
    args
}

pub fn run(profil: &str, extra_args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let settings = load_settings()?;
    let profile = settings.profiles.get(profil).ok_or_else(|| {
        let available: Vec<_> = settings.profiles.keys().collect();
        format!(
            "Profile '{}' not found. Available: {}",
            profil,
            if available.is_empty() {
                "none".to_string()
            } else {
                available.into_iter().cloned().collect::<Vec<_>>().join(", ")
            }
        )
    })?;

    let claude_path = find_claude()?;
    let env_vars = build_env_vars(profile);
    let args = build_claude_args(&profile.model, extra_args);

    let status = Command::new(&claude_path)
        .args(&args)
        .envs(&env_vars)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}
```

- [ ] **Step 4: Rendre Profile constructible dans les tests (pub fields + dérivation)**

Le struct `Profile` a déjà des champs `pub` — vérifier que ça compile.

- [ ] **Step 5: Vérifier que les tests passent**

```bash
cargo test --test runner_test
```

Expected: ALL PASS

- [ ] **Step 6: Vérifier que tous les tests passent**

```bash
cargo test
```

Expected: ALL PASS

- [ ] **Step 7: Commit**

```bash
git add src/runner.rs tests/runner_test.rs
git commit -m "feat: module runner — résolution claude, env vars, exec"
```

---

### Task 4: Intégration et build final

**Files:**
- Modify: `src/main.rs` (déjà fait en Task 1, vérifier cohérence)

- [ ] **Step 1: Vérifier le build release**

```bash
cargo build --release
```

Expected: BUILD SUCCESS

- [ ] **Step 2: Tester le binaire — help**

```bash
./target/release/lcc --help
```

Expected: affiche l'aide avec les 3 sous-commandes

- [ ] **Step 3: Tester le binaire — list (sans fichier settings)**

```bash
./target/release/lcc list
```

Expected: message d'erreur indiquant le chemin du fichier manquant

- [ ] **Step 4: Créer un settings.json de test**

```bash
mkdir -p ~/.config/launch-claude-code
cat > ~/.config/launch-claude-code/settings.json << 'EOF'
{
  "profiles": {
    "gemma4": {
      "model": "gemma4",
      "base_url": "http://localhost:11434/v1",
      "api_key": "",
      "auth_token": "ollama"
    },
    "openrouter-llama4": {
      "model": "meta-llama/llama-4-maverick",
      "base_url": "https://openrouter.ai/api/v1",
      "api_key": "sk-or-xxx",
      "auth_token": "openrouter",
      "env": {
        "CLAUDE_CODE_AUTO_COMPACT_WINDOW": "50000"
      }
    }
  }
}
EOF
```

- [ ] **Step 5: Tester list avec config**

```bash
./target/release/lcc list
```

Expected: tableau avec gemma4 et openrouter-llama4

- [ ] **Step 6: Tester settings --validate**

```bash
./target/release/lcc settings --validate
```

Expected: `OK — 2 profile(s) found.`

- [ ] **Step 7: Commit**

```bash
git commit --allow-empty -m "test: validation intégration CLI complète"
```

---

### Task 5: Installation du binaire

- [ ] **Step 1: Installer le binaire**

```bash
cargo install --path /Users/recarnot/dev/launch-ccode-cli
```

- [ ] **Step 2: Vérifier l'installation**

```bash
lcc --help
lcc list
lcc settings --validate
```

Expected: tout fonctionne depuis n'importe quel répertoire

- [ ] **Step 3: Commit final**

```bash
git add -A
git commit -m "docs: plan d'implémentation complet"
```
