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

Run `scripts/generate_daily.py` to create or refresh `Journal/YYYY-MM-DD.md`
in the configured Obsidian vault.

## Obsidian plugins

Obsidian remains a generated, readable view; this repo and
`docs/CURRENT_STATE.md` remain the source of truth. For low-risk use, start
with Calendar to navigate `Journal/YYYY-MM-DD.md`. Dataview may be useful
later for read-only dashboards once generated notes have stable
metadata/frontmatter.

Use Tasks with caution because `cowork/tasks.json` is the task source of truth.
Avoid Obsidian Git for now so commits, checkpoints, and pushes stay in the WSL
repo workflow. Plugins should not mutate generated project state or bypass
`checkpoint.py`.
