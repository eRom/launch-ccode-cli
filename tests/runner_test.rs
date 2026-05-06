use lcc::config::{ModelEntry, MultiProfile, SingleProfile, Slot};
use std::collections::HashMap;

#[test]
fn test_build_env_vars_basic() {
    let profile = SingleProfile {
        model: "gemma4".to_string(),
        base_url: "http://localhost:11434/v1".to_string(),
        api_key: "".to_string(),
        auth_token: "ollama".to_string(),
        env: None,
    };

    let env = lcc::runner::build_env_vars_single(&profile).unwrap();

    assert_eq!(env.get("ANTHROPIC_BASE_URL").unwrap(), "http://localhost:11434/v1");
    assert!(!env.contains_key("ANTHROPIC_API_KEY"));
    assert_eq!(env.get("ANTHROPIC_AUTH_TOKEN").unwrap(), "ollama");
    assert_eq!(env.get("ANTHROPIC_MODEL").unwrap(), "gemma4");
    assert_eq!(env.get("ANTHROPIC_DEFAULT_OPUS_MODEL").unwrap(), "gemma4");
    assert_eq!(env.get("ANTHROPIC_DEFAULT_SONNET_MODEL").unwrap(), "gemma4");
    assert_eq!(env.get("ANTHROPIC_DEFAULT_HAIKU_MODEL").unwrap(), "gemma4");
    assert_eq!(env.get("CLAUDE_CODE_SUBAGENT_MODEL").unwrap(), "gemma4");
    assert_eq!(env.get("CLAUDE_CODE_ATTRIBUTION_HEADER").unwrap(), "0");
}

#[test]
fn test_build_env_vars_expands_placeholders() {
    std::env::set_var("LCC_TEST_RUNNER_KEY", "sk-real-secret");
    let profile = SingleProfile {
        model: "deepseek/deepseek-v4-pro".to_string(),
        base_url: "https://openrouter.ai/api/v1".to_string(),
        api_key: "${LCC_TEST_RUNNER_KEY}".to_string(),
        auth_token: "openrouter".to_string(),
        env: None,
    };
    let env = lcc::runner::build_env_vars_single(&profile).unwrap();
    assert_eq!(env.get("ANTHROPIC_API_KEY").unwrap(), "sk-real-secret");
}

#[test]
fn test_build_env_vars_missing_var_errors() {
    let profile = SingleProfile {
        model: "x".to_string(),
        base_url: "x".to_string(),
        api_key: "${LCC_TEST_RUNNER_MISSING_QQQ}".to_string(),
        auth_token: "x".to_string(),
        env: None,
    };
    let result = lcc::runner::build_env_vars_single(&profile);
    assert!(result.is_err());
}

#[test]
fn test_build_env_vars_with_custom_env() {
    let mut custom = HashMap::new();
    custom.insert("MY_VAR".to_string(), "hello".to_string());

    let profile = SingleProfile {
        model: "test".to_string(),
        base_url: "https://api.example.com/v1".to_string(),
        api_key: "sk-xxx".to_string(),
        auth_token: "bearer".to_string(),
        env: Some(custom),
    };

    let env = lcc::runner::build_env_vars_single(&profile).unwrap();
    assert_eq!(env.get("MY_VAR").unwrap(), "hello");
    assert_eq!(env.get("ANTHROPIC_API_KEY").unwrap(), "sk-xxx");
    assert!(!env.contains_key("ANTHROPIC_AUTH_TOKEN"));
}

#[test]
fn test_build_env_vars_api_key_takes_precedence() {
    let profile = SingleProfile {
        model: "x".into(),
        base_url: "x".into(),
        api_key: "sk-real".into(),
        auth_token: "should-be-ignored".into(),
        env: None,
    };
    let env = lcc::runner::build_env_vars_single(&profile).unwrap();
    assert_eq!(env.get("ANTHROPIC_API_KEY").unwrap(), "sk-real");
    assert!(!env.contains_key("ANTHROPIC_AUTH_TOKEN"));
}

