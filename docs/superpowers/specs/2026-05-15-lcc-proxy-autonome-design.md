# lcc — Proxy LiteLLM autonome (V1)

**Date** : 2026-05-15
**Statut** : design validé, plan d'implémentation à venir
**Auteur** : Romain Ecarnot (avec Claude Opus 4.7)

---

## 1. Contexte & problème

`lcc` permet de lancer Claude Code (`claude`) avec des profils pointant vers des APIs Anthropic-compatibles (OpenRouter, DeepSeek, Groq, etc.). Plusieurs modèles « récalcitrants » (Qwen3.6-plus, DeepSeek, Gemma) échouent silencieusement : le terminal revient sans output, sans erreur.

**Cause racine identifiée** (debug du 2026-05-15) : Claude Code valide une signature cryptographique sur les content blocks de type `thinking`. Les modèles reasoning hostés via OpenRouter renvoient bien des blocs `thinking` au format Anthropic, mais sans la signature qu'Anthropic seul peut produire. Claude Code drop la réponse en silence → terminal vide.

**Vérifié par curl** : `POST https://openrouter.ai/api/v1/messages` avec un model Qwen renvoie un JSON Anthropic-shape valide contenant `{"content":[{"type":"thinking","thinking":"..."}]}` sans champ `signature`.

## 2. Objectifs

**V1 — ce design** :
- Tout appel `lcc start --profil X` passe par un proxy local **transparent** et **autonome**.
- Le proxy strip les `thinking` blocks problématiques avant que la réponse atteigne `claude`.
- Aucune action manuelle par l'utilisateur après le `lcc proxy install` initial.
- Le proxy survit aux reboots (daemon launchd).
- macOS-only en V1.

**Non-objectifs V1** (cf §10) :
- Multi-OS, hot-reload, TLS, UI/TUI, métriques exposées, load balancing, proxy Rust maison.

## 3. Principe directeur — séparation `claude` vs `lcc`

| Outil | Cible | Auth | Path réseau |
|---|---|---|---|
| `claude ...` | API Anthropic native (abo Max) | clé Anthropic ou OAuth Max | direct → `api.anthropic.com` |
| `lcc start --profil X ...` | N'importe quel provider via profil | dépend du profil | **toujours** via `localhost:4000` (LiteLLM daemon) |

**Conséquences** :
- Si LiteLLM daemon est down → `claude` natif marche, `lcc` échoue net avec message clair.
- Pas de logique conditionnelle « ce profil est-il proxy ou direct » dans lcc — on traduit toujours le profil en route LiteLLM.
- Le binaire `claude` lancé par `lcc` croit parler à Anthropic ; en vrai il parle à LiteLLM, qui parle au vrai provider.

## 4. Architecture & data flow

```
USER
  │ $ lcc start --profil qwen -p "raconte une blague"
  ▼
lcc (Rust)
  │ 1. Charge ~/.config/launch-claude-code/settings.json
  │ 2. Résout profil "qwen"
  │ 3. Health check GET http://localhost:4000/health/liveness (timeout 1s)
  │    └─ down → exit 1 + message "lance: lcc proxy doctor"
  │ 4. Override env vars :
  │      ANTHROPIC_BASE_URL  = http://localhost:4000
  │      ANTHROPIC_AUTH_TOKEN = <master_key keychain>
  │      ANTHROPIC_MODEL     = qwen   (model_name LiteLLM, pas le slug provider)
  │ 5. exec claude avec ces env + args utilisateur
  ▼
claude (binaire CC)
  │ POST http://localhost:4000/v1/messages
  │ {model: "qwen", messages: [...], max_tokens: ...}
  ▼
LiteLLM daemon (Python, géré par launchd)
  │ - Lit ~/.config/launch-claude-code/litellm.yaml
  │ - Route "qwen" → openrouter/qwen/qwen3.6-plus
  │ - Auth Bearer $OPENROUTER_API_KEY (env du daemon, depuis .zshrc à l'install)
  │ - Translation Anthropic ↔ OpenAI
  │ - Strip thinking blocks (callback custom lcc_strip_thinking.py)
  ▼
Provider externe (OpenRouter, DeepSeek, ...)
```

### 4.1 Composants & fichiers

