# OPERATIONS

## Workflow
1. run `cowork doctor`
2. append context notes with `cowork context "<note>"`
3. append tasks with `cowork task "<goal>"`
4. append task notes with `cowork task-note <task-id> "<note>"`
5. update status with `cowork task-status <task-id> <status>`
6. list tasks with `cowork tasks` or `cowork tasks --status queued`
7. commit working changes

Context append, task append, task notes, and status updates auto-run
`scripts/sync_obsidian.py` when present. Obsidian sync writes `Dashboard.md`,
`Tasks.md`, and `Context.md`; the dashboard summarizes status counts and the
current task queues.
