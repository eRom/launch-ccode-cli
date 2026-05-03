use crate::config::{expand_env_vars, load_settings, Profile};
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

pub fn build_env_vars(
    profile: &Profile,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let model = expand_env_vars(&profile.model)?;
    let api_key = expand_env_vars(&profile.api_key)?;
    let auth_token = expand_env_vars(&profile.auth_token)?;
    let mut env = HashMap::new();
    env.insert("ANTHROPIC_BASE_URL".into(), expand_env_vars(&profile.base_url)?);
    if !api_key.is_empty() {
        env.insert("ANTHROPIC_API_KEY".into(), api_key);
    } else if !auth_token.is_empty() {
        env.insert("ANTHROPIC_AUTH_TOKEN".into(), auth_token);
    }
    env.insert("ANTHROPIC_MODEL".into(), model.clone());
    env.insert("ANTHROPIC_DEFAULT_OPUS_MODEL".into(), model.clone());
    env.insert("ANTHROPIC_DEFAULT_SONNET_MODEL".into(), model.clone());
    env.insert("ANTHROPIC_DEFAULT_HAIKU_MODEL".into(), model.clone());
    env.insert("CLAUDE_CODE_SUBAGENT_MODEL".into(), model);
    env.insert("CLAUDE_CODE_ATTRIBUTION_HEADER".into(), "0".into());

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
    let env_vars = build_env_vars(profile)?;
    let model = expand_env_vars(&profile.model)?;
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
