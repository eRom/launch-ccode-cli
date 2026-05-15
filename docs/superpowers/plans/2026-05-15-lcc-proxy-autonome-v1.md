# lcc Proxy Autonome V1 — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Render `lcc start --profil X -p "..."` fonctionnel end-to-end avec un proxy LiteLLM autonome géré par launchd, qui strip les `thinking` blocks problématiques (Qwen, DeepSeek, Gemma) avant qu'ils n'atteignent `claude`.

**Architecture:** Tous les appels `lcc` passent par `http://localhost:4000` (LiteLLM Python en daemon launchd). `lcc proxy install` setup tout (uv tool install, venv, plist, wrapper script, master key keychain). `lcc start` health-check le daemon puis spawn `claude` avec `ANTHROPIC_BASE_URL=http://localhost:4000`. Le binaire `claude` natif (sans lcc) garde son path direct vers Anthropic.

**Tech Stack:** Rust 2024 edition (sync, pas de tokio), `serde_yaml` pour génération yaml, `ureq` pour HTTP sync, `security` CLI macOS pour Keychain, `launchctl` pour daemon, LiteLLM Python ≥1.50 via `uv tool install`.

**Spec source:** `docs/superpowers/specs/2026-05-15-lcc-proxy-autonome-design.md`

---

## File Structure

**Nouveaux fichiers Rust :**
```
src/proxy/
  mod.rs                       # pub re-exports + paths constants centralisés
  cli.rs                       # enum Proxy + dispatch des sous-commandes
  provider_map.rs              # base_url → préfixe LiteLLM (table)
  generators/
    mod.rs                     # re-exports
    yaml.rs                    # settings.json → litellm.yaml
    plist.rs                   # struct → plist XML
    wrapper.rs                 # bash wrapper script
  keychain.rs                  # wrapper sync sur `security` CLI
  launchctl.rs                 # wrapper sync sur `launchctl`
  health.rs                    # GET /health/liveness avec timeout
  install.rs                   # orchestration `proxy install`
  uninstall.rs                 # orchestration `proxy uninstall`
  reload.rs                    # regen yaml + restart daemon
  lifecycle.rs                 # start/stop/restart wrappers
  status.rs                    # status / logs / doctor (basique V1)
```

**Modifications :**
```
Cargo.toml                     # + serde_yaml, ureq, dirs déjà présent
src/main.rs                    # + subcommand Proxy
src/runner.rs                  # + health check pre-spawn + env override
```

**Assets versionnés :**
```
assets/
  lcc_strip_thinking.py        # callback Python LiteLLM
  tests/
    test_strip_thinking.py     # pytest pour le callback
```

**Tests d'intégration Rust :**
```
tests/
  proxy_yaml_test.rs           # snapshot tests yaml gen
  proxy_plist_test.rs          # snapshot tests plist gen
  proxy_wrapper_test.rs        # snapshot tests wrapper gen
  proxy_provider_map_test.rs   # cas par provider
  proxy_health_test.rs         # mock HTTP server
  runner_test.rs               # MODIFY: + health check pre-spawn
```

---

## Phase 0 — Setup

### Task 0.1: Dependencies + module skeleton

**Files:**
- Modify: `Cargo.toml`
- Create: `src/proxy/mod.rs`
- Create: `src/proxy/generators/mod.rs`
- Modify: `src/main.rs:1-10` (add `mod proxy;`)

- [ ] **Step 1: Add deps to Cargo.toml**

Edit `[dependencies]` section, add :
```toml
serde_yaml = "0.9"
ureq = { version = "2.10", features = ["json"] }
```

- [ ] **Step 2: Create `src/proxy/mod.rs` skeleton**

```rust
//! Proxy LiteLLM autonome — install, lifecycle, génération de config.
//!
//! Voir docs/superpowers/specs/2026-05-15-lcc-proxy-autonome-design.md

pub mod cli;
pub mod generators;
pub mod health;
pub mod install;
pub mod keychain;
pub mod launchctl;
pub mod lifecycle;
pub mod provider_map;
pub mod reload;
pub mod status;
pub mod uninstall;

use std::path::PathBuf;

/// Identifiant launchd du daemon.
pub const LAUNCHD_LABEL: &str = "com.lcc.litellm";

/// Slug Keychain pour la master_key.
pub const KEYCHAIN_SERVICE: &str = "lcc.litellm.master_key";

/// Port par défaut du proxy.
pub const DEFAULT_PORT: u16 = 4000;

/// Path du yaml généré.
pub fn yaml_path() -> PathBuf {
    dirs::config_dir()
        .expect("config_dir introuvable")
        .join("launch-claude-code")
        .join("litellm.yaml")
}

/// Path du plist LaunchAgent.
pub fn plist_path() -> PathBuf {
    dirs::home_dir()
        .expect("home_dir introuvable")
        .join("Library/LaunchAgents")
        .join(format!("{LAUNCHD_LABEL}.plist"))
}

/// Path du wrapper bash.
pub fn wrapper_path() -> PathBuf {
    dirs::data_local_dir()
        .expect("data_local_dir introuvable")
        .join("lcc")
        .join("lcc-litellm-launcher.sh")
}

/// Dossier de logs.
pub fn logs_dir() -> PathBuf {
    dirs::home_dir()
        .expect("home_dir introuvable")
        .join("Library/Logs/lcc")
}
```

- [ ] **Step 3: Create `src/proxy/generators/mod.rs`**

```rust
pub mod plist;
pub mod wrapper;
pub mod yaml;
```

- [ ] **Step 4: Wire module into main.rs**

Add `mod proxy;` near the top of `src/main.rs` (after existing `mod` declarations, line 1-10 area).

- [ ] **Step 5: Stub all referenced sub-modules**

Create empty stubs (`pub fn placeholder() {}`) for each module declared in `mod.rs` so the project compiles :
- `src/proxy/cli.rs`
- `src/proxy/health.rs`
- `src/proxy/install.rs`
- `src/proxy/keychain.rs`
- `src/proxy/launchctl.rs`
- `src/proxy/lifecycle.rs`
- `src/proxy/provider_map.rs`
- `src/proxy/reload.rs`
- `src/proxy/status.rs`
- `src/proxy/uninstall.rs`
- `src/proxy/generators/plist.rs`
- `src/proxy/generators/wrapper.rs`
- `src/proxy/generators/yaml.rs`

Each file contains just :
```rust
//! TODO: implementation (cf. plan task X)
```

- [ ] **Step 6: Compile + commit**

```bash
cargo build
# Expected: success, warnings about unused modules (OK)

git add Cargo.toml src/main.rs src/proxy/
git commit -m "chore(proxy): module skeleton + dependencies"
```

---

## Phase 1 — Pure logic (TDD)

### Task 1.1: provider_map

**Files:**
- Modify: `src/proxy/provider_map.rs`
- Create: `tests/proxy_provider_map_test.rs`

- [ ] **Step 1: Write failing tests**