| Composant | Lieu | Lifecycle |
|---|---|---|
| `lcc` binary | `~/.cargo/bin/lcc` (ou `/usr/local/bin/lcc`) | invoqué à la demande |
| `settings.json` (profils) | `~/.config/launch-claude-code/settings.json` | édité par l'utilisateur |
| `litellm.yaml` (généré) | `~/.config/launch-claude-code/litellm.yaml` | régénéré par `lcc proxy reload` |
| LiteLLM venv | `~/.local/share/lcc/litellm-venv/` (dir uv tool) | installé par `lcc proxy install` |
| `lcc_strip_thinking.py` | versionné dans `assets/`, copié dans `<venv>/lib/python*/site-packages/` (sys.path du daemon) | déployé par `proxy install` |
| `lcc-litellm-launcher.sh` | `~/.local/share/lcc/lcc-litellm-launcher.sh` | wrapper généré par `proxy install`, lance LiteLLM avec env Keychain |
| Plist launchd | `~/Library/LaunchAgents/com.lcc.litellm.plist` (pointe vers le wrapper) | créé par `proxy install` |
| Master key (auth proxy) | Keychain macOS, item `lcc.litellm.master_key` | auto-généré au premier install |
| Logs LiteLLM | `~/Library/Logs/lcc/litellm.{out,err}.log` | rotation par launchd |
| `proxy.toml` (port, etc.) | `~/.config/launch-claude-code/proxy.toml` | édité via `lcc proxy set-port` |

### 4.2 Points de design clés

- **Master key obligatoire** : LiteLLM exige un `master_key`. Auto-généré au premier `proxy install` (32 bytes random hex), stocké dans le Keychain macOS via `security add-generic-password -s lcc.litellm.master_key -a $USER`. Deux usages :
  - **Côté daemon** (au boot) : le LaunchAgent lance un wrapper `~/.local/share/lcc/lcc-litellm-launcher.sh` (généré par `proxy install`) qui exporte `LCC_MASTER_KEY=$(security find-generic-password -s lcc.litellm.master_key -w)` puis `exec litellm --config <yaml> --port <port>`. LaunchAgent tourne dans la session user → Keychain accessible.
  - **Côté lcc** (à chaque `start`) : lit la même master_key via `security find-generic-password -s lcc.litellm.master_key -w` et l'injecte dans `ANTHROPIC_AUTH_TOKEN` pour que `claude` puisse s'auth contre le proxy.
  - Empêche un autre process local d'utiliser le proxy à l'insu du user.
- **Port 4000 par défaut**, configurable via `lcc proxy set-port <n>`. Stocké dans `proxy.toml`. Changement → régénère plist + restart daemon.
- **`model_name` court comme alias** : dans le yaml, le `model_name` LiteLLM = le nom du profil (`qwen`, `deepseek`). C'est ce qu'on passe à `claude` via `ANTHROPIC_MODEL`. Le slug réel du provider (`openrouter/qwen/qwen3.6-plus`) reste interne au yaml.

## 5. Subcommands `lcc proxy *`

Sous-ensemble parallèle à `start`/`list`/`settings`.

```
lcc proxy install      # one-shot setup : check uv → uv tool install litellm[proxy]
                       #                 → génère keychain master_key
                       #                 → génère litellm.yaml depuis settings.json
                       #                 → écrit le plist launchd
                       #                 → launchctl load
                       #                 → attend health-check OK
                       #                 → "✓ proxy ready on :4000"

lcc proxy uninstall    # launchctl unload + supprime plist + uv tool uninstall litellm
                       #   (laisse yaml et master_key par défaut, --purge pour tout effacer)

lcc proxy status       # daemon up/down, PID, port, uptime, modèles routés,
                       #   dernière erreur dans logs si down

lcc proxy restart      # launchctl kickstart -k
lcc proxy stop         # launchctl unload (sans supprimer le plist)
lcc proxy start        # launchctl load
lcc proxy reload       # régénère yaml depuis settings.json + restart daemon
lcc proxy logs [-f]    # tail ~/Library/Logs/lcc/litellm.err.log (et .out si --all)
lcc proxy doctor       # diagnostic complet : uv ? venv ? daemon ? port ? yaml ?
                       #   master_key ? curl /health/liveness ? → check-list verte/rouge
lcc proxy set-port <n> # change port, regen plist, restart
```

### 5.1 UX

- **`install` est idempotent** : si déjà installé, prompt `reinstall`/`reload`/`abort`.
- **`reload` auto-suggéré** : après modification de `settings.json` via `lcc settings`, propose « profils modifiés, recharger le proxy maintenant ? (y/N) ». Évite l'oubli.
- **`doctor` est le point d'entrée de tout message d'erreur** : « Le proxy ne répond pas. Lance `lcc proxy doctor` pour diagnostiquer. »
- **Pas de `lcc proxy edit-yaml`** : single source of truth = `settings.json`. Modifier le yaml à la main est non supporté.

## 6. Mapping profil → `litellm.yaml`

### 6.1 Exemple complet

**Profil source (`settings.json`) — format inchangé** :

```json
{
  "profiles": {
    "qwen": {
      "model": "qwen/qwen3.6-plus",
      "base_url": "https://openrouter.ai/api/v1",
      "auth_token": "${OPENROUTER_API_KEY}"
    },
    "deepseek": {
      "model": "deepseek-chat",
      "base_url": "https://api.deepseek.com/v1",
      "auth_token": "${DEEPSEEK_API_KEY}"
    }
  }
}
```

