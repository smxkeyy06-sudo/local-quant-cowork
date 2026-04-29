#!/usr/bin/env python3
import json
import os
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
COWORK_DIR = REPO_ROOT / "cowork"

VAULT_DIR = Path(
    os.environ.get(
        "OBSIDIAN_VAULT_DIR",
        "/mnt/c/Users/Alejandro/Documents/Obsidian/LocalQuantCoworkVault",
    )
)

TASKS_JSON = COWORK_DIR / "tasks.json"
CONTEXT_MD = COWORK_DIR / "context.md"
KNOWN_STATUSES = ("queued", "active", "done", "blocked")


def load_env_file(path: Path) -> None:
    if not path.exists():
        return

    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        key = key.strip()
        value = value.strip()
        if key and key not in os.environ:
            os.environ[key] = value


def load_tasks():
    with TASKS_JSON.open("r", encoding="utf-8") as f:
        return json.load(f)


def task_timestamp(task):
    return task.get("updated_at") or task.get("created_at") or ""


def timestamp_sort_key(task):
    timestamp = task_timestamp(task)
    if timestamp.startswith("unix:"):
        raw_value = timestamp.split(":", 1)[1]
        if raw_value.isdigit():
            return int(raw_value)
    return 0


def task_summary(task):
    updated = task.get("updated_at")
    timestamp = f" | updated: `{updated}`" if updated else ""
    notes = task.get("notes") or []
    note_count = f" | notes: {len(notes)}" if notes else ""
    return f"`{task.get('id','')}` | **{task.get('status','unknown')}** | {task.get('goal','')}{timestamp}{note_count}"


def tasks_by_status(tasks):
    grouped = {status: [] for status in KNOWN_STATUSES}
    for task in tasks:
        status = task.get("status", "unknown")
        if status in grouped:
            grouped[status].append(task)
    return grouped


def append_task_section(lines, title, tasks, empty_message, limit=None):
    selected = sorted(tasks, key=timestamp_sort_key, reverse=True)
    if limit is not None:
        selected = selected[:limit]

    lines.append(f"## {title}")
    if selected:
        for task in selected:
            lines.append(f"- {task_summary(task)}")
    else:
        lines.append(f"- {empty_message}")
    lines.append("")


def latest_context_snippet():
    if not CONTEXT_MD.exists():
        return ""

    text = CONTEXT_MD.read_text(encoding="utf-8").strip()
    if not text:
        return ""

    lines = [line.strip() for line in text.splitlines() if line.strip()]
    snippet = " ".join(lines[-6:])
    if len(snippet) > 320:
        return snippet[-320:].lstrip()
    return snippet


def ensure_vault():
    VAULT_DIR.mkdir(parents=True, exist_ok=True)
    (VAULT_DIR / "Journal").mkdir(exist_ok=True)
    (VAULT_DIR / "Research").mkdir(exist_ok=True)


def build_tasks_md(tasks_data):
    tasks = tasks_data.get("tasks", [])
    counts = {status: 0 for status in KNOWN_STATUSES}

    for task in tasks:
        status = task.get("status", "unknown")
        if status in counts:
            counts[status] += 1

    lines = []
    lines.append("# Tasks")
    lines.append("")
    lines.append("## Summary")
    lines.append(f"- total: {len(tasks)}")
    lines.append(f"- queued: {counts['queued']}")
    lines.append(f"- active: {counts['active']}")
    lines.append(f"- done: {counts['done']}")
    lines.append(f"- blocked: {counts['blocked']}")
    lines.append("")
    lines.append("## Task Queue")
    for task in tasks:
        status = task.get("status", "unknown")
        mark = "x" if status == "done" else " "
        updated = task.get("updated_at")
        timestamp = f" | updated: `{updated}`" if updated else ""
        notes = task.get("notes") or []
        note_count = f" | notes: {len(notes)}" if notes else ""
        lines.append(f"- [{mark}] `{task.get('id','')}` | **{status}** | {task.get('goal','')}{timestamp}{note_count}")
    lines.append("")
    return "\n".join(lines)


def build_dashboard_md(tasks_data):
    tasks = tasks_data.get("tasks", [])
    grouped = tasks_by_status(tasks)
    counts = {status: len(grouped[status]) for status in KNOWN_STATUSES}

    context_snippet = latest_context_snippet()

    lines = []
    lines.append("# Local Quant Cowork Dashboard")
    lines.append("")
    lines.append("## Quick Links")
    lines.append("- [[Tasks]]")
    lines.append("- [[Context]]")
    lines.append("- [[Current Focus]]")
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
    append_task_section(lines, "Latest Queued Tasks", grouped["queued"], "No queued tasks.", limit=5)
    append_task_section(lines, "Recently Done Tasks", grouped["done"], "No done tasks.", limit=5)

    lines.append("## Latest Context")
    if context_snippet:
        lines.append(context_snippet)
    else:
        lines.append("No context captured yet.")
    lines.append("")
    return "\n".join(lines)


def main():
    load_env_file(REPO_ROOT / ".env")

    global VAULT_DIR
    VAULT_DIR = Path(
        os.environ.get(
            "OBSIDIAN_VAULT_DIR",
            str(VAULT_DIR),
        )
    )

    ensure_vault()
    tasks_data = load_tasks()

    tasks_md = build_tasks_md(tasks_data)
    dashboard_md = build_dashboard_md(tasks_data)
    context_md = CONTEXT_MD.read_text(encoding="utf-8") if CONTEXT_MD.exists() else "# Context\n"

    (VAULT_DIR / "Tasks.md").write_text(tasks_md, encoding="utf-8")
    (VAULT_DIR / "Dashboard.md").write_text(dashboard_md, encoding="utf-8")
    (VAULT_DIR / "Context.md").write_text(context_md, encoding="utf-8")

    print(f"Synced Obsidian vault: {VAULT_DIR}")


if __name__ == "__main__":
    main()