Create `tests/proxy_provider_map_test.rs` :
```rust
use launch_claude_code::proxy::provider_map::detect_litellm_prefix;

#[test]
fn openrouter_detected() {
    assert_eq!(
        detect_litellm_prefix("https://openrouter.ai/api/v1"),
        Some("openrouter")
    );
}

#[test]
fn deepseek_detected() {
    assert_eq!(
        detect_litellm_prefix("https://api.deepseek.com/v1"),
        Some("deepseek")
    );
}

#[test]
fn groq_detected() {
    assert_eq!(
        detect_litellm_prefix("https://api.groq.com/openai/v1"),
        Some("groq")
    );
}

#[test]
fn together_detected() {
    assert_eq!(
        detect_litellm_prefix("https://api.together.xyz/v1"),
        Some("together_ai")
    );
}

#[test]
fn mistral_detected() {
    assert_eq!(
        detect_litellm_prefix("https://api.mistral.ai/v1"),
        Some("mistral")
    );
}

#[test]
fn unknown_returns_none() {
    assert_eq!(
        detect_litellm_prefix("https://example.com/v1"),
        None
    );
}

#[test]
fn http_scheme_works() {
    assert_eq!(
        detect_litellm_prefix("http://openrouter.ai/api/v1"),
        Some("openrouter")
    );
}
```

**Note:** The tests reference `launch_claude_code` — verify the actual crate name in `Cargo.toml`. If the binary crate is `lcc`, you may need to expose a lib via `src/lib.rs` or restructure tests as integration tests inside `src/`. Quick-check before writing code: read `Cargo.toml` for `[package] name = ...` and adjust the import path accordingly. If only `[[bin]]` exists, add a `[lib]` section + `src/lib.rs` re-exporting modules.

- [ ] **Step 2: Run tests to verify failure**

```bash
cargo test --test proxy_provider_map_test
```
Expected: compile errors (function not implemented).

- [ ] **Step 3: Implement provider_map**

```rust
// src/proxy/provider_map.rs
//! Détection du préfixe LiteLLM à partir d'un base_url provider.
//!
//! LiteLLM identifie chaque provider par un préfixe sur le model id
//! (ex: `openrouter/qwen/qwen3.6-plus`). On déduit ce préfixe à partir
//! du `base_url` du profil lcc.

const PROVIDER_MAP: &[(&str, &str)] = &[
    ("openrouter.ai",     "openrouter"),
    ("api.deepseek.com",  "deepseek"),
    ("api.groq.com",      "groq"),
    ("api.together.xyz",  "together_ai"),
    ("api.mistral.ai",    "mistral"),
    // Ajout : étendre cette table au fur et à mesure des besoins.
];

/// Retourne le préfixe LiteLLM correspondant au host du `base_url`,
/// ou `None` si non reconnu (le caller fallbackra sur `openai/`).
pub fn detect_litellm_prefix(base_url: &str) -> Option<&'static str> {
    let url = url_strip_scheme(base_url);
    for (host, prefix) in PROVIDER_MAP {
        if url.starts_with(host) {
            return Some(prefix);
        }
    }
    None
}

fn url_strip_scheme(url: &str) -> &str {
    url.trim_start_matches("https://")
        .trim_start_matches("http://")
}
```

- [ ] **Step 4: Run tests, verify pass**

```bash
cargo test --test proxy_provider_map_test
```
Expected: all 7 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/proxy/provider_map.rs tests/proxy_provider_map_test.rs Cargo.toml src/lib.rs
git commit -m "feat(proxy): provider map base_url -> litellm prefix"
```

---

### Task 1.2: yaml generator (single profile)

**Files:**
- Modify: `src/proxy/generators/yaml.rs`
- Create: `tests/proxy_yaml_test.rs`

- [ ] **Step 1: Write failing test for single profile**

```rust
// tests/proxy_yaml_test.rs
use launch_claude_code::config::{Profile, Settings, SingleProfile};
use launch_claude_code::proxy::generators::yaml::generate_litellm_yaml;
use std::collections::HashMap;

fn single_profile(model: &str, base_url: &str, auth_token: Option<&str>) -> Profile {
    Profile::Single(SingleProfile {
        model: model.to_string(),
        base_url: base_url.to_string(),
        api_key: None,
        auth_token: auth_token.map(String::from),
        env: None,
    })
}

#[test]
fn single_openrouter_profile_generates_correct_yaml() {
    let mut profiles = HashMap::new();
    profiles.insert(
        "qwen".to_string(),
        single_profile(
            "qwen/qwen3.6-plus",
            "https://openrouter.ai/api/v1",
            Some("${OPENROUTER_API_KEY}"),
        ),
    );
    let settings = Settings { profiles };

    let yaml = generate_litellm_yaml(&settings);

    let expected = r#"model_list:
- model_name: qwen
  litellm_params:
    model: openrouter/qwen/qwen3.6-plus
    api_key: os.environ/OPENROUTER_API_KEY
    drop_params: true
litellm_settings:
  drop_params: true
  set_verbose: false
  callbacks:
  - lcc_strip_thinking
general_settings:
  master_key: os.environ/LCC_MASTER_KEY
  database_url: null
  store_model_in_db: false
"#;
    assert_eq!(yaml, expected);
}
```

- [ ] **Step 2: Verify failure**

```bash
cargo test --test proxy_yaml_test single_openrouter_profile_generates_correct_yaml
```
Expected: compile error (function missing).

- [ ] **Step 3: Implement minimal `generate_litellm_yaml` for single profile**

```rust
// src/proxy/generators/yaml.rs
//! Génère le `litellm.yaml` à partir des profils `lcc`.

use crate::config::{Profile, Settings};
use crate::proxy::provider_map::detect_litellm_prefix;
use serde_yaml::{Mapping, Sequence, Value};

/// Génère le contenu YAML à écrire dans `~/.config/launch-claude-code/litellm.yaml`.
pub fn generate_litellm_yaml(settings: &Settings) -> String {
    let mut model_list = Sequence::new();

    for (profile_name, profile) in &settings.profiles {
        match profile {
            Profile::Single(sp) => {
                model_list.push(build_model_entry(
                    profile_name,
                    &sp.model,
                    &sp.base_url,
                    sp.auth_token.as_deref().or(sp.api_key.as_deref()),
                ));
            }
            Profile::Multi(_mp) => {
                // Implementé en Task 1.3
            }
        }
    }

    let mut root = Mapping::new();
    root.insert(Value::from("model_list"), Value::Sequence(model_list));
    root.insert(Value::from("litellm_settings"), litellm_settings_block());
    root.insert(Value::from("general_settings"), general_settings_block());

    serde_yaml::to_string(&Value::Mapping(root))
        .expect("serialization yaml LiteLLM ne peut pas échouer")
}

fn build_model_entry(
    name: &str,
    model_id: &str,
    base_url: &str,
    auth: Option<&str>,
) -> Value {
    let prefix = detect_litellm_prefix(base_url).unwrap_or("openai");
    let full_model = format!("{prefix}/{model_id}");

    let mut params = Mapping::new();
    params.insert(Value::from("model"), Value::from(full_model));
    if let Some(env_ref) = auth.and_then(extract_env_var) {
        params.insert(
            Value::from("api_key"),
            Value::from(format!("os.environ/{env_ref}")),
        );
    }
    params.insert(Value::from("drop_params"), Value::from(true));

    let mut entry = Mapping::new();
    entry.insert(Value::from("model_name"), Value::from(name));
    entry.insert(Value::from("litellm_params"), Value::Mapping(params));
    Value::Mapping(entry)
}

/// Extrait `OPENROUTER_API_KEY` depuis `${OPENROUTER_API_KEY}`. Retourne None si pas en format ${VAR}.
fn extract_env_var(s: &str) -> Option<String> {
    if s.starts_with("${") && s.ends_with('}') {
        Some(s[2..s.len() - 1].to_string())
    } else {
        None
    }
}

