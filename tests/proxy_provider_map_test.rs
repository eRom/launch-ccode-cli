use lcc::proxy::provider_map::detect_litellm_prefix;

#[test]
fn openrouter_detected() {
    assert_eq!(
        detect_litellm_prefix("https://openrouter.ai/api/v1"),
        Some("openrouter")
    );
}

#[test]
fn deepseek_detected() {
    assert_eq!(
        detect_litellm_prefix("https://api.deepseek.com/v1"),
        Some("deepseek")
    );
}

#[test]
fn groq_detected() {
    assert_eq!(
        detect_litellm_prefix("https://api.groq.com/openai/v1"),
        Some("groq")
    );
}

#[test]
fn together_detected() {
    assert_eq!(
        detect_litellm_prefix("https://api.together.xyz/v1"),
        Some("together_ai")
    );
}

#[test]
fn mistral_detected() {
    assert_eq!(
        detect_litellm_prefix("https://api.mistral.ai/v1"),
        Some("mistral")
    );
}

#[test]
fn unknown_returns_none() {
    assert_eq!(
        detect_litellm_prefix("https://example.com/v1"),
        None
    );
}

#[test]
fn http_scheme_works() {
    assert_eq!(
        detect_litellm_prefix("http://openrouter.ai/api/v1"),
        Some("openrouter")
    );
}
