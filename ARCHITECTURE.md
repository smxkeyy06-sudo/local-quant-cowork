# ARCHITECTURE

## Core shape
- local-first repo
- Rust workspace under `rust/`
- single `cowork-cli` binary
- memory files under `cowork/`

## Commands
- `cowork chat`
- `cowork context "<note>"`
- `cowork task "<goal>"`
- `cowork task-note <task-id> "<note>"`
- `cowork task-status <task-id> <status>`
- `cowork tasks`
- `cowork doctor`

## Obsidian sync
- context append, task append, task notes, and status updates auto-run `scripts/sync_obsidian.py` when present
- generated files: `Dashboard.md`, `Tasks.md`, `Context.md`
- `Dashboard.md` summarizes status counts, active/blocked tasks, recent queue movement, and latest context