fn litellm_settings_block() -> Value {
    let mut m = Mapping::new();
    m.insert(Value::from("drop_params"), Value::from(true));
    m.insert(Value::from("set_verbose"), Value::from(false));
    m.insert(
        Value::from("callbacks"),
        Value::Sequence(vec![Value::from("lcc_strip_thinking")]),
    );
    Value::Mapping(m)
}

fn general_settings_block() -> Value {
    let mut m = Mapping::new();
    m.insert(
        Value::from("master_key"),
        Value::from("os.environ/LCC_MASTER_KEY"),
    );
    m.insert(Value::from("database_url"), Value::Null);
    m.insert(Value::from("store_model_in_db"), Value::from(false));
    Value::Mapping(m)
}
```

- [ ] **Step 4: Run, verify pass**

```bash
cargo test --test proxy_yaml_test single_openrouter_profile_generates_correct_yaml
```
Expected: PASS.

**If the test fails on key ordering** : `serde_yaml::Mapping` preserves insertion order, but the assertion is byte-exact. If keys come out in a different order, switch to `assert_yaml_eq` (parse both sides) using `serde_yaml::from_str::<Value>` and compare.

- [ ] **Step 5: Commit**

```bash
git add src/proxy/generators/yaml.rs tests/proxy_yaml_test.rs
git commit -m "feat(proxy): yaml generator for single profile"
```

---

### Task 1.3: yaml generator (multi profile)

**Files:**
- Modify: `src/proxy/generators/yaml.rs`
- Modify: `tests/proxy_yaml_test.rs`

- [ ] **Step 1: Add failing test for multi profile**

Append to `tests/proxy_yaml_test.rs` :
```rust
use launch_claude_code::config::{ModelEntry, ModelSlot, MultiProfile};

#[test]
fn multi_profile_expands_each_model() {
    let mut models = HashMap::new();
    models.insert(
        "fast".to_string(),
        ModelEntry {
            id: "qwen/qwen3.6-flash".to_string(),
            slot: ModelSlot::Haiku,
            description: Some("modèle rapide".to_string()),
        },
    );
    models.insert(
        "smart".to_string(),
        ModelEntry {
            id: "qwen/qwen3.6-plus".to_string(),
            slot: ModelSlot::Sonnet,
            description: None,
        },
    );

    let mp = MultiProfile {
        base_url: "https://openrouter.ai/api/v1".to_string(),
        api_key: None,
        auth_token: Some("${OPENROUTER_API_KEY}".to_string()),
        default: "smart".to_string(),
        models,
        env: None,
    };

    let mut profiles = HashMap::new();
    profiles.insert("openrouter-multi".to_string(), Profile::Multi(mp));
    let settings = Settings { profiles };

    let yaml = generate_litellm_yaml(&settings);
    // Two model_list entries expected, one per ModelEntry, with stable naming
    // pattern "<profile>/<model_alias>"
    assert!(yaml.contains("model_name: openrouter-multi/fast"));
    assert!(yaml.contains("model: openrouter/qwen/qwen3.6-flash"));
    assert!(yaml.contains("model_name: openrouter-multi/smart"));
    assert!(yaml.contains("model: openrouter/qwen/qwen3.6-plus"));
    assert!(yaml.contains("api_key: os.environ/OPENROUTER_API_KEY"));
}
```

- [ ] **Step 2: Verify failure**

```bash
cargo test --test proxy_yaml_test multi_profile_expands_each_model
```
Expected: FAIL (no entries generated for Multi).

- [ ] **Step 3: Implement Multi branch**

In `src/proxy/generators/yaml.rs`, replace the `Profile::Multi(_mp)` arm in `generate_litellm_yaml` with :

```rust
Profile::Multi(mp) => {
    let auth = mp.auth_token.as_deref().or(mp.api_key.as_deref());
    // Sort keys for deterministic output (HashMap iteration is random)
    let mut keys: Vec<&String> = mp.models.keys().collect();
    keys.sort();
    for alias in keys {
        let entry = &mp.models[alias];
        let composite_name = format!("{profile_name}/{alias}");
        model_list.push(build_model_entry(
            &composite_name,
            &entry.id,
            &mp.base_url,
            auth,
        ));
    }
}
```

- [ ] **Step 4: Run, verify pass**

```bash
cargo test --test proxy_yaml_test
```
Expected: both single + multi tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/proxy/generators/yaml.rs tests/proxy_yaml_test.rs
git commit -m "feat(proxy): yaml generator for multi profile (1 entry per model)"
```

---

### Task 1.4: plist + wrapper generators

**Files:**
- Modify: `src/proxy/generators/plist.rs`
- Modify: `src/proxy/generators/wrapper.rs`
- Create: `tests/proxy_plist_test.rs`
- Create: `tests/proxy_wrapper_test.rs`

- [ ] **Step 1: Write failing plist test**

```rust
// tests/proxy_plist_test.rs
use launch_claude_code::proxy::generators::plist::generate_plist;
use std::collections::HashMap;

#[test]
fn plist_contains_required_keys() {
    let mut env = HashMap::new();
    env.insert("OPENROUTER_API_KEY".to_string(), "sk-or-v1-xxx".to_string());

    let plist = generate_plist(
        "/Users/test/.local/share/lcc/lcc-litellm-launcher.sh",
        "/Users/test/Library/Logs/lcc/litellm.out.log",
        "/Users/test/Library/Logs/lcc/litellm.err.log",
        &env,
    );

    assert!(plist.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    assert!(plist.contains("<key>Label</key>"));
    assert!(plist.contains("<string>com.lcc.litellm</string>"));
    assert!(plist.contains("<key>ProgramArguments</key>"));
    assert!(plist.contains("/Users/test/.local/share/lcc/lcc-litellm-launcher.sh"));
    assert!(plist.contains("<key>RunAtLoad</key>"));
    assert!(plist.contains("<true/>"));
    assert!(plist.contains("<key>KeepAlive</key>"));
    assert!(plist.contains("<key>StandardOutPath</key>"));
    assert!(plist.contains("/litellm.out.log"));
    assert!(plist.contains("<key>StandardErrorPath</key>"));
    assert!(plist.contains("<key>EnvironmentVariables</key>"));
    assert!(plist.contains("<key>OPENROUTER_API_KEY</key>"));
    assert!(plist.contains("<string>sk-or-v1-xxx</string>"));
}
```

- [ ] **Step 2: Implement plist generator**

```rust
// src/proxy/generators/plist.rs
//! Génère le plist LaunchAgent pour le daemon LiteLLM.
//!
//! Format Apple plist XML (cf `man launchd.plist`).

use crate::proxy::LAUNCHD_LABEL;
use std::collections::HashMap;

pub fn generate_plist(
    wrapper_path: &str,
    stdout_log: &str,
    stderr_log: &str,
    env_vars: &HashMap<String, String>,
) -> String {
    let mut env_block = String::new();
    // Sort for deterministic output
    let mut keys: Vec<&String> = env_vars.keys().collect();
    keys.sort();
    for k in keys {
        env_block.push_str(&format!(
            "        <key>{}</key>\n        <string>{}</string>\n",
            xml_escape(k),
            xml_escape(&env_vars[k]),
        ));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{LAUNCHD_LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{}</string>
    <key>StandardErrorPath</key>
    <string>{}</string>
    <key>EnvironmentVariables</key>
    <dict>
{}    </dict>
</dict>
</plist>
"#,
        xml_escape(wrapper_path),
        xml_escape(stdout_log),
        xml_escape(stderr_log),
        env_block,
    )
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
```

