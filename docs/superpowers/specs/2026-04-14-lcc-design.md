# lcc — Launch Claude Code CLI

Wrapper CLI Rust pour lancer `claude` avec des profils de modèles opensource (local + cloud).

## Commandes

### `lcc start --profil <name> [-- <claude args...>]`

1. Charge `~/.config/launch-claude-code/settings.json`
2. Résout le profil par nom
3. Résout le binaire `claude` : PATH d'abord, puis `~/.claude/local/claude`
4. Injecte les variables d'environnement :
   - `ANTHROPIC_BASE_URL` = profil.base_url
   - `ANTHROPIC_API_KEY` = profil.api_key
   - `ANTHROPIC_AUTH_TOKEN` = profil.auth_token
   - `ANTHROPIC_DEFAULT_OPUS_MODEL` = profil.model
   - `ANTHROPIC_DEFAULT_SONNET_MODEL` = profil.model
   - `ANTHROPIC_DEFAULT_HAIKU_MODEL` = profil.model
   - `CLAUDE_CODE_SUBAGENT_MODEL` = profil.model
   - `CLAUDE_CODE_ATTRIBUTION_HEADER` = `0` (hardcodé)
   - + chaque entrée de profil.env (optionnel)
5. Exec `claude --model <profil.model> <claude args...>`

### `lcc list`

Affiche un tableau ASCII des profils disponibles : nom, modèle, base_url.

### `lcc settings`

- Sans flag : ouvre le fichier settings.json avec `open` (macOS)
- `--validate` : parse le JSON, vérifie la structure, affiche OK ou liste des erreurs

## Format settings.json

Chemin : `~/.config/launch-claude-code/settings.json`

```json
{
  "profiles": {
    "gemma4": {
      "model": "gemma4",
      "base_url": "http://localhost:11434/v1",
      "api_key": "",
      "auth_token": "ollama"
    },
    "openrouter-llama4": {
      "model": "meta-llama/llama-4-maverick",
      "base_url": "https://openrouter.ai/api/v1",
      "api_key": "sk-or-xxx",
      "auth_token": "openrouter",
      "env": {
        "CLAUDE_CODE_AUTO_COMPACT_WINDOW": "50000"
      }
    }
  }
}
```

Champs requis par profil : `model`, `base_url`, `api_key`, `auth_token`.
Champ optionnel : `env` (map clé/valeur de variables d'environnement additionnelles).

## Structure du projet

```
src/
  main.rs       — CLI clap (derive), dispatch des commandes
  config.rs     — types serde, chargement et validation du settings.json
  runner.rs     — résolution du binaire claude, construction env, exec
Cargo.toml
```

## Dépendances

- `clap` (derive) — parsing CLI
- `serde` + `serde_json` — (dé)sérialisation config
- `dirs` — résolution du home directory

## Gestion d'erreurs

- Profil introuvable : message clair + liste des profils disponibles
- settings.json absent : message avec le chemin attendu
- `claude` introuvable : message avec les chemins tentés
- JSON invalide (validate) : affiche les erreurs de parsing
