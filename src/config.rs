use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub profiles: HashMap<String, Profile>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Profile {
    Multi(MultiProfile),
    Single(SingleProfile),
}

#[derive(Debug, Deserialize)]
pub struct SingleProfile {
    pub model: String,
    pub base_url: String,
    pub api_key: String,
    pub auth_token: String,
    pub env: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct MultiProfile {
    pub base_url: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub auth_token: String,
    pub default: String,
    pub models: HashMap<String, ModelEntry>,
    pub env: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct ModelEntry {
    pub id: String,
    #[serde(default)]
    pub slot: Option<Slot>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Slot {
    Opus,
    Sonnet,
    Haiku,
    Custom,
}

pub fn expand_env_vars(s: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find("${") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let end = after
            .find('}')
            .ok_or_else(|| format!("Unclosed variable placeholder in: {s}"))?;
        let var_name = &after[..end];
        let value = std::env::var(var_name)
            .map_err(|_| format!("Environment variable not set: {var_name}"))?;
        out.push_str(&value);
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

pub fn settings_path() -> PathBuf {
    let home = dirs::home_dir().expect("cannot resolve home directory");
    home.join(".config").join("launch-claude-code").join("settings.json")
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
        match &settings.profiles[name] {
            Profile::Single(p) => {
                println!("{:<20} {:<30} {}", name, p.model, p.base_url);
            }
            Profile::Multi(p) => {
                let label = format!("multi: {} models (default: {})", p.models.len(), p.default);
                println!("{:<20} {:<30} {}", name, label, p.base_url);
            }
        }
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