- [ ] **Step 3: Run plist test, verify pass**

```bash
cargo test --test proxy_plist_test
```
Expected: PASS.

- [ ] **Step 4: Write failing wrapper test**

```rust
// tests/proxy_wrapper_test.rs
use launch_claude_code::proxy::generators::wrapper::generate_wrapper;

#[test]
fn wrapper_loads_master_key_then_execs_litellm() {
    let wrapper = generate_wrapper(
        "/Users/test/.local/share/lcc/litellm-venv/bin/litellm",
        "/Users/test/.config/launch-claude-code/litellm.yaml",
        4000,
    );

    assert!(wrapper.starts_with("#!/bin/bash"));
    assert!(wrapper.contains("set -euo pipefail"));
    assert!(wrapper.contains(
        r#"LCC_MASTER_KEY="$(security find-generic-password -s lcc.litellm.master_key -w)""#
    ));
    assert!(wrapper.contains("export LCC_MASTER_KEY"));
    assert!(wrapper.contains(
        "exec /Users/test/.local/share/lcc/litellm-venv/bin/litellm"
    ));
    assert!(wrapper.contains("--config /Users/test/.config/launch-claude-code/litellm.yaml"));
    assert!(wrapper.contains("--port 4000"));
}
```

- [ ] **Step 5: Implement wrapper generator**

```rust
// src/proxy/generators/wrapper.rs
//! Génère le wrapper bash lancé par launchd. Le wrapper récupère la
//! master_key depuis le Keychain puis exec litellm avec la bonne config.

use crate::proxy::KEYCHAIN_SERVICE;

pub fn generate_wrapper(litellm_bin: &str, yaml_path: &str, port: u16) -> String {
    format!(
        r#"#!/bin/bash
# Wrapper généré par `lcc proxy install`. NE PAS ÉDITER À LA MAIN.
# Re-générer via `lcc proxy reload` après changement de config.

set -euo pipefail

# Récupération de la master_key depuis le Keychain macOS.
# Échoue net si la clé n'existe pas (lcc proxy install non lancé).
LCC_MASTER_KEY="$(security find-generic-password -s {KEYCHAIN_SERVICE} -w)"
export LCC_MASTER_KEY

# S'assure que le PYTHONPATH inclut le site-packages du venv pour le callback custom.
exec {litellm_bin} --config {yaml_path} --port {port}
"#
    )
}
```

- [ ] **Step 6: Run wrapper test, verify pass**

```bash
cargo test --test proxy_wrapper_test
```
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/proxy/generators/ tests/proxy_plist_test.rs tests/proxy_wrapper_test.rs
git commit -m "feat(proxy): plist + wrapper generators"
```

---

## Phase 2 — System wrappers

### Task 2.1: keychain wrapper

**Files:**
- Modify: `src/proxy/keychain.rs`

**Note:** Pas de test unitaire automatique (touche le Keychain réel). Tests manuels après implémentation.

- [ ] **Step 1: Implement keychain module**

```rust
// src/proxy/keychain.rs
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
```

- [ ] **Step 2: Compile**

```bash
cargo build
```
Expected: success.

- [ ] **Step 3: Manual smoke test**

```bash
# Test interactif (n'écrit dans le Keychain qu'une fois cargo run + une commande dédiée existe).
# Pour ce step, on se contente de vérifier que le module compile.
echo "OK — tests Keychain réels viendront en Task 4.1."
```

- [ ] **Step 4: Commit**

```bash
git add src/proxy/keychain.rs
git commit -m "feat(proxy): keychain wrapper (set/get/delete master_key)"
```

---

### Task 2.2: launchctl wrapper

**Files:**
- Modify: `src/proxy/launchctl.rs`
- Modify: `src/proxy/lifecycle.rs`

- [ ] **Step 1: Implement launchctl module**

```rust
// src/proxy/launchctl.rs
//! Wrapper sync sur `launchctl` pour gérer le LaunchAgent du proxy.

use crate::proxy::{plist_path, LAUNCHD_LABEL};
use std::io::{self, ErrorKind};
use std::process::Command;

fn service_target() -> io::Result<String> {
    let uid = unsafe { libc_getuid() };
    Ok(format!("gui/{uid}/{LAUNCHD_LABEL}"))
}

extern "C" {
    fn getuid() -> u32;
}

#[allow(non_snake_case)]
unsafe fn libc_getuid() -> u32 {
    getuid()
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
```

- [ ] **Step 2: Implement lifecycle wrappers**

```rust
// src/proxy/lifecycle.rs
//! Sous-commandes simples : start / stop / restart.

use crate::proxy::launchctl;
use std::io;

pub fn start() -> io::Result<()> {
    println!("→ launchctl load…");
    launchctl::load()?;
    println!("✓ proxy lancé.");
    Ok(())
}

pub fn stop() -> io::Result<()> {
    println!("→ launchctl unload…");
    launchctl::unload()?;
    println!("✓ proxy arrêté.");
    Ok(())
}

pub fn restart() -> io::Result<()> {
    println!("→ launchctl kickstart -k…");
    launchctl::kickstart()?;
    println!("✓ proxy redémarré.");
    Ok(())
}
```

- [ ] **Step 3: Compile**

```bash
cargo build
```
Expected: success (libc getuid via extern "C" should compile on macOS).

- [ ] **Step 4: Commit**

```bash
git add src/proxy/launchctl.rs src/proxy/lifecycle.rs
git commit -m "feat(proxy): launchctl + lifecycle wrappers"
```

---

### Task 2.3: health check

**Files:**
- Modify: `src/proxy/health.rs`
- Create: `tests/proxy_health_test.rs`

- [ ] **Step 1: Write failing test (live mock server)**

```rust
// tests/proxy_health_test.rs
use launch_claude_code::proxy::health::is_alive;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

/// Mini HTTP server qui répond 200 sur GET /health/liveness.
fn spawn_mock_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        for stream in listener.incoming().take(5) {
            if let Ok(mut s) = stream {
                use std::io::{Read, Write};
                let mut buf = [0u8; 1024];
                let _ = s.read(&mut buf);
                let _ = s.write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 21\r\n\r\n{\"status\":\"healthy\"}\n",
                );
            }
        }
    });
    // Donne un peu de temps au listener
    thread::sleep(Duration::from_millis(50));
    port
}

#[test]
fn alive_when_server_responds_200() {
    let port = spawn_mock_server();
    assert!(is_alive(port, Duration::from_secs(1)));
}

#[test]
fn dead_when_no_server() {
    // Port très probablement libre
    assert!(!is_alive(59999, Duration::from_millis(200)));
}
```

- [ ] **Step 2: Verify failure**

```bash
cargo test --test proxy_health_test
```
Expected: compile error (function missing).

- [ ] **Step 3: Implement health check**

```rust
// src/proxy/health.rs
//! Health check HTTP du daemon LiteLLM (`GET /health/liveness`).

use std::time::Duration;

