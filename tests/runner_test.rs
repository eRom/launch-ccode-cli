#[test]
fn test_build_env_vars_basic() {
    let profile = lcc::config::Profile {
        model: "gemma4".to_string(),
        base_url: "http://localhost:11434/v1".to_string(),
        api_key: "".to_string(),
        auth_token: "ollama".to_string(),
        env: None,
    };

    let env = lcc::runner::build_env_vars(&profile);

    assert_eq!(env.get("ANTHROPIC_BASE_URL").unwrap(), "http://localhost:11434/v1");
    assert_eq!(env.get("ANTHROPIC_API_KEY").unwrap(), "");
    assert_eq!(env.get("ANTHROPIC_AUTH_TOKEN").unwrap(), "ollama");
    assert_eq!(env.get("ANTHROPIC_DEFAULT_OPUS_MODEL").unwrap(), "gemma4");
    assert_eq!(env.get("ANTHROPIC_DEFAULT_SONNET_MODEL").unwrap(), "gemma4");
    assert_eq!(env.get("ANTHROPIC_DEFAULT_HAIKU_MODEL").unwrap(), "gemma4");
    assert_eq!(env.get("CLAUDE_CODE_SUBAGENT_MODEL").unwrap(), "gemma4");
    assert_eq!(env.get("CLAUDE_CODE_ATTRIBUTION_HEADER").unwrap(), "0");
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

    let env = lcc::runner::build_env_vars(&profile);
    assert_eq!(env.get("MY_VAR").unwrap(), "hello");
    assert_eq!(env.get("ANTHROPIC_API_KEY").unwrap(), "sk-xxx");
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
