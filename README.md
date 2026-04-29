# local-quant-cowork

Focused local-first cowork AI scaffold for a Rust-based quant coordination system.

## Scope (v0)
- Single-agent command line workflow.
- Narrow tool surface and explicit memory files.
- No giant framework and no multi-agent swarm.
- Documentation-first repository layout.

## Commands
```bash
cowork chat
cowork context "<note>"
cowork task "<goal>"
cowork task-note <task-id> "<note>"
cowork task-status <task-id> <status>
cowork tasks
cowork tasks --status queued
cowork doctor
```

Context append, task append, task notes, and status updates auto-run
`scripts/sync_obsidian.py` when present. Obsidian sync writes `Dashboard.md`,
`Tasks.md`, and `Context.md`; the dashboard summarizes task status, focus
queues, recent completions, and latest context.