/// Retourne `true` si le proxy répond 200 sur `/health/liveness`.
/// Utilise `ureq` (sync, léger).
pub fn is_alive(port: u16, timeout: Duration) -> bool {
    let url = format!("http://127.0.0.1:{port}/health/liveness");
    let agent = ureq::AgentBuilder::new()
        .timeout(timeout)
        .build();
    match agent.get(&url).call() {
        Ok(resp) => resp.status() == 200,
        Err(_) => false,
    }
}
```

- [ ] **Step 4: Run, verify pass**

```bash
cargo test --test proxy_health_test
```
Expected: both tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/proxy/health.rs tests/proxy_health_test.rs
git commit -m "feat(proxy): health check HTTP"
```

---

## Phase 3 — Python callback

### Task 3.1: lcc_strip_thinking.py + pytest

**Files:**
- Create: `assets/lcc_strip_thinking.py`
- Create: `assets/tests/test_strip_thinking.py`
- Create: `assets/tests/conftest.py` (vide pour le moment)
- Create: `assets/requirements-dev.txt`

- [ ] **Step 1: Create the callback**

```python
# assets/lcc_strip_thinking.py
"""LiteLLM callback: strip 'thinking' content blocks without Anthropic signature.

Claude Code valide une signature cryptographique sur les blocs `thinking`.
Les modèles reasoning hostés ailleurs (Qwen, DeepSeek) renvoient des blocs
thinking sans signature → CC drop la réponse en silence.

Ce callback retire ces blocs problématiques avant que LiteLLM forward la
réponse à `claude`.
"""

from typing import Any

from litellm.integrations.custom_logger import CustomLogger


class StripThinkingCallback(CustomLogger):
    async def async_post_call_success_hook(
        self,
        data: dict[str, Any],
        user_api_key_dict: Any,
        response: Any,
    ) -> Any:
        _strip_thinking_blocks(response)
        return response


def _strip_thinking_blocks(response: Any) -> None:
    """Mute les blocs thinking sans signature dans une réponse Anthropic-shape."""
    content = getattr(response, "content", None)
    if not isinstance(content, list):
        return
    response.content = [b for b in content if not _is_unsigned_thinking(b)]


def _is_unsigned_thinking(block: Any) -> bool:
    if not isinstance(block, dict):
        return False
    return block.get("type") == "thinking" and "signature" not in block


# LiteLLM résout `callbacks: ["lcc_strip_thinking"]` en cherchant cet attribut module-level.
lcc_strip_thinking = StripThinkingCallback()
```

- [ ] **Step 2: Create the pytest**

```python
# assets/tests/test_strip_thinking.py
"""Tests unitaires pour le callback strip_thinking."""

from types import SimpleNamespace

from lcc_strip_thinking import _strip_thinking_blocks


def make_response(content: list) -> SimpleNamespace:
    return SimpleNamespace(content=content)


def test_strips_unsigned_thinking_block():
    resp = make_response([
        {"type": "thinking", "thinking": "raisonnement…"},
        {"type": "text", "text": "Hello!"},
    ])
    _strip_thinking_blocks(resp)
    assert resp.content == [{"type": "text", "text": "Hello!"}]


def test_keeps_signed_thinking_block():
    resp = make_response([
        {"type": "thinking", "thinking": "…", "signature": "sig-abc"},
        {"type": "text", "text": "yo"},
    ])
    _strip_thinking_blocks(resp)
    assert len(resp.content) == 2
    assert resp.content[0]["type"] == "thinking"


def test_handles_no_content_attribute():
    resp = SimpleNamespace()  # pas de .content
    _strip_thinking_blocks(resp)  # ne doit pas crasher


def test_handles_non_list_content():
    resp = make_response("plain string")
    _strip_thinking_blocks(resp)
    assert resp.content == "plain string"  # inchangé


def test_empty_after_strip_is_ok():
    resp = make_response([
        {"type": "thinking", "thinking": "…"},
    ])
    _strip_thinking_blocks(resp)
    assert resp.content == []
```

- [ ] **Step 3: Create dev deps file**

```
# assets/requirements-dev.txt
pytest>=8.0
litellm>=1.50
```

- [ ] **Step 4: Run pytest**

```bash
cd assets
uv venv .venv-dev
source .venv-dev/bin/activate
uv pip install -r requirements-dev.txt
PYTHONPATH=. pytest tests/test_strip_thinking.py -v
deactivate
```
Expected: 5 tests pass.

- [ ] **Step 5: Commit**

```bash
git add assets/
git commit -m "feat(proxy): python callback to strip unsigned thinking blocks"
```

---

## Phase 4 — Install orchestration

### Task 4.1: install — uv check + venv install

**Files:**
- Modify: `src/proxy/install.rs`

- [ ] **Step 1: Implement uv detection + LiteLLM install**

```rust
// src/proxy/install.rs
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
```

- [ ] **Step 2: Compile**

```bash
cargo build
```

- [ ] **Step 3: Smoke test (manual, depuis un shell séparé)**

```bash
# Vérifie que uv est installé. Si non :
brew install uv

# Note : le smoke test du install complet viendra en Task 4.4.
echo "Step compile-only OK."
```

- [ ] **Step 4: Commit**

```bash
git add src/proxy/install.rs
git commit -m "feat(proxy): install — uv detect + litellm uv tool install"
```

---

### Task 4.2: install — write yaml/plist/wrapper, generate keychain key

**Files:**
- Modify: `src/proxy/install.rs`

- [ ] **Step 1: Add file-writing functions to install.rs**

Append to `src/proxy/install.rs` :
```rust
use crate::config::load_settings;
use crate::proxy::{
    generators::{plist::generate_plist, wrapper::generate_wrapper, yaml::generate_litellm_yaml},
    keychain, logs_dir, plist_path, wrapper_path, yaml_path, DEFAULT_PORT,
};
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;

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
```

- [ ] **Step 2: Compile**

```bash
cargo build
```
Expected: success.

- [ ] **Step 3: Commit**

```bash
git add src/proxy/install.rs
git commit -m "feat(proxy): install — write yaml/wrapper/plist + master_key"
```

---

### Task 4.3: install — copy Python callback + launchctl load + wait health

**Files:**
- Modify: `src/proxy/install.rs`

- [ ] **Step 1: Add callback-copy + launch + wait functions**

Append to `install.rs` :
```rust
use crate::proxy::{health, launchctl, DEFAULT_PORT as PORT};
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

/// Path du fichier Python du callback dans les assets du repo.
/// En dev (cargo run depuis le repo) : ./assets/lcc_strip_thinking.py
/// En install (cargo install --path .) : embarqué via include_str! ci-dessous.
const CALLBACK_PY: &str = include_str!("../../assets/lcc_strip_thinking.py");

pub fn install_python_callback() -> io::Result<()> {
    let venv_dir = uv_tool_dir().join("litellm");
    let site_packages = find_site_packages(&venv_dir)?;
    let dest = site_packages.join("lcc_strip_thinking.py");
    fs::write(&dest, CALLBACK_PY)?;
    println!("✓ callback Python installé : {}", dest.display());
    Ok(())
}

fn uv_tool_dir() -> PathBuf {
    let output = Command::new("uv")
        .args(["tool", "dir"])
        .output()
        .expect("uv tool dir");
    PathBuf::from(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn find_site_packages(venv_dir: &PathBuf) -> io::Result<PathBuf> {
    let lib = venv_dir.join("lib");
    for entry in fs::read_dir(&lib)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() && p.file_name().unwrap().to_string_lossy().starts_with("python") {
            let sp = p.join("site-packages");
            if sp.exists() {
                return Ok(sp);
            }
        }
    }
    Err(io::Error::new(
        ErrorKind::NotFound,
        format!("site-packages introuvable sous {}", lib.display()),
    ))
}

pub fn load_and_wait() -> io::Result<()> {
    println!("→ launchctl load…");
    launchctl::load()?;
    println!("→ attente health check (max 10s)…");
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if health::is_alive(PORT, Duration::from_millis(500)) {
            println!("✓ proxy ready on :{PORT}");
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
```

