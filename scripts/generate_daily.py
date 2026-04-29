#!/usr/bin/env python3
import os
import sys
from datetime import date
from pathlib import Path

from sync_obsidian import (
    KNOWN_STATUSES,
    REPO_ROOT,
    VAULT_DIR as DEFAULT_VAULT_DIR,
    append_task_section,
    latest_context_snippet,
    load_env_file,
    load_tasks,
    tasks_by_status,
)


def resolve_vault_dir():
    load_env_file(REPO_ROOT / ".env")
    return Path(os.environ.get("OBSIDIAN_VAULT_DIR", str(DEFAULT_VAULT_DIR)))


def build_daily_note(tasks_data, today):
    tasks = tasks_data.get("tasks", [])
    grouped = tasks_by_status(tasks)
    counts = {status: len(grouped[status]) for status in KNOWN_STATUSES}
    recently_done = [
        task
        for task in grouped["done"]
        if task.get("updated_at") or task.get("created_at")
    ]
    context_snippet = latest_context_snippet()

    lines = []
    lines.append(f"# Daily Log - {today}")
    lines.append("")
    lines.append("## Links")
    lines.append("- [[Dashboard]]")
    lines.append("- [[Tasks]]")
    lines.append("- [[Context]]")
    lines.append("")
    lines.append("## Task Counts")
    lines.append(f"- total: {len(tasks)}")
    lines.append(f"- queued: {counts['queued']}")
    lines.append(f"- active: {counts['active']}")
    lines.append(f"- blocked: {counts['blocked']}")
    lines.append(f"- done: {counts['done']}")
    lines.append("")

    append_task_section(lines, "Active Tasks", grouped["active"], "No active tasks.")
    append_task_section(lines, "Blocked Tasks", grouped["blocked"], "No blocked tasks.")
    append_task_section(lines, "Recently Done Tasks", recently_done, "No timestamped done tasks.", limit=5)

    lines.append("## Latest Context")
    if context_snippet:
        lines.append(context_snippet)
    else:
        lines.append("No context captured yet.")
    lines.append("")

    return "\n".join(lines)


def write_daily_note(vault_dir, today, content):
    journal_dir = vault_dir / "Journal"
    daily_path = journal_dir / f"{today}.md"

    try:
        journal_dir.mkdir(parents=True, exist_ok=True)
        daily_path.write_text(content, encoding="utf-8")
    except OSError as exc:
        print(f"daily_log: error writing {daily_path}: {exc}", file=sys.stderr)
        return 1

    print(f"Generated daily note: {daily_path}")
    return 0


def main():
    today = date.today().isoformat()
    vault_dir = resolve_vault_dir()
    tasks_data = load_tasks()
    content = build_daily_note(tasks_data, today)
    return write_daily_note(vault_dir, today, content)


if __name__ == "__main__":
    raise SystemExit(main())
