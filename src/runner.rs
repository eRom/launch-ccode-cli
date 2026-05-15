use crate::config::{expand_env_vars, load_settings, MultiProfile, Profile, SingleProfile, Slot};
use crate::proxy::{health, DEFAULT_PORT};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

pub fn find_claude() -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Ok(output) = Command::new("which").arg("claude").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Ok(PathBuf::from(path));
        }
    }

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

/// Construit les variables d'environnement pour un profil Single.
/// Toutes les invocations lcc passent par le proxy local (localhost:DEFAULT_PORT).
/// `profile_name` est le nom court du profil (alias LiteLLM dans le yaml).
pub fn build_env_vars_single(
    profile_name: &str,
    profile: &SingleProfile,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let master_key = crate::proxy::keychain::get_master_key()
        .map_err(|e| format!("master_key Keychain : {e}. Lance `lcc proxy install`."))?;

    let mut env = HashMap::new();
    env.insert(
        "ANTHROPIC_BASE_URL".into(),
        format!("http://localhost:{DEFAULT_PORT}"),
    );
    env.insert("ANTHROPIC_AUTH_TOKEN".into(), master_key);
    env.insert("ANTHROPIC_MODEL".into(), profile_name.to_string());
    env.insert(
        "ANTHROPIC_DEFAULT_OPUS_MODEL".into(),
        profile_name.to_string(),
    );
    env.insert(
        "ANTHROPIC_DEFAULT_SONNET_MODEL".into(),
        profile_name.to_string(),
    );
    env.insert(
        "ANTHROPIC_DEFAULT_HAIKU_MODEL".into(),
        profile_name.to_string(),
    );
    env.insert("CLAUDE_CODE_ATTRIBUTION_HEADER".into(), "0".into());

    if let Some(custom) = &profile.env {
        for (k, v) in custom {
            env.insert(k.clone(), expand_env_vars(v)?);
        }
    }

    Ok(env)
}

/// Construit les variables d'environnement pour un profil Multi.
/// `profile_name` est le nom court du profil ; les modèles sont référencés
/// sous la forme `<profile_name>/<alias>` (correspond au yaml LiteLLM généré).
pub fn build_env_vars_multi(
    profile_name: &str,
    profile: &MultiProfile,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    // Vérifie que le modèle par défaut existe.
    if !profile.models.contains_key(&profile.default) {
        return Err(format!(
            "default model '{}' not found in profile.models",
            profile.default
        )
        .into());
    }

    let master_key = crate::proxy::keychain::get_master_key()
        .map_err(|e| format!("master_key Keychain : {e}. Lance `lcc proxy install`."))?;

    let mut env = HashMap::new();
    env.insert(
        "ANTHROPIC_BASE_URL".into(),
        format!("http://localhost:{DEFAULT_PORT}"),
    );
    env.insert("ANTHROPIC_AUTH_TOKEN".into(), master_key);
    // ANTHROPIC_MODEL = <profile_name>/<default_alias>
    env.insert(
        "ANTHROPIC_MODEL".into(),
        format!("{}/{}", profile_name, profile.default),
    );
    env.insert("CLAUDE_CODE_ATTRIBUTION_HEADER".into(), "0".into());

    let mut custom_set: Option<String> = None;
    for (alias, entry) in &profile.models {
        let Some(slot) = entry.slot else { continue };
        let composite = format!("{}/{}", profile_name, alias);
        match slot {
            Slot::Opus => {
                env.insert("ANTHROPIC_DEFAULT_OPUS_MODEL".into(), composite);
            }
            Slot::Sonnet => {
                env.insert("ANTHROPIC_DEFAULT_SONNET_MODEL".into(), composite);
            }
            Slot::Haiku => {
                env.insert("ANTHROPIC_DEFAULT_HAIKU_MODEL".into(), composite);
            }
            Slot::Custom => {
                if let Some(prev) = &custom_set {
                    return Err(format!(
                        "two models claim slot 'custom': '{prev}' and '{alias}'. Only one is allowed."
                    )
                    .into());
                }
                custom_set = Some(alias.clone());
            }
        }
    }

    if let Some(custom) = &profile.env {
        for (k, v) in custom {
            env.insert(k.clone(), expand_env_vars(v)?);
        }
    }

    Ok(env)
}

pub fn build_claude_args(model: &str, extra: &[String]) -> Vec<String> {
    let mut args = vec!["--model".to_string(), model.to_string()];
    args.extend(extra.iter().cloned());
    args
}

pub fn run(profil: &str, extra_args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if !health::is_alive(DEFAULT_PORT, Duration::from_secs(1)) {
        return Err(format!(
            "✗ Le proxy LiteLLM ne répond pas sur :{DEFAULT_PORT}.\n  → Lance: lcc proxy doctor\n  → Ou:   lcc proxy start"
        )
        .into());
    }

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

    let (env_vars, model) = match profile {
        Profile::Single(p) => {
            let env = build_env_vars_single(profil, p)?;
            (env, profil.to_string())
        }
        Profile::Multi(p) => {
            let env = build_env_vars_multi(profil, p)?;
            let model = format!("{}/{}", profil, p.default);
            (env, model)
        }
    };

    let args = build_claude_args(&model, extra_args);

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