- [ ] **Step 2: Implement the top-level `run_install`**

Append to `install.rs` :
```rust
pub fn run_install() -> io::Result<()> {
    println!("=== lcc proxy install ===\n");
    ensure_uv_installed()?;
    install_litellm_via_uv()?;
    write_master_key_to_keychain()?;
    write_yaml()?;
    write_wrapper()?;
    write_plist()?;
    install_python_callback()?;
    load_and_wait()?;
    println!("\n✓ tout est prêt. `lcc start --profil <X>` devrait marcher.");
    Ok(())
}
```

- [ ] **Step 3: Compile**

```bash
cargo build
```

- [ ] **Step 4: Commit**

```bash
git add src/proxy/install.rs
git commit -m "feat(proxy): install — python callback + launchctl + wait health"
```

---

### Task 4.4: Wire `proxy install` to CLI + first manual smoke

**Files:**
- Modify: `src/proxy/cli.rs`

- [ ] **Step 1: Implement Proxy enum**

```rust
// src/proxy/cli.rs
//! Subcommand `lcc proxy *` — dispatch.

use clap::Subcommand;
use std::io;

use crate::proxy::{install, lifecycle, reload, status, uninstall};

#[derive(Subcommand, Debug)]
pub enum Proxy {
    /// Setup complet du proxy LiteLLM (uv install + plist + keychain + load).
    Install,
    /// Désinstalle le proxy (unload + supprime plist).
    Uninstall {
        /// Supprime aussi le yaml, le venv litellm et la master_key.
        #[arg(long)]
        purge: bool,
    },
    /// Régénère le yaml depuis settings.json + redémarre le daemon.
    Reload,
    /// Démarre le LaunchAgent.
    Start,
    /// Arrête le LaunchAgent.
    Stop,
    /// Redémarre le LaunchAgent (kickstart -k).
    Restart,
    /// Affiche l'état du proxy (PID, port, modèles routés).
    Status,
    /// Tail des logs du daemon.
    Logs {
        /// Suivre en continu (-f).
        #[arg(short = 'f', long)]
        follow: bool,
    },
    /// Diagnostic complet (uv ? plist ? daemon ? port ? yaml ?).
    Doctor,
}

pub fn dispatch(cmd: Proxy) -> io::Result<()> {
    match cmd {
        Proxy::Install => install::run_install(),
        Proxy::Uninstall { purge } => uninstall::run_uninstall(purge),
        Proxy::Reload => reload::run_reload(),
        Proxy::Start => lifecycle::start(),
        Proxy::Stop => lifecycle::stop(),
        Proxy::Restart => lifecycle::restart(),
        Proxy::Status => status::run_status(),
        Proxy::Logs { follow } => status::run_logs(follow),
        Proxy::Doctor => status::run_doctor(),
    }
}
```

- [ ] **Step 2: Wire in main.rs**

In `src/main.rs`, find the `Commands` enum and add :
```rust
/// Gestion du proxy LiteLLM autonome.
Proxy {
    #[command(subcommand)]
    cmd: crate::proxy::cli::Proxy,
},
```

In the dispatch `match` block of `main`, add :
```rust
Commands::Proxy { cmd } => crate::proxy::cli::dispatch(cmd)
    .map_err(|e| { eprintln!("✗ {e}"); std::process::exit(1) })
    .unwrap(),
```

(Adjust to match the existing error handling style of `main.rs`.)

- [ ] **Step 3: Compile**

```bash
cargo build
```
Expected: success. May need stub functions in `uninstall.rs`, `reload.rs`, `status.rs` returning `Ok(())` to satisfy the dispatch — implemented in Phase 5.

- [ ] **Step 4: First end-to-end manual smoke**

```bash
# Pré-requis : settings.json contient un profil "qwen" avec base_url openrouter
# et OPENROUTER_API_KEY exporté dans .zshrc.
cargo run -- proxy install
# Vérifications attendues :
#   - "✓ uv détecté"
#   - "→ uv tool install litellm[proxy]" puis OK
#   - "✓ master_key générée"
#   - "✓ litellm.yaml généré"
#   - "✓ wrapper généré"
#   - "✓ plist généré"
#   - "✓ callback Python installé"
#   - "→ launchctl load"
#   - "✓ proxy ready on :4000"

# Vérification indépendante :
curl -sS http://localhost:4000/health/liveness
# Attendu : {"status":"healthy"}
```

- [ ] **Step 5: Commit**

```bash
git add src/proxy/cli.rs src/main.rs
git commit -m "feat(proxy): cli wiring + first end-to-end install"
```

---

## Phase 5 — Companion commands

### Task 5.1: uninstall + reload

**Files:**
- Modify: `src/proxy/uninstall.rs`
- Modify: `src/proxy/reload.rs`

- [ ] **Step 1: Implement uninstall**

```rust
// src/proxy/uninstall.rs
//! `lcc proxy uninstall [--purge]` — symétrique de `install`.

use crate::proxy::{keychain, launchctl, plist_path, wrapper_path, yaml_path};
use std::fs;
use std::io;
use std::process::Command;

pub fn run_uninstall(purge: bool) -> io::Result<()> {
    println!("=== lcc proxy uninstall ===\n");

    println!("→ launchctl unload…");
    let _ = launchctl::unload();

    let plist = plist_path();
    if plist.exists() {
        fs::remove_file(&plist)?;
        println!("✓ plist supprimé : {}", plist.display());
    }

    let wrapper = wrapper_path();
    if wrapper.exists() {
        fs::remove_file(&wrapper)?;
        println!("✓ wrapper supprimé");
    }

    if purge {
        let yaml = yaml_path();
        if yaml.exists() {
            fs::remove_file(&yaml)?;
            println!("✓ litellm.yaml supprimé (--purge)");
        }
        let _ = keychain::delete_master_key();
        println!("✓ master_key supprimée du Keychain (--purge)");

        println!("→ uv tool uninstall litellm…");
        let _ = Command::new("uv")
            .args(["tool", "uninstall", "litellm"])
            .status();
        println!("✓ litellm désinstallé (--purge)");
    } else {
        println!(
            "ℹ litellm venv, yaml et master_key conservés. Utilise --purge pour tout effacer."
        );
    }

    println!("\n✓ uninstall terminé.");
    Ok(())
}
```

- [ ] **Step 2: Implement reload**

```rust
// src/proxy/reload.rs
//! `lcc proxy reload` — régénère le yaml depuis settings.json + restart.

use crate::proxy::{install, launchctl};
use std::io;

pub fn run_reload() -> io::Result<()> {
    println!("=== lcc proxy reload ===\n");
    install::write_yaml()?;
    println!("→ launchctl kickstart -k…");
    launchctl::kickstart()?;
    println!("✓ proxy rechargé.");
    Ok(())
}
```

- [ ] **Step 3: Compile + smoke**

```bash
cargo build
cargo run -- proxy reload
# Attendu : "✓ proxy rechargé." + curl /health/liveness toujours OK
```

