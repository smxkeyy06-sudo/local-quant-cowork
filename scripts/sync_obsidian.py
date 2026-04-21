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


def ensure_vault():
    VAULT_DIR.mkdir(parents=True, exist_ok=True)
    (VAULT_DIR / "Journal").mkdir(exist_ok=True)
    (VAULT_DIR / "Research").mkdir(exist_ok=True)


def build_tasks_md(tasks_data):
    tasks = tasks_data.get("tasks", [])
    counts = {"queued": 0, "active": 0, "done": 0, "blocked": 0}

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
        lines.append(f"- [{mark}] `{task.get('id','')}` | **{status}** | {task.get('goal','')}")
    lines.append("")
    return "\n".join(lines)


def build_dashboard_md(tasks_data):
    tasks = tasks_data.get("tasks", [])
    counts = {"queued": 0, "active": 0, "done": 0, "blocked": 0}

    for task in tasks:
        status = task.get("status", "unknown")
        if status in counts:
            counts[status] += 1

    latest = list(reversed(tasks[-5:]))

    lines = []
    lines.append("# Local Quant Cowork Dashboard")
    lines.append("")
    lines.append("## Repo")
    lines.append("- local-quant-cowork")
    lines.append("")
    lines.append("## Task Counts")
    lines.append(f"- queued: {counts['queued']}")
    lines.append(f"- active: {counts['active']}")
    lines.append(f"- done: {counts['done']}")
    lines.append(f"- blocked: {counts['blocked']}")
    lines.append("")
    lines.append("## Quick Links")
    lines.append("- [[Tasks]]")
    lines.append("- [[Context]]")
    lines.append("- [[Current Focus]]")
    lines.append("")
    lines.append("## Latest Tasks")
    for task in latest:
        lines.append(f"- `{task.get('id','')}` — {task.get('goal','')}")
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
    context_md = CONTEXT_MD.read_text(encoding="utf-8")

    (VAULT_DIR / "Tasks.md").write_text(tasks_md, encoding="utf-8")
    (VAULT_DIR / "Dashboard.md").write_text(dashboard_md, encoding="utf-8")
    (VAULT_DIR / "Context.md").write_text(context_md, encoding="utf-8")

    print(f"Synced Obsidian vault: {VAULT_DIR}")


if __name__ == "__main__":
    main()
