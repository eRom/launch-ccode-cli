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

fn insert_auth(
    env: &mut HashMap<String, String>,
    api_key: &str,
    auth_token: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = expand_env_vars(api_key)?;
    let auth_token = expand_env_vars(auth_token)?;
    if !api_key.is_empty() {
        env.insert("ANTHROPIC_API_KEY".into(), api_key);
    } else if !auth_token.is_empty() {
        env.insert("ANTHROPIC_AUTH_TOKEN".into(), auth_token);
    }
    Ok(())
}

pub fn build_env_vars_single(
    profile: &SingleProfile,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let model = expand_env_vars(&profile.model)?;
    let mut env = HashMap::new();
    env.insert("ANTHROPIC_BASE_URL".into(), expand_env_vars(&profile.base_url)?);
    insert_auth(&mut env, &profile.api_key, &profile.auth_token)?;
    env.insert("ANTHROPIC_MODEL".into(), model.clone());
    env.insert("ANTHROPIC_DEFAULT_OPUS_MODEL".into(), model.clone());
    env.insert("ANTHROPIC_DEFAULT_SONNET_MODEL".into(), model.clone());
    env.insert("ANTHROPIC_DEFAULT_HAIKU_MODEL".into(), model.clone());
    env.insert("ANTHROPIC_SMALL_FAST_MODEL".into(), model.clone());
    env.insert("CLAUDE_CODE_SUBAGENT_MODEL".into(), model);
    env.insert("CLAUDE_CODE_ATTRIBUTION_HEADER".into(), "0".into());

    if let Some(custom) = &profile.env {
        for (k, v) in custom {
            env.insert(k.clone(), expand_env_vars(v)?);
        }
    }

    Ok(env)
}

pub fn build_env_vars_multi(
    profile: &MultiProfile,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let default_entry = profile.models.get(&profile.default).ok_or_else(|| {
        format!(
            "default model '{}' not found in profile.models",
            profile.default
        )
    })?;
    let default_id = expand_env_vars(&default_entry.id)?;

    let mut env = HashMap::new();
    env.insert("ANTHROPIC_BASE_URL".into(), expand_env_vars(&profile.base_url)?);
    insert_auth(&mut env, &profile.api_key, &profile.auth_token)?;
    env.insert("ANTHROPIC_MODEL".into(), default_id.clone());
    env.insert("CLAUDE_CODE_SUBAGENT_MODEL".into(), default_id);
    env.insert("CLAUDE_CODE_ATTRIBUTION_HEADER".into(), "0".into());

    let mut custom_set: Option<String> = None;
    for (name, entry) in &profile.models {
        let Some(slot) = entry.slot else { continue };
        let id = expand_env_vars(&entry.id)?;
        match slot {
            Slot::Opus => {
                env.insert("ANTHROPIC_DEFAULT_OPUS_MODEL".into(), id);
            }
            Slot::Sonnet => {
                env.insert("ANTHROPIC_DEFAULT_SONNET_MODEL".into(), id);
            }
            Slot::Haiku => {
                env.insert("ANTHROPIC_DEFAULT_HAIKU_MODEL".into(), id.clone());
                env.insert("ANTHROPIC_SMALL_FAST_MODEL".into(), id);
            }
            Slot::Custom => {
                if let Some(prev) = &custom_set {
                    return Err(format!(
                        "two models claim slot 'custom': '{prev}' and '{name}'. Only one is allowed."
                    )
                    .into());
                }
                env.insert("ANTHROPIC_CUSTOM_MODEL_OPTION".into(), id);
                env.insert("ANTHROPIC_CUSTOM_MODEL_OPTION_NAME".into(), name.clone());
                if let Some(d) = &entry.description {
                    env.insert(
                        "ANTHROPIC_CUSTOM_MODEL_OPTION_DESCRIPTION".into(),
                        expand_env_vars(d)?,
                    );
                }
                custom_set = Some(name.clone());
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
            let env = build_env_vars_single(p)?;
            let m = expand_env_vars(&p.model)?;
            (env, m)
        }
        Profile::Multi(p) => {
            let env = build_env_vars_multi(p)?;
            let default_entry = p.models.get(&p.default).ok_or_else(|| {
                format!(
                    "default model '{}' not found in profile.models",
                    p.default
                )
            })?;
            let m = expand_env_vars(&default_entry.id)?;
            (env, m)
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
