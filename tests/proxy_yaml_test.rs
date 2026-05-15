use lcc::config::{ModelEntry, MultiProfile, Profile, Settings, SingleProfile};
use lcc::proxy::generators::yaml::generate_litellm_yaml;
use std::collections::HashMap;

fn single_profile(model: &str, base_url: &str, auth_token: &str) -> Profile {
    Profile::Single(SingleProfile {
        model: model.to_string(),
        base_url: base_url.to_string(),
        api_key: String::new(),
        auth_token: auth_token.to_string(),
        env: None,
    })
}

#[test]
fn single_openrouter_profile_generates_correct_yaml() {
    let mut profiles = HashMap::new();
    profiles.insert(
        "qwen".to_string(),
        single_profile(
            "qwen/qwen3.6-plus",
            "https://openrouter.ai/api/v1",
            "${OPENROUTER_API_KEY}",
        ),
    );
    let settings = Settings { profiles };

    let yaml = generate_litellm_yaml(&settings);

    // Parse both sides + compare as YAML values (avoids ordering / spacing brittleness)
    let got: serde_yaml::Value = serde_yaml::from_str(&yaml).expect("yaml parses");

    let expected_str = r#"model_list:
- model_name: qwen
  litellm_params:
    model: openrouter/qwen/qwen3.6-plus
    api_key: os.environ/OPENROUTER_API_KEY
    drop_params: true
litellm_settings:
  drop_params: true
  set_verbose: false
  callbacks:
  - lcc_strip_thinking
general_settings:
  master_key: os.environ/LCC_MASTER_KEY
  database_url: null
  store_model_in_db: false
"#;
    let expected: serde_yaml::Value = serde_yaml::from_str(expected_str).unwrap();
    assert_eq!(got, expected);
}

#[test]
fn multi_profile_expands_each_model() {
    let mut models = HashMap::new();
    models.insert(
        "fast".to_string(),
        ModelEntry {
            id: "qwen/qwen3.6-flash".to_string(),
            slot: None,
            description: Some("modèle rapide".to_string()),
        },
    );
    models.insert(
        "smart".to_string(),
        ModelEntry {
            id: "qwen/qwen3.6-plus".to_string(),
            slot: None,
            description: None,
        },
    );

    let mp = MultiProfile {
        base_url: "https://openrouter.ai/api/v1".to_string(),
        api_key: String::new(),
        auth_token: "${OPENROUTER_API_KEY}".to_string(),
        default: "smart".to_string(),
        models,
        env: None,
    };

    let mut profiles = HashMap::new();
    profiles.insert("openrouter-multi".to_string(), Profile::Multi(mp));
    let settings = Settings { profiles };

    let yaml = generate_litellm_yaml(&settings);
    // 1 entry per ModelEntry, named "<profile>/<alias>"
    assert!(
        yaml.contains("model_name: openrouter-multi/fast"),
        "yaml should contain composite name 'openrouter-multi/fast'\n{yaml}"
    );
    assert!(
        yaml.contains("model: openrouter/qwen/qwen3.6-flash"),
        "yaml should contain mapped model id\n{yaml}"
    );
    assert!(
        yaml.contains("model_name: openrouter-multi/smart"),
        "yaml should contain composite name 'openrouter-multi/smart'\n{yaml}"
    );
    assert!(
        yaml.contains("model: openrouter/qwen/qwen3.6-plus"),
        "yaml should contain mapped model id\n{yaml}"
    );
    assert!(
        yaml.contains("api_key: os.environ/OPENROUTER_API_KEY"),
        "yaml should contain api_key env ref\n{yaml}"
    );
}
