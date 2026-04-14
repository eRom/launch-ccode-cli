## Contexte projet (.memory)

Le dossier `.memory/` contient la cartographie persistante du projet :
- `architecture.md` — vue d'ensemble, stack, flux de données
- `key-files.md` — fichiers critiques et leur rôle
- `patterns.md` — conventions et patterns récurrents
- `gotchas.md` — pièges, bugs résolus, workarounds

**Ne lis PAS ces fichiers au démarrage.** Lis-les à la demande, uniquement quand la question de l'utilisateur touche au domaine concerné (ex: question archi → `architecture.md`, bug étrange → `gotchas.md`). Pour une question triviale ou sans rapport avec le projet lui-même, ne les lis pas du tout.

## Gerber

Ce projet est indexé dans **gerber** sous le slug `launch-ccode-cli`.
Slug cross-projet : `erom` (design system, conventions, preferences personnelles). Pour les sujets design/UI, conventions, stack : chercher aussi dans `erom`.

Entites :
- **Notes** (atoms + documents) — mémoire de connaissance, recherche sémantique/fulltext
- **Tasks** — tâches projet avec kanban 7 colonnes (inbox → brainstorming → specification → plan → implementation → test → done)
- **Issues** — problèmes/bugs avec kanban 4 colonnes (inbox → in_progress → in_review → closed)
- **Messages** — bus inter-sessions (context + reminder)

Skills disponibles :
- `/gerber-recall` — recherche contextuelle dans la mémoire cross-projets
- `/gerber-capture` — capture rapide d'un atome de connaissance
- `/gerber-archive` — extraction et archivage fin de session
- `/gerber-review` — maintenance hebdomadaire (notes, tasks, issues)
- `/gerber-import` — migration one-shot depuis .memory/
- `/gerber-inbox` — consulter les messages inter-sessions
- `/gerber-send` — envoyer un message inter-session
- `/gerber-task` — gestion des tâches projet (kanban)
- `/gerber-issue` — gestion des issues projet
