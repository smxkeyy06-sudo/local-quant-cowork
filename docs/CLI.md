# CLI Contract

## `cowork chat`
- Prints repository location and active prompt/role profile.
- Intentionally non-interactive in this scaffold pass.

## `cowork context "<note>"`
- Requires a non-empty `<note>` argument.
- Appends a timestamped markdown note to `cowork/context.md`.
- Prints a JSON success result with `context_file`, `timestamp`, and `note_length`.
- If `scripts/sync_obsidian.py` exists, attempts to run it after append; sync failures are warnings and do not fail context append.

## `cowork task "<goal>"`
- Requires a non-empty `<goal>` argument (returns non-zero on missing goal).
- Loads and validates `cowork/tasks.json`.
- Appends a new queued task with `created_at` and `updated_at`, then writes pretty JSON back to `cowork/tasks.json`.
- Prints a JSON success result with `id`, `goal`, `status`, and `total_task_count`.
- If `scripts/sync_obsidian.py` exists, attempts to run it after append; sync failures are warnings and do not fail task append.
- Obsidian sync writes `Dashboard.md`, `Tasks.md`, and `Context.md`.

## `cowork task-note <task-id> "<note>"`
- Requires an existing `<task-id>` and a non-empty `<note>`.
- Appends a timestamped note to the matching task.
- Preserves existing notes and `created_at`, updates `updated_at`, and writes pretty JSON back to `cowork/tasks.json`.
- Prints a JSON success result with `id`, `note_timestamp`, `note_length`, and `total_notes`.
- If `scripts/sync_obsidian.py` exists, attempts to run it after append; sync failures are warnings and do not fail note append.

## `cowork task-status <task-id> <status>`
- Requires an existing `<task-id>` and a known `<status>`.
- Allowed statuses: `queued`, `active`, `done`, `blocked`.
- Updates the matching task status, preserves `created_at`, updates `updated_at`, and writes pretty JSON back to `cowork/tasks.json`.
- Prints a JSON success result with `id`, `old_status`, `new_status`, and `goal`.
- If `scripts/sync_obsidian.py` exists, attempts to run it after update; sync failures are warnings and do not fail status update.

## `cowork tasks`
- Loads and validates `cowork/tasks.json`.
- Prints total task count, counts by status, and each task. Shows `updated_at` and note count when present.

## `cowork doctor`
- Reports repo root and memory directory.
- Checks required top-level repo files and required memory files.
- Parses `cowork/tasks.json` and reports version, total count, and counts by status.
- Reports duplicate task IDs, invalid task statuses, and empty task goals.
- Reports `COWORK_MODEL` / `COWORK_DATA_DIR` env status.
- Prints visible tool names from the registry.
- Returns non-zero if required files are missing or task records are invalid.
