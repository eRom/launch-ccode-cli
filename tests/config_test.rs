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

#[test]
fn test_expand_env_vars_simple() {
    std::env::set_var("LCC_TEST_EXPAND_SIMPLE", "hello");
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
    std::env::set_var("LCC_TEST_EXPAND_PARTIAL", "secret");
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
