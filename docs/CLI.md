# CLI Contract

## `cowork chat`
- Prints repository location, active prompt/role profile, and visible tool names.
- Intentionally non-interactive in this scaffold pass.

## `cowork task "<goal>"`
- Requires a non-empty `<goal>` argument (returns non-zero on missing goal).
- Loads and validates `cowork/tasks.json`.
- Appends a new queued task and writes pretty JSON back to `cowork/tasks.json`.
- Prints a JSON success result with `id`, `goal`, `status`, and `total_task_count`.

## `cowork doctor`
- Reports repo root and memory directory.
- Checks required top-level repo files and required memory files.
- Parses `cowork/tasks.json` and reports version, total count, and counts by status.
- Reports duplicate task IDs, invalid task statuses, and empty task goals.
- Reports `COWORK_MODEL` / `COWORK_DATA_DIR` env status.
- Prints visible tool names from the registry.
- Returns non-zero if required files are missing or task records are invalid.
