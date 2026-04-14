# Gotchas — lcc
> Dernière mise à jour : 2026-04-14

## dirs::config_dir() sur macOS
- `dirs::config_dir()` retourne `~/Library/Application Support/` sur macOS
- Le projet utilise `dirs::home_dir().join(".config/")` à la place pour respecter la convention XDG
- Corrigé dans le commit `c85fca1`

## Résolution du binaire claude
- `find_claude()` utilise `which` via `Command::new("which")` — fonctionne sur macOS/Linux
- Pas de support Windows (`which` n'existe pas) — acceptable pour le moment
