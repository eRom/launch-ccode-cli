//! Génère le `litellm.yaml` à partir des profils `lcc`.

use crate::config::{Profile, Settings};
use crate::proxy::provider_map::detect_litellm_prefix;
use serde_yaml::{Mapping, Sequence, Value};

/// Génère le contenu YAML à écrire dans `~/.config/launch-claude-code/litellm.yaml`.
pub fn generate_litellm_yaml(settings: &Settings) -> String {
    let mut model_list = Sequence::new();

    // Sort profile keys for deterministic output
    let mut profile_keys: Vec<&String> = settings.profiles.keys().collect();
    profile_keys.sort();

    for profile_name in profile_keys {
        match &settings.profiles[profile_name] {
            Profile::Single(sp) => {
                let auth = pick_auth_str(&sp.auth_token, &sp.api_key);
                model_list.push(build_model_entry(
                    profile_name,
                    &sp.model,
                    &sp.base_url,
                    auth,
                ));
            }
            Profile::Multi(mp) => {
                let auth = pick_auth_str(&mp.auth_token, &mp.api_key);
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
    if let Some(auth_str) = auth {
        if let Some(env_ref) = extract_env_var(auth_str) {
            params.insert(
                Value::from("api_key"),
                Value::from(format!("os.environ/{env_ref}")),
            );
        }
    }
    params.insert(Value::from("drop_params"), Value::from(true));

    let mut entry = Mapping::new();
    entry.insert(Value::from("model_name"), Value::from(name));
    entry.insert(Value::from("litellm_params"), Value::Mapping(params));
    Value::Mapping(entry)
}

/// Choisit l'auth à utiliser pour LiteLLM : `auth_token` en priorité,
/// `api_key` en fallback. Fonctionne avec des String (même si vides).
fn pick_auth_str<'a>(auth_token: &'a str, api_key: &'a str) -> Option<&'a str> {
    if !auth_token.is_empty() {
        Some(auth_token)
    } else if !api_key.is_empty() {
        Some(api_key)
    } else {
        None
    }
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