- [ ] **Step 4: Commit**

```bash
git add src/proxy/uninstall.rs src/proxy/reload.rs
git commit -m "feat(proxy): uninstall + reload commands"
```

---

### Task 5.2: status + logs (basique)

**Files:**
- Modify: `src/proxy/status.rs`

- [ ] **Step 1: Implement status / logs / doctor**

```rust
// src/proxy/status.rs
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
    check("uv installé", Command::new("uv").arg("--version").status().is_ok());
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
```

- [ ] **Step 2: Compile + smoke**

```bash
cargo build
cargo run -- proxy status
cargo run -- proxy doctor
cargo run -- proxy logs
```
Expected: status shows "up + OK", doctor shows all green ✓.

- [ ] **Step 3: Commit**

```bash
git add src/proxy/status.rs
git commit -m "feat(proxy): status + logs + doctor (V1 basique)"
```

---

## Phase 6 — Runner integration

### Task 6.1: health check pre-spawn in runner

**Files:**
- Modify: `src/runner.rs:134-184` (la fonction `run`)

- [ ] **Step 1: Add health check + helpful error**

In `src/runner.rs`, at the top of the `run` function (right after profile resolution, before env build), insert :
```rust
use crate::proxy::{health, DEFAULT_PORT};
use std::time::Duration;

if !health::is_alive(DEFAULT_PORT, Duration::from_secs(1)) {
    return Err(format!(
        "✗ Le proxy LiteLLM ne répond pas sur :{DEFAULT_PORT}.\n  → Lance: lcc proxy doctor\n  → Ou:   lcc proxy start"
    )
    .into());
}
```

(Ajuste le type `Box<dyn Error>` ou équivalent pour matcher le retour de `run`.)

- [ ] **Step 2: Add corresponding test**

In `tests/runner_test.rs`, append :
```rust
#[test]
fn run_fails_clearly_when_proxy_down() {
    // Cette fonction n'est pas trivialement testable en unit (lancement de claude).
    // À couvrir en E2E manuel : voir Task 7.2.
}
```

- [ ] **Step 3: Compile + manual smoke**

```bash
cargo build
# Test 1 : avec proxy up → start fonctionne (cf Task 6.2)
# Test 2 : stoppe le proxy puis tente un start
cargo run -- proxy stop
cargo run -- start --profil qwen -p "hello"
# Attendu : "✗ Le proxy LiteLLM ne répond pas..."
cargo run -- proxy start
```

- [ ] **Step 4: Commit**

```bash
git add src/runner.rs tests/runner_test.rs
git commit -m "feat(runner): health check du proxy avant spawn claude"
```

---

### Task 6.2: env override toward proxy

**Files:**
- Modify: `src/runner.rs:42-126` (les fonctions `build_single_env` / `build_multi_env`)

- [ ] **Step 1: Override BASE_URL, AUTH_TOKEN, MODEL**

In `build_single_env` (around line 42-64 of runner.rs), replace the construction of env vars to **always** point to the proxy. The provider's actual `base_url` and `auth_token` are no longer passed to `claude` — they're already in the LiteLLM yaml.

```rust
use crate::proxy::{keychain, DEFAULT_PORT};

// Construction des env vars pour profil Single — toujours via proxy.
fn build_single_env(profile_name: &str, sp: &SingleProfile) -> Result<HashMap<String, String>, ...> {
    let master_key = keychain::get_master_key()
        .map_err(|e| format!("master_key Keychain : {e}. Lance `lcc proxy install`."))?;

    let mut env = HashMap::new();
    env.insert("ANTHROPIC_BASE_URL".to_string(),
               format!("http://localhost:{DEFAULT_PORT}"));
    env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), master_key);
    // Le model name pour LiteLLM est le nom du profil (cf yaml gen).
    env.insert("ANTHROPIC_MODEL".to_string(), profile_name.to_string());
    // Slots Default* : on les pointe sur le même alias par défaut.
    // Plus fin si besoin via Multi profiles (Task 6.2.bis).
    env.insert(
        "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
        profile_name.to_string(),
    );
    env.insert(
        "ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
        profile_name.to_string(),
    );
    env.insert(
        "ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
        profile_name.to_string(),
    );
    env.insert("CLAUDE_CODE_ATTRIBUTION_HEADER".to_string(), "0".to_string());
    // env vars custom du profil (inchangé)
    if let Some(custom) = &sp.env {
        for (k, v) in custom {
            env.insert(k.clone(), expand_env_vars(v));
        }
    }
    Ok(env)
}
```

- [ ] **Step 2: Same logic for Multi**

In `build_multi_env`, mappe chaque slot (Opus/Sonnet/Haiku) sur l'alias composite généré par yaml gen (`<profile>/<model_alias>`) :
```rust
fn build_multi_env(profile_name: &str, mp: &MultiProfile) -> Result<HashMap<String, String>, ...> {
    let master_key = keychain::get_master_key()
        .map_err(|e| format!("master_key Keychain : {e}. Lance `lcc proxy install`."))?;
    let mut env = HashMap::new();
    env.insert("ANTHROPIC_BASE_URL".to_string(),
               format!("http://localhost:{DEFAULT_PORT}"));
    env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), master_key);

    // Default = le model nommé dans `default`
    let default_alias = format!("{profile_name}/{}", mp.default);
    env.insert("ANTHROPIC_MODEL".to_string(), default_alias.clone());

    // Mappe chaque slot sur le model dont le `slot` correspond.
    for (alias, entry) in &mp.models {
        let composite = format!("{profile_name}/{alias}");
        let key = match entry.slot {
            ModelSlot::Opus => "ANTHROPIC_DEFAULT_OPUS_MODEL",
            ModelSlot::Sonnet => "ANTHROPIC_DEFAULT_SONNET_MODEL",
            ModelSlot::Haiku => "ANTHROPIC_DEFAULT_HAIKU_MODEL",
            ModelSlot::Custom => continue,
        };
        env.insert(key.to_string(), composite);
    }
    env.insert("CLAUDE_CODE_ATTRIBUTION_HEADER".to_string(), "0".to_string());
    if let Some(custom) = &mp.env {
        for (k, v) in custom {
            env.insert(k.clone(), expand_env_vars(v));
        }
    }
    Ok(env)
}
```

- [ ] **Step 3: Update existing runner tests**

Les tests existants (`runner_test.rs`) check que les anciennes env vars (`ANTHROPIC_API_KEY` direct, `ANTHROPIC_BASE_URL` du provider) sont set. Ces tests vont **casser**. C'est attendu : on les met à jour pour vérifier les NOUVELLES env vars (toutes pointent vers proxy, master_key Keychain).

Pour rendre les tests passables sans Keychain réel, mocker `keychain::get_master_key` via une feature flag `mock-keychain` :
```rust
#[cfg(feature = "mock-keychain")]
pub fn get_master_key() -> std::io::Result<String> { Ok("mock-master-key".to_string()) }
```
Et activer la feature dans `Cargo.toml [dev-dependencies]` ou via `[features] mock-keychain = []` + `cargo test --features mock-keychain`.

