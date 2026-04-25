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
cowork task "<goal>"
cowork task-status <task-id> <status>
cowork tasks
cowork doctor
```

Task append and status updates auto-run `scripts/sync_obsidian.py` when
present. Obsidian sync writes `Dashboard.md`, `Tasks.md`, and `Context.md`.
