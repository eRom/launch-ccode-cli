# Architecture — lcc (Launch Claude Code)
> Dernière mise à jour : 2026-04-14

## Vue d'ensemble
CLI wrapper Rust pour lancer `claude` avec des profils de modèles configurables (local Ollama, cloud OpenRouter, etc.).

## Stack
- **Langage** : Rust (edition 2021)
- **CLI** : clap 4 (derive)
- **Sérialisation** : serde + serde_json
- **Système** : dirs (résolution home dir)

## Arborescence
```
src/
  main.rs       — point d'entrée CLI, dispatch 3 sous-commandes
  lib.rs        — exposition modules publics (pour tests)
  config.rs     — types Settings/Profile, chargement/validation JSON
  runner.rs     — résolution binaire claude, construction env vars, exec
tests/
  config_test.rs  — 4 tests parsing config
  runner_test.rs  — 4 tests env vars + args
docs/superpowers/
  specs/          — design spec
  plans/          — plan d'implémentation
```

## Flux principal
1. `lcc start --profil <name>` parse les args (clap)
2. `config::load_settings()` charge `~/.config/launch-claude-code/settings.json`
3. `runner::run()` résout le profil, trouve le binaire `claude`, injecte les env vars, exec

## Config externe
- `~/.config/launch-claude-code/settings.json` — profils utilisateur
