# DECISIONS

## D-0001
Use a local-first Rust CLI foundation.

## D-0002
Use single-agent flow first.

## D-0003
Persist working state in `cowork/tasks.json`, `cowork/mission.md`, and `cowork/context.md`.

## D-0004
Keep task state as pretty JSON with backward-compatible optional fields.

## D-0005
Use warning-only Obsidian sync from mutating commands so task/context changes are not blocked by vault availability.

## D-0006
Generate Obsidian files from repo state: `Dashboard.md`, `Tasks.md`, `Context.md`, and daily notes under `Journal/`.

## D-0007
Keep daily log generation as a script for now, not a Rust CLI command.

## D-0008
Next phase is local workflow stabilization before larger AI/provider integration.
