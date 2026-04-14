# Fichiers clés — lcc
> Dernière mise à jour : 2026-04-14

## Core
| Fichier | Rôle |
|---------|------|
| `src/main.rs` | Point d'entrée CLI clap — 3 commandes : start, list, settings |
| `src/config.rs` | Types Settings/Profile, load_settings(), list_profiles(), settings_command() |
| `src/runner.rs` | find_claude(), build_env_vars(), build_claude_args(), run() |
| `src/lib.rs` | Expose pub mod config + runner pour les tests d'intégration |

## Tests
| Fichier | Rôle |
|---------|------|
| `tests/config_test.rs` | 4 tests : parsing valide, env custom, champ manquant, profil absent |
| `tests/runner_test.rs` | 4 tests : env vars basique, env custom, args avec/sans extras |

## Config
| Fichier | Rôle |
|---------|------|
| `Cargo.toml` | Dépendances : clap 4, serde 1, serde_json 1, dirs 6 |
| `~/.config/launch-claude-code/settings.json` | Profils utilisateur (externe au repo) |
