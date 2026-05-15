use lcc::config::Profile;

#[test]
fn test_parse_valid_settings_single() {
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
    let Profile::Single(p) = settings.profiles.get("gemma4").unwrap() else {
        panic!("expected Single variant");
    };
    assert_eq!(p.model, "gemma4");
    assert_eq!(p.base_url, "http://localhost:11434/v1");
    assert_eq!(p.api_key, "");
    assert_eq!(p.auth_token, "ollama");
    assert!(p.env.is_none());
}

#[test]
fn test_parse_single_profile_minimal_fields() {
    // api_key et auth_token sont optionnels (defaut chaine vide) — cf v0.2.1.
    let json = r#"{
        "profiles": {
            "minimal": {
                "model": "google/gemma-4-31b-it:free",
                "base_url": "https://openrouter.ai/api"
            }
        }
    }"#;

    let settings: lcc::config::Settings = serde_json::from_str(json).unwrap();
    let Profile::Single(p) = settings.profiles.get("minimal").unwrap() else {
        panic!("expected Single variant");
    };
    assert_eq!(p.model, "google/gemma-4-31b-it:free");
    assert_eq!(p.api_key, "");
    assert_eq!(p.auth_token, "");
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
    let Profile::Single(p) = settings.profiles.get("test").unwrap() else {
        panic!("expected Single variant");
    };
    let env = p.env.as_ref().unwrap();
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
fn test_parse_get_profile_not_found() {
    let json = r#"{"profiles": {}}"#;
    let settings: lcc::config::Settings = serde_json::from_str(json).unwrap();
    assert!(settings.profiles.get("nope").is_none());
}

#[test]
fn test_parse_valid_settings_multi() {
    let json = r#"{
        "profiles": {
            "openrouter": {
                "base_url": "https://openrouter.ai/api",
                "auth_token": "${OPENROUTER_API_KEY}",
                "default": "deepseek-pro",
                "models": {
                    "owl": {
                        "id": "openrouter/owl-alpha",
                        "slot": "opus"
                    },
                    "deepseek-pro": {
                        "id": "deepseek/deepseek-v4-pro",
                        "slot": "sonnet"
                    },
                    "deepseek-flash": {
                        "id": "deepseek/deepseek-v4-flash",
                        "slot": "haiku"
                    },
                    "kimi": {
                        "id": "moonshotai/kimi-k2.6",
                        "slot": "custom",
                        "description": "Moonshot Kimi"
                    },
                    "gemma": {
                        "id": "google/gemma-4-26b-a4b-it"
                    }
                }
            }
        }
    }"#;

    let settings: lcc::config::Settings = serde_json::from_str(json).unwrap();
    let Profile::Multi(p) = settings.profiles.get("openrouter").unwrap() else {
        panic!("expected Multi variant");
    };
    assert_eq!(p.base_url, "https://openrouter.ai/api");
    assert_eq!(p.default, "deepseek-pro");
    assert_eq!(p.models.len(), 5);
    assert_eq!(p.models.get("kimi").unwrap().description.as_deref(), Some("Moonshot Kimi"));
    assert!(p.models.get("gemma").unwrap().slot.is_none());
}

#[test]
fn test_expand_env_vars_simple() {
    // TODO: Audit that the environment access only happens in single-threaded code.
    unsafe { std::env::set_var("LCC_TEST_EXPAND_SIMPLE", "hello") };
    let s = lcc::config::expand_env_vars("${LCC_TEST_EXPAND_SIMPLE}").unwrap();
    assert_eq!(s, "hello");
}

#[test]
fn test_expand_env_vars_no_placeholder() {
    let s = lcc::config::expand_env_vars("plain text").unwrap();
    assert_eq!(s, "plain text");
}

#[test]
fn test_expand_env_vars_partial() {
    // TODO: Audit that the environment access only happens in single-threaded code.
    unsafe { std::env::set_var("LCC_TEST_EXPAND_PARTIAL", "secret") };
    let s = lcc::config::expand_env_vars("Bearer ${LCC_TEST_EXPAND_PARTIAL}").unwrap();
    assert_eq!(s, "Bearer secret");
}

#[test]
fn test_expand_env_vars_missing() {
    let result = lcc::config::expand_env_vars("${LCC_TEST_NEVER_DEFINED_XYZ}");
    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(msg.contains("LCC_TEST_NEVER_DEFINED_XYZ"));
}

#[test]
fn test_expand_env_vars_unclosed() {
    let result = lcc::config::expand_env_vars("${UNCLOSED");
    assert!(result.is_err());
}

#[test]
fn test_expand_env_vars_empty() {
    let s = lcc::config::expand_env_vars("").unwrap();
    assert_eq!(s, "");
}
