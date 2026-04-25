# ARCHITECTURE

## Core shape
- local-first repo
- Rust workspace under `rust/`
- single `cowork-cli` binary
- memory files under `cowork/`

## Commands
- `cowork chat`
- `cowork task "<goal>"`
- `cowork tasks`
- `cowork doctor`

## Obsidian sync
- task append auto-runs `scripts/sync_obsidian.py` when present
- generated files: `Dashboard.md`, `Tasks.md`, `Context.md`
