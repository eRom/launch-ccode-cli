use lcc::config::{ModelEntry, MultiProfile, SingleProfile, Slot};
use std::collections::HashMap;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn entry(id: &str, slot: Option<Slot>, description: Option<&str>) -> ModelEntry {
    ModelEntry {
        id: id.to_string(),
        slot,
        description: description.map(String::from),
    }
}

fn proxy_url() -> String {
    format!("http://localhost:{}", lcc::proxy::DEFAULT_PORT)
}

// ── Single-profile tests ──────────────────────────────────────────────────────

/// Vérifie que les env vars pointent vers le proxy local et utilisent
/// le nom du profil comme ANTHROPIC_MODEL (pas l'id provider brut).
#[cfg(feature = "mock-keychain")]
#[test]
fn single_profile_env_points_to_proxy() {
    let profile = SingleProfile {
        model: "gemma4".to_string(),
        base_url: "http://localhost:11434/v1".to_string(),
        api_key: "".to_string(),
        auth_token: "ollama".to_string(),
        env: None,
    };

    let env = lcc::runner::build_env_vars_single("qwen", &profile).unwrap();

    assert_eq!(env.get("ANTHROPIC_BASE_URL").unwrap(), &proxy_url());
    assert_eq!(env.get("ANTHROPIC_AUTH_TOKEN").unwrap(), "mock-master-key");
    assert_eq!(env.get("ANTHROPIC_MODEL").unwrap(), "qwen");
    assert_eq!(env.get("ANTHROPIC_DEFAULT_OPUS_MODEL").unwrap(), "qwen");
    assert_eq!(env.get("ANTHROPIC_DEFAULT_SONNET_MODEL").unwrap(), "qwen");
    assert_eq!(env.get("ANTHROPIC_DEFAULT_HAIKU_MODEL").unwrap(), "qwen");
    assert_eq!(env.get("CLAUDE_CODE_ATTRIBUTION_HEADER").unwrap(), "0");

    // Les champs provider (api_key, auth_token) ne doivent PAS être exposés.
    assert!(!env.contains_key("ANTHROPIC_API_KEY"));
    // L'ANTHROPIC_AUTH_TOKEN est la master_key du proxy, pas la valeur "ollama" du profil.
    assert_ne!(env.get("ANTHROPIC_AUTH_TOKEN").unwrap(), "ollama");
}

/// Les vars custom `env:` du profil sont transmises à claude.
#[cfg(feature = "mock-keychain")]
#[test]
fn single_profile_custom_env_forwarded() {
    let mut custom = HashMap::new();
    custom.insert("MY_VAR".to_string(), "hello".to_string());

    let profile = SingleProfile {
        model: "test".to_string(),
        base_url: "https://api.example.com/v1".to_string(),
        api_key: "sk-xxx".to_string(),
        auth_token: "bearer".to_string(),
        env: Some(custom),
    };

    let env = lcc::runner::build_env_vars_single("myprofil", &profile).unwrap();
    assert_eq!(env.get("MY_VAR").unwrap(), "hello");
    // Proxy auth prend le dessus — pas la clé provider.
    assert_eq!(env.get("ANTHROPIC_AUTH_TOKEN").unwrap(), "mock-master-key");
    assert!(!env.contains_key("ANTHROPIC_API_KEY"));
}

/// La var custom peut utiliser ${VAR} expansion.
#[cfg(feature = "mock-keychain")]
#[test]
fn single_profile_custom_env_var_expansion() {
    // TODO: Audit that the environment access only happens in single-threaded code.
    unsafe { std::env::set_var("LCC_TEST_CUSTOM_ENV_VALUE", "expanded-value") };

    let mut custom = HashMap::new();
    custom.insert("MY_KEY".to_string(), "${LCC_TEST_CUSTOM_ENV_VALUE}".to_string());

    let profile = SingleProfile {
        model: "test".to_string(),
        base_url: "https://api.example.com/v1".to_string(),
        api_key: "".to_string(),
        auth_token: "".to_string(),
        env: Some(custom),
    };

    let env = lcc::runner::build_env_vars_single("p", &profile).unwrap();
    assert_eq!(env.get("MY_KEY").unwrap(), "expanded-value");
}

// ── build_claude_args tests (inchangés) ───────────────────────────────────────

