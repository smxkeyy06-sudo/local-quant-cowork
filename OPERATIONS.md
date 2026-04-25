# OPERATIONS

## Workflow
1. run `cowork doctor`
2. append tasks with `cowork task "<goal>"`
3. update status with `cowork task-status <task-id> <status>`
4. list tasks with `cowork tasks`
5. commit working changes

Task append and status updates auto-run `scripts/sync_obsidian.py` when
present. Obsidian sync writes `Dashboard.md`, `Tasks.md`, and `Context.md`.
