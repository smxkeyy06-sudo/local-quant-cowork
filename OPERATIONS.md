# OPERATIONS

## Workflow
1. run `cowork doctor`
2. append context notes with `cowork context "<note>"`
3. append tasks with `cowork task "<goal>"`
4. update status with `cowork task-status <task-id> <status>`
5. list tasks with `cowork tasks`
6. commit working changes

Context append, task append, and status updates auto-run `scripts/sync_obsidian.py`
when present. Obsidian sync writes `Dashboard.md`, `Tasks.md`, and `Context.md`.
