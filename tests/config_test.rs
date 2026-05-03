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