#[test]
fn test_build_claude_args() {
    let extra = vec![
        "--dangerously-skip-permissions".to_string(),
        "-p".to_string(),
        "hello".to_string(),
    ];
    let args = lcc::runner::build_claude_args("gemma4", &extra);
    assert_eq!(
        args,
        vec![
            "--model",
            "gemma4",
            "--dangerously-skip-permissions",
            "-p",
            "hello"
        ]
    );
}

#[test]
fn test_build_claude_args_no_extra() {
    let args = lcc::runner::build_claude_args("gemma4", &[]);
    assert_eq!(args, vec!["--model", "gemma4"]);
}

// ── Multi-profile tests ───────────────────────────────────────────────────────

/// Vérifie le routage proxy et les noms composites `<profile>/<alias>`.
#[cfg(feature = "mock-keychain")]
#[test]
fn multi_profile_env_points_to_proxy_with_composite_names() {
    let mut models = HashMap::new();
    models.insert(
        "owl".to_string(),
        entry("openrouter/owl-alpha", Some(Slot::Opus), None),
    );
    models.insert(
        "dpro".to_string(),
        entry("deepseek/deepseek-v4-pro", Some(Slot::Sonnet), None),
    );
    models.insert(
        "dflash".to_string(),
        entry("deepseek/deepseek-v4-flash", Some(Slot::Haiku), None),
    );
    models.insert(
        "kimi".to_string(),
        entry("moonshotai/kimi-k2.6", Some(Slot::Custom), Some("Moonshot Kimi")),
    );
    models.insert(
        "gemma".to_string(),
        entry("google/gemma-4-26b-a4b-it", None, None),
    );

    let profile = MultiProfile {
        base_url: "https://openrouter.ai/api".into(),
        api_key: "".into(),
        auth_token: "tok".into(),
        default: "dpro".into(),
        models,
        env: None,
    };

    let env = lcc::runner::build_env_vars_multi("myprofile", &profile).unwrap();

    assert_eq!(env.get("ANTHROPIC_BASE_URL").unwrap(), &proxy_url());
    assert_eq!(env.get("ANTHROPIC_AUTH_TOKEN").unwrap(), "mock-master-key");
    // Modèle par défaut en composite.
    assert_eq!(env.get("ANTHROPIC_MODEL").unwrap(), "myprofile/dpro");
    assert_eq!(
        env.get("ANTHROPIC_DEFAULT_OPUS_MODEL").unwrap(),
        "myprofile/owl"
    );
    assert_eq!(
        env.get("ANTHROPIC_DEFAULT_SONNET_MODEL").unwrap(),
        "myprofile/dpro"
    );
    assert_eq!(
        env.get("ANTHROPIC_DEFAULT_HAIKU_MODEL").unwrap(),
        "myprofile/dflash"
    );
    assert_eq!(env.get("CLAUDE_CODE_ATTRIBUTION_HEADER").unwrap(), "0");

    // Les champs provider ne doivent pas fuiter.
    assert_ne!(env.get("ANTHROPIC_AUTH_TOKEN").unwrap(), "tok");
}

/// Le modèle par défaut manquant doit retourner une erreur lisible.
#[cfg(feature = "mock-keychain")]
#[test]
fn multi_profile_default_missing_errors() {
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
    let result = lcc::runner::build_env_vars_multi("p", &profile);
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("nonexistent"));
}

/// Deux modèles sur le slot 'custom' = erreur.
#[cfg(feature = "mock-keychain")]
#[test]
fn multi_profile_two_custom_errors() {
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
    let result = lcc::runner::build_env_vars_multi("p", &profile);
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("custom"));
}

/// Les modèles sans slot ne doivent pas setter les vars DEFAULT_*.
#[cfg(feature = "mock-keychain")]
#[test]
fn multi_profile_no_slot_does_not_set_picker_vars() {
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
    let env = lcc::runner::build_env_vars_multi("p", &profile).unwrap();
    assert!(!env.contains_key("ANTHROPIC_DEFAULT_OPUS_MODEL"));
    assert!(!env.contains_key("ANTHROPIC_DEFAULT_SONNET_MODEL"));
    assert!(!env.contains_key("ANTHROPIC_DEFAULT_HAIKU_MODEL"));
    assert!(!env.contains_key("ANTHROPIC_CUSTOM_MODEL_OPTION"));
    // Le modèle par défaut reste en composite même sans slot.
    assert_eq!(env.get("ANTHROPIC_MODEL").unwrap(), "p/a");
}
