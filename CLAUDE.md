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