Update les tests :
```rust
#[test]
fn single_profile_env_points_to_proxy() {
    let sp = SingleProfile {
        model: "qwen/qwen3.6-plus".to_string(),
        base_url: "https://openrouter.ai/api/v1".to_string(),
        api_key: None,
        auth_token: Some("${OPENROUTER_API_KEY}".to_string()),
        env: None,
    };
    let env = build_single_env("qwen", &sp).unwrap();
    assert_eq!(env.get("ANTHROPIC_BASE_URL"), Some(&"http://localhost:4000".to_string()));
    assert_eq!(env.get("ANTHROPIC_AUTH_TOKEN"), Some(&"mock-master-key".to_string()));
    assert_eq!(env.get("ANTHROPIC_MODEL"), Some(&"qwen".to_string()));
}
```

- [ ] **Step 4: Run tests**

```bash
cargo test --features mock-keychain
```
Expected: all tests pass (including the updated runner tests).

- [ ] **Step 5: Commit**

```bash
git add src/runner.rs tests/runner_test.rs Cargo.toml src/proxy/keychain.rs
git commit -m "feat(runner): env vars overridées vers le proxy local"
```

---

## Phase 7 — E2E smoke + documentation

### Task 7.1: E2E smoke test (manual checklist)

**Files:**
- Modify: `README.md` — ajouter section "Smoke test V1 proxy"

- [ ] **Step 1: Add README section**

Append to `README.md` :
```markdown
## Smoke test du proxy LiteLLM (V1)

Pré-requis :
- macOS
- `brew install uv`
- `OPENROUTER_API_KEY` (ou autre clé provider) exportée dans `.zshrc`
- Un profil dans `~/.config/launch-claude-code/settings.json` pointant vers OpenRouter (ex: `qwen` avec `model: qwen/qwen3.6-plus`)

```bash
# 1. Build + install
cargo install --path .

# 2. Setup proxy (one-shot, ~30s)
lcc proxy install

# 3. Vérifier que tout est vert
lcc proxy doctor
# Attendu : 7 lignes "[✓]"

# 4. Premier appel
lcc start --profil qwen -p "Dis moi bonjour en une phrase"
# Attendu : une vraie réponse texte (pas de terminal vide !)

# 5. Vérifier que les `thinking` blocks sont strippés
lcc proxy logs | grep -i thinking || echo "OK : pas de thinking dans les logs récents"

# 6. Tester le path direct claude (doit toujours marcher)
claude -p "ping"
# Attendu : réponse Anthropic native, totalement indépendant du proxy
```

Si le step 4 échoue silencieusement (terminal vide), c'est une régression. Vérifier :
- `lcc proxy logs` — l'erreur exacte renvoyée par le provider
- `curl -sS http://localhost:4000/v1/messages -H "x-api-key: $(security find-generic-password -s lcc.litellm.master_key -w)" -H "anthropic-version: 2023-06-01" -H "content-type: application/json" -d '{"model":"qwen","max_tokens":50,"messages":[{"role":"user","content":"hi"}]}'` — appel direct au proxy
```

- [ ] **Step 2: Manual run of the full smoke**

Suis pas-à-pas la checklist du README. Documente les écarts éventuels (ex: une étape doit être ré-écrite parce que la commande exacte ne marche pas). Si tout marche → succès du V1.

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "docs: smoke test V1 du proxy LiteLLM"
```

---

### Task 7.2: Final cleanup + version bump

**Files:**
- Modify: `Cargo.toml` (version bump)
- Modify: `CHANGELOG.md` ou notes de release

- [ ] **Step 1: Vérifier qu'aucun warning cargo ne reste**

```bash
cargo build --release 2>&1 | grep -E "warning|error"
```
Si warnings : fix-les ou allowlist explicite.

- [ ] **Step 2: Vérifier que tous les tests passent**

```bash
cargo test
cargo test --features mock-keychain
cd assets && PYTHONPATH=. .venv-dev/bin/pytest tests/ -v && cd ..
```
Expected : tout vert.

- [ ] **Step 3: Bump version**

In `Cargo.toml`, bump `version = "0.1.1"` → `version = "0.2.0"`.

- [ ] **Step 4: Commit + tag**

```bash
git add Cargo.toml
git commit -m "release: v0.2.0 — proxy LiteLLM autonome"
git tag v0.2.0
# Note: ne pas push --tags sans confirmation explicite de Romain
```

---

## Self-Review Checklist

**Spec coverage** (vérifié contre `2026-05-15-lcc-proxy-autonome-design.md`) :

| Spec section | Couvert par |
|---|---|
| §3 Séparation claude/lcc | Task 6.2 (env override systématique) + Task 7.1 (smoke valide les deux paths) |
| §4 Architecture data flow | Task 6.1 + 6.2 |
| §4.1 Composants & paths | Task 0.1 (paths constants) |
| §4.2 Master key keychain | Task 2.1 + 4.2 + 4.3 |
| §5 Subcommands proxy * | Task 4.4 + 5.1 + 5.2 |
| §6.1 yaml gen exemple | Task 1.2 + 1.3 |
| §6.2 algo gen | Task 1.1 + 1.2 + 1.3 |
| §6.3 Strip thinking callback | Task 3.1 + 4.3 (copy to venv) |
| §7.1 Health check pre-spawn | Task 6.1 |
| §7.2 Doctor | Task 5.2 |
| §8 Sécurité | Task 4.2 (env snapshot to plist) + 2.1 (keychain) |
| §9 Tests | Tasks 1.x, 2.3, 3.1 |
| §10 Non-objectifs | respectés (pas d'impl) |
| §11 Migration | Task 7.1 (README) |
| §12 Risques | Mitigations dans les tasks (mock-keychain feature, openssl rand fallback, sort des HashMap pour déterminisme) |

**Placeholders scan** : aucun "TBD"/"TODO"/"implement later" laissé. Les `// Note:` indiquent des points d'attention concrets.

**Type consistency check** :
- `generate_litellm_yaml(&Settings)` est appelé partout avec `&Settings` ✓
- `keychain::get_master_key() -> io::Result<String>` cohérent dans install + runner ✓
- `health::is_alive(port: u16, timeout: Duration) -> bool` cohérent ✓
- `Profile::Single`/`Profile::Multi` enum names matchent `config.rs` (à reverifier au moment de Task 1.2 — si l'enum existant utilise d'autres noms, ajuster le code des tasks)
- `DEFAULT_PORT` constant utilisé partout (pas de magic 4000 dans le code) ✓
- Tous les paths via les helpers de `proxy/mod.rs` (pas de path hard-codé dans les tasks) ✓

**Spec → plan gap check** : aucune exigence du spec sans task associée. La V2 (proxy Rust maison, multi-OS, etc.) est explicitement hors scope V1 — pas une lacune.

---

## Notes de mise en œuvre

- **Crate name** : Task 1.1 step 1 mentionne `launch_claude_code` comme nom de crate. Vérifier le vrai nom dans `Cargo.toml` au moment d'écrire les tests. Si le projet n'a pas de `lib`, **ajouter une `[lib]` section + `src/lib.rs` qui re-exporte `pub mod config; pub mod proxy; pub mod runner;`** avant la Task 1.1 step 5. Sinon les tests d'intégration ne pourront pas importer ces modules.
- **Cargo features** : ajouter `[features] mock-keychain = []` dans `Cargo.toml` pour les tests de la Task 6.2.
- **Ordre des commits** : chaque task se termine par un commit. Si tu fais le plan en subagent-driven, respect strict de cet ordre.
- **Détection régression** : le test E2E manuel de Task 7.1 est la **seule** validation que le bug initial (Qwen silent fail) est résolu. Sans ce test passé, la V1 n'est pas livrable.
