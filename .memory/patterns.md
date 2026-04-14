# Patterns — lcc
> Dernière mise à jour : 2026-04-14

## Architecture
- **Module par responsabilité** : config (données) / runner (exécution) / main (dispatch)
- **Lib + bin** : lib.rs expose les modules pour les tests d'intégration, main.rs est le binaire

## Conventions de code
- Error handling : `Result<(), Box<dyn std::error::Error>>` partout
- Pas de struct d'erreur custom (projet simple)
- Clap derive pour le CLI parsing
- Serde Deserialize pour la config (pas de Serialize — lecture seule)

## Tests
- Framework : `cargo test` natif
- Organisation : `tests/` (integration tests) accédant via `lcc::module::fn`
- Pattern TDD suivi pendant le développement
- Pas de mocks — tests purs sur les fonctions utilitaires (build_env_vars, build_claude_args, serde parsing)

## Env vars injectées
Les env vars sont hardcodées dans `runner::build_env_vars()` :
- ANTHROPIC_BASE_URL, ANTHROPIC_API_KEY, ANTHROPIC_AUTH_TOKEN
- ANTHROPIC_DEFAULT_{OPUS,SONNET,HAIKU}_MODEL (tous = profile.model)
- CLAUDE_CODE_SUBAGENT_MODEL
- CLAUDE_CODE_ATTRIBUTION_HEADER = "0" (hardcodé)
- + profile.env custom
