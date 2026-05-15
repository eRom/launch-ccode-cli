//! Détection du préfixe LiteLLM à partir d'un base_url provider.
//!
//! LiteLLM identifie chaque provider par un préfixe sur le model id
//! (ex: `openrouter/qwen/qwen3.6-plus`). On déduit ce préfixe à partir
//! du `base_url` du profil lcc.

const PROVIDER_MAP: &[(&str, &str)] = &[
    ("openrouter.ai", "openrouter"),
    ("api.deepseek.com", "deepseek"),
    ("api.groq.com", "groq"),
    ("api.together.xyz", "together_ai"),
    ("api.mistral.ai", "mistral"),
    // Ajout : étendre cette table au fur et à mesure des besoins.
];

/// Retourne le préfixe LiteLLM correspondant au host du `base_url`,
/// ou `None` si non reconnu (le caller fallbackra sur `openai/`).
pub fn detect_litellm_prefix(base_url: &str) -> Option<&'static str> {
    let url = url_strip_scheme(base_url);
    for (host, prefix) in PROVIDER_MAP {
        if url.starts_with(host) {
            return Some(prefix);
        }
    }
    None
}

fn url_strip_scheme(url: &str) -> &str {
    url.trim_start_matches("https://")
        .trim_start_matches("http://")
}