**`litellm.yaml` généré** :

```yaml
model_list:
  - model_name: qwen
    litellm_params:
      model: openrouter/qwen/qwen3.6-plus
      api_key: os.environ/OPENROUTER_API_KEY
      drop_params: true
  - model_name: deepseek
    litellm_params:
      model: deepseek/deepseek-chat
      api_key: os.environ/DEEPSEEK_API_KEY
      drop_params: true

litellm_settings:
  drop_params: true
  set_verbose: false
  callbacks: ["lcc_strip_thinking"]

general_settings:
  master_key: os.environ/LCC_MASTER_KEY
  database_url: null
  store_model_in_db: false
```

### 6.2 Algo de génération (dans `lcc proxy reload`)

1. Charger `settings.json` (réutilise `config.rs`).
2. Pour chaque profil `Single` → 1 entrée `model_list`.
3. Pour chaque profil `Multi` → N entrées (une par modèle déclaré dans `models`).
4. Détecter le préfixe LiteLLM à coller au model id (`openrouter/`, `deepseek/`, `groq/`, ...) à partir du `base_url`, via une **table de mapping codée en dur** dans Rust :

   ```rust
   const PROVIDER_MAP: &[(&str, &str)] = &[
       ("openrouter.ai",      "openrouter"),
       ("api.deepseek.com",   "deepseek"),
       ("api.groq.com",       "groq"),
       ("api.together.xyz",   "together_ai"),
       ("api.mistral.ai",     "mistral"),
       // extensible, pull request bienvenue
   ];
   ```

5. Sérialiser en YAML (crate `serde_yaml`), écrire dans `litellm.yaml` (mode 600).
6. `launchctl kickstart -k gui/$(id -u)/com.lcc.litellm` pour reload le daemon (cible LaunchAgent user, pas système).

### 6.3 Strip des `thinking` blocks

Petit fichier Python `assets/lcc_strip_thinking.py` (~30 lignes), versionné dans le repo, copié dans le venv au moment du `proxy install`.

```python
from litellm.integrations.custom_logger import CustomLogger

class StripThinkingCallback(CustomLogger):
    async def async_post_call_success_hook(self, data, user_api_key_dict, response):
        # Filtre les content blocks de type "thinking" qui n'ont pas de signature
        # → claude ne plantera plus en silence sur les modèles reasoning
        if hasattr(response, 'content') and isinstance(response.content, list):
            response.content = [
                b for b in response.content
                if not (isinstance(b, dict) and b.get('type') == 'thinking' and 'signature' not in b)
            ]
        return response

lcc_strip_thinking = StripThinkingCallback()
```

Référencé dans `litellm.yaml` via `callbacks: ["lcc_strip_thinking"]`.

## 7. Health check & error handling

### 7.1 Flow `lcc start`

```
1. Health check : GET http://localhost:4000/health/liveness (timeout 1s)
   ├─ 200 OK → continue
   └─ fail → exit 1 + message :
        "✗ Le proxy LiteLLM ne répond pas sur :4000.
         → Lance: lcc proxy doctor
         → Ou:   lcc proxy start"

2. Lire master_key depuis Keychain (échec → suggère lcc proxy install)

3. Spawn claude avec env overridé.

4. Si claude exit avec code != 0 ET log proxy contient une erreur récente :
   suggère "lcc proxy logs" en bas du message d'erreur.
```

**Pas de retry, pas de spawn-on-failure.** Le user a explicitement choisi le modèle daemon → si daemon down c'est un état système, pas une glitch transitoire. Message clair > magie.

### 7.2 `lcc proxy doctor`

Check-list séquentielle avec marker visuel par étape :

```
Diagnostic du proxy LiteLLM
─────────────────────────────
[✓] uv installé                  (v0.4.18)
[✓] LiteLLM venv présent          (~/.local/share/lcc/litellm-venv)
[✓] LiteLLM version               (v1.50.2)
[✓] Plist launchd présent         (com.lcc.litellm)
[✓] Daemon en cours d'exécution   (PID 4321, uptime 2h13m)
[✗] Health check HTTP             (timeout sur :4000)
    → Le daemon tourne mais n'écoute pas. Vérifie ~/Library/Logs/lcc/litellm.err.log
    → Suggéré : lcc proxy restart
[ ] Master key dans Keychain      (skipped: étape précédente échouée)
[ ] Yaml valide                   (skipped)
[ ] Modèles routés                (skipped)

✗ Diagnostic : 1 erreur, 2 étapes skippées
```

## 8. Sécurité