#[test]
fn test_build_env_vars_empty_api_key_uses_auth_token() {
    let profile = SingleProfile {
        model: "x".into(),
        base_url: "x".into(),
        api_key: "".into(),
        auth_token: "ollama".into(),
        env: None,
    };
    let env = lcc::runner::build_env_vars_single(&profile).unwrap();
    assert!(!env.contains_key("ANTHROPIC_API_KEY"));
    assert_eq!(env.get("ANTHROPIC_AUTH_TOKEN").unwrap(), "ollama");
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

fn entry(id: &str, slot: Option<Slot>, description: Option<&str>) -> ModelEntry {
    ModelEntry {
        id: id.to_string(),
        slot,
        description: description.map(String::from),
    }
}

#[test]
fn test_build_env_vars_multi_full() {
    let mut models = HashMap::new();
    models.insert("owl".to_string(), entry("openrouter/owl-alpha", Some(Slot::Opus), None));
    models.insert("dpro".to_string(), entry("deepseek/deepseek-v4-pro", Some(Slot::Sonnet), None));
    models.insert("dflash".to_string(), entry("deepseek/deepseek-v4-flash", Some(Slot::Haiku), None));
    models.insert("kimi".to_string(), entry("moonshotai/kimi-k2.6", Some(Slot::Custom), Some("Moonshot Kimi")));
    models.insert("gemma".to_string(), entry("google/gemma-4-26b-a4b-it", None, None));

    let profile = MultiProfile {
        base_url: "https://openrouter.ai/api".into(),
        api_key: "".into(),
        auth_token: "tok".into(),
        default: "dpro".into(),
        models,
        env: None,
    };

    let env = lcc::runner::build_env_vars_multi(&profile).unwrap();

    assert_eq!(env.get("ANTHROPIC_BASE_URL").unwrap(), "https://openrouter.ai/api");
    assert_eq!(env.get("ANTHROPIC_AUTH_TOKEN").unwrap(), "tok");
    assert_eq!(env.get("ANTHROPIC_MODEL").unwrap(), "deepseek/deepseek-v4-pro");
    assert_eq!(env.get("CLAUDE_CODE_SUBAGENT_MODEL").unwrap(), "deepseek/deepseek-v4-pro");
    assert_eq!(env.get("ANTHROPIC_DEFAULT_OPUS_MODEL").unwrap(), "openrouter/owl-alpha");
    assert_eq!(env.get("ANTHROPIC_DEFAULT_SONNET_MODEL").unwrap(), "deepseek/deepseek-v4-pro");
    assert_eq!(env.get("ANTHROPIC_DEFAULT_HAIKU_MODEL").unwrap(), "deepseek/deepseek-v4-flash");
    assert_eq!(env.get("ANTHROPIC_CUSTOM_MODEL_OPTION").unwrap(), "moonshotai/kimi-k2.6");
    assert_eq!(env.get("ANTHROPIC_CUSTOM_MODEL_OPTION_NAME").unwrap(), "kimi");
    assert_eq!(env.get("ANTHROPIC_CUSTOM_MODEL_OPTION_DESCRIPTION").unwrap(), "Moonshot Kimi");
}

#[test]
fn test_build_env_vars_multi_default_missing() {
    let mut models = HashMap::new();
    models.insert("a".to_string(), entry("provider/a", None, None));
    let profile = MultiProfile {
        base_url: "https://x".into(),
        api_key: "".into(),
        auth_token: "t".into(),
        default: "nonexistent".into(),
        models,
        env: None,
    };
    let result = lcc::runner::build_env_vars_multi(&profile);
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("nonexistent"));
}

#[test]
fn test_build_env_vars_multi_two_custom_errors() {
    let mut models = HashMap::new();
    models.insert("a".to_string(), entry("provider/a", Some(Slot::Custom), None));
    models.insert("b".to_string(), entry("provider/b", Some(Slot::Custom), None));
    let profile = MultiProfile {
        base_url: "https://x".into(),
        api_key: "".into(),
        auth_token: "t".into(),
        default: "a".into(),
        models,
        env: None,
    };
    let result = lcc::runner::build_env_vars_multi(&profile);
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("custom"));
}

#[test]
fn test_build_env_vars_multi_no_slot_does_not_set_picker_vars() {
    let mut models = HashMap::new();
    models.insert("a".to_string(), entry("provider/a", None, None));
    models.insert("b".to_string(), entry("provider/b", None, None));
    let profile = MultiProfile {
        base_url: "https://x".into(),
        api_key: "".into(),
        auth_token: "t".into(),
        default: "a".into(),
        models,
        env: None,
    };
    let env = lcc::runner::build_env_vars_multi(&profile).unwrap();
    assert!(!env.contains_key("ANTHROPIC_DEFAULT_OPUS_MODEL"));
    assert!(!env.contains_key("ANTHROPIC_DEFAULT_SONNET_MODEL"));
    assert!(!env.contains_key("ANTHROPIC_DEFAULT_HAIKU_MODEL"));
    assert!(!env.contains_key("ANTHROPIC_CUSTOM_MODEL_OPTION"));
    assert_eq!(env.get("ANTHROPIC_MODEL").unwrap(), "provider/a");
}

#[test]
fn test_build_env_vars_multi_id_expansion() {
    std::env::set_var("LCC_TEST_MULTI_PROVIDER", "deepseek");
    let mut models = HashMap::new();
    models.insert("p".to_string(), entry("${LCC_TEST_MULTI_PROVIDER}/v4-pro", Some(Slot::Sonnet), None));
    let profile = MultiProfile {
        base_url: "https://x".into(),
        api_key: "".into(),
        auth_token: "t".into(),
        default: "p".into(),
        models,
        env: None,
    };
    let env = lcc::runner::build_env_vars_multi(&profile).unwrap();
    assert_eq!(env.get("ANTHROPIC_MODEL").unwrap(), "deepseek/v4-pro");
    assert_eq!(env.get("ANTHROPIC_DEFAULT_SONNET_MODEL").unwrap(), "deepseek/v4-pro");
}
