# ROADMAP

## Current phase
- local cowork AI foundation complete
- core state lives in `cowork/tasks.json` and `cowork/context.md`
- Obsidian output is generated from local repo state

## Completed
- `cowork context "<note>"` appends durable context notes
- tasks have `created_at`, `updated_at`, and optional notes
- `cowork task "<goal>"` appends queued tasks
- `cowork task-note <task-id> "<note>"` appends task notes
- `cowork task-status <task-id> <status>` updates task status
- `cowork tasks` lists all tasks
- `cowork tasks --status <status>` filters by status
- `cowork doctor` audits repo and task health
- `scripts/sync_obsidian.py` writes `Dashboard.md`, `Tasks.md`, and `Context.md`
- `scripts/generate_daily.py` writes `Journal/YYYY-MM-DD.md`

## Next phase
Stabilize the local operating loop before adding larger AI behavior.

Start with:
- confirm the Obsidian vault path and write permissions are reliable
- add lightweight script smoke checks where useful
- keep command output and docs aligned with implemented behavior
- avoid new task schema changes until the current workflow has been used in practice
