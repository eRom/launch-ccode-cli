#[test]
fn test_build_env_vars_basic() {
    let profile = lcc::config::Profile {
        model: "gemma4".to_string(),
        base_url: "http://localhost:11434/v1".to_string(),
        api_key: "".to_string(),
        auth_token: "ollama".to_string(),
        env: None,
    };

    let env = lcc::runner::build_env_vars(&profile).unwrap();

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
    let profile = lcc::config::Profile {
        model: "deepseek/deepseek-v4-pro".to_string(),
        base_url: "https://openrouter.ai/api/v1".to_string(),
        api_key: "${LCC_TEST_RUNNER_KEY}".to_string(),
        auth_token: "openrouter".to_string(),
        env: None,
    };
    let env = lcc::runner::build_env_vars(&profile).unwrap();
    assert_eq!(env.get("ANTHROPIC_API_KEY").unwrap(), "sk-real-secret");
}

#[test]
fn test_build_env_vars_missing_var_errors() {
    let profile = lcc::config::Profile {
        model: "x".to_string(),
        base_url: "x".to_string(),
        api_key: "${LCC_TEST_RUNNER_MISSING_QQQ}".to_string(),
        auth_token: "x".to_string(),
        env: None,
    };
    let result = lcc::runner::build_env_vars(&profile);
    assert!(result.is_err());
}

#[test]
fn test_build_env_vars_with_custom_env() {
    let mut custom = std::collections::HashMap::new();
    custom.insert("MY_VAR".to_string(), "hello".to_string());

    let profile = lcc::config::Profile {
        model: "test".to_string(),
        base_url: "https://api.example.com/v1".to_string(),
        api_key: "sk-xxx".to_string(),
        auth_token: "bearer".to_string(),
        env: Some(custom),
    };

    let env = lcc::runner::build_env_vars(&profile).unwrap();
    assert_eq!(env.get("MY_VAR").unwrap(), "hello");
    assert_eq!(env.get("ANTHROPIC_API_KEY").unwrap(), "sk-xxx");
    assert!(!env.contains_key("ANTHROPIC_AUTH_TOKEN"));
}

#[test]
fn test_build_env_vars_api_key_takes_precedence() {
    let profile = lcc::config::Profile {
        model: "x".into(),
        base_url: "x".into(),
        api_key: "sk-real".into(),
        auth_token: "should-be-ignored".into(),
        env: None,
    };
    let env = lcc::runner::build_env_vars(&profile).unwrap();
    assert_eq!(env.get("ANTHROPIC_API_KEY").unwrap(), "sk-real");
    assert!(!env.contains_key("ANTHROPIC_AUTH_TOKEN"));
}

#[test]
fn test_build_env_vars_empty_api_key_uses_auth_token() {
    let profile = lcc::config::Profile {
        model: "x".into(),
        base_url: "x".into(),
        api_key: "".into(),
        auth_token: "ollama".into(),
        env: None,
    };
    let env = lcc::runner::build_env_vars(&profile).unwrap();
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