| Secret | Stockage | Accès |
|---|---|---|
| `LCC_MASTER_KEY` | Keychain macOS (`security add-generic-password -s lcc.litellm.master_key -a $USER`) | (a) lu par lcc à chaque `start` pour `ANTHROPIC_AUTH_TOKEN` ; (b) lu par le wrapper `lcc-litellm-launcher.sh` au boot du daemon pour exporter `LCC_MASTER_KEY` (cf §4.2) |
| Clés providers (OpenRouter, DeepSeek, ...) | **shell env (`.zshrc`)** | passées au plist `EnvironmentVariables` au moment de `proxy install` (snapshot) |
| `litellm.yaml` | mode 600, `~/.config/launch-claude-code/` | lecture daemon uniquement |

**Règles dures** :
- Aucun secret en clair dans `litellm.yaml` ni dans le plist en repo.
- Toujours `os.environ/...` côté yaml.
- Toujours `${VAR}` côté `settings.json` (déjà supporté par `config.rs`).
- Si une clé provider change dans `.zshrc`, l'utilisateur relance `lcc proxy reload` (qui re-snapshot l'env vers le plist).

**V2 envisagée** : migrer toutes les clés providers vers Keychain également (plus propre, mais plus de friction au bootstrap).

## 9. Tests

| Niveau | Quoi | Comment |
|---|---|---|
| Unit (Rust) | Génération yaml depuis `settings.json` | snapshot tests : profil → yaml attendu byte-à-byte |
| Unit (Rust) | Génération plist | snapshot tests |
| Unit (Rust) | Health check parser | mock HTTP server local (`mockito` ou similaire) |
| Unit (Rust) | Détection préfixe provider depuis base_url | table de cas |
| Unit (Python) | Strip thinking callback | pytest avec fixtures de réponses Qwen typiques (avec et sans signature) |
| Integration (Rust + vrai LiteLLM) | `lcc proxy install/start/status/stop/uninstall` E2E | scriptable en CI macOS, sinon manuel |
| Smoke | `lcc start --profil qwen -p "hi"` retourne du texte non-vide | manuel, documenté dans README |

**Pas de tests sur le proxy LiteLLM lui-même** — c'est de l'upstream, on leur fait confiance.

## 10. Non-objectifs explicites V1

Pour cadrer le scope :

- ❌ Multi-OS (Linux/Windows) — macOS only en V1.
- ❌ Hot-reload du yaml sans `proxy reload` (LiteLLM ne le supporte pas proprement).
- ❌ TLS sur le proxy (localhost only, suffisant).
- ❌ UI / TUI pour la config (`settings.json` reste éditeur de texte).
- ❌ Métriques Prometheus exposées dans `lcc proxy status` (LiteLLM les expose en interne, V2).
- ❌ Pool de profils actifs / load balancing (1 profil par invocation).
- ❌ Migration vers proxy Rust maison (V2/V3, design séparé).
- ❌ Auto-update de LiteLLM (à la charge du user via `uv tool upgrade litellm`).
- ❌ Support Linux/systemd (un éventuel V1.1).

## 11. Migration depuis l'état actuel

Pour Romain, après l'implémentation :

```bash
brew install uv                       # si pas déjà
git pull && cargo install --path .    # nouveau lcc
lcc proxy install                     # one-shot, ~30s
lcc proxy doctor                      # vérif tout vert
lcc start --profil qwen -p "hello"    # marche enfin
```

Profils existants dans `settings.json` continuent de marcher tels quels — ils sont juste interprétés différemment (mappés en routes LiteLLM au lieu d'être passés direct à claude).

## 12. Risques & questions ouvertes

| Risque | Mitigation |
|---|---|
| LiteLLM upstream change l'API du callback `CustomLogger` | pin de version dans `lcc proxy install` (`litellm==X.Y.Z`), update manuel via `uv tool upgrade` |
| Format des `thinking` blocks varie selon provider | callback testé sur Qwen ; étendre les fixtures au fur et à mesure |
| Conflit de port 4000 (autre service local) | `lcc proxy set-port` + détection au `install` (`lsof -i :4000`) |
| Master key perdue (Keychain effacé) | `lcc proxy install --reset-key` pour régénérer |
| Provider non dans `PROVIDER_MAP` | fallback : utiliser le `base_url` brut, prefix `openai/` (LiteLLM accepte tout endpoint OpenAI-compat) ; logger un warning |

## 13. Roadmap

- **V1** (ce design) : LiteLLM Python via launchd, macOS, OpenRouter/DeepSeek/Groq/Together/Mistral.
- **V1.1** : support Linux (systemd user units), Keychain → Secret Service.
- **V2** : exposition de métriques basiques dans `lcc proxy status`, hot-reload partiel.
- **V3** : éventuel proxy Rust maison si justifié par profil de perf / besoin d'indépendance Python.
