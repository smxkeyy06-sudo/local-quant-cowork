#!/usr/bin/env python3
import json
import os
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
CURRENT_STATE_MD = REPO_ROOT / "docs" / "CURRENT_STATE.md"
TASKS_JSON = REPO_ROOT / "cowork" / "tasks.json"
CHECK_OBSIDIAN_PY = REPO_ROOT / "scripts" / "check_obsidian.py"
SYNC_OBSIDIAN_PY = REPO_ROOT / "scripts" / "sync_obsidian.py"
ENV_FILE = REPO_ROOT / ".env"
VAULT_ENV_KEY = "OBSIDIAN_VAULT_DIR"


class Report:
    def __init__(self):
        self.failed = False

    def pass_(self, label, detail):
        print(f"PASS {label}: {detail}")

    def fail(self, label, detail):
        self.failed = True
        print(f"FAIL {label}: {detail}")


def load_env_file(path):
    values = {}
    if not path.exists():
        return values

    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        key = key.strip()
        value = value.strip().strip("'\"")
        if key:
            values[key] = value
    return values


def run_read_check(command, cwd):
    return subprocess.run(
        command,
        cwd=cwd,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )


def check_repo_root(report):
    cwd = Path.cwd().resolve()
    result = run_read_check(["git", "rev-parse", "--show-toplevel"], REPO_ROOT)
    if result.returncode != 0:
        report.fail("repo root", result.stderr.strip() or "git rev-parse --show-toplevel failed")
        return

    git_repo_root = Path(result.stdout.strip()).resolve()
    if git_repo_root != REPO_ROOT:
        report.fail("repo root", f"expected {REPO_ROOT}, git reported {git_repo_root}")
        return

    detail = f"detected {git_repo_root}"
    if cwd == REPO_ROOT:
        detail += " from current working directory"
    else:
        detail += f" while running from {cwd}"
    report.pass_("repo root", detail)


def check_file_exists(report, label, path):
    if path.is_file():
        report.pass_(label, str(path.relative_to(REPO_ROOT)))
    else:
        report.fail(label, f"missing {path.relative_to(REPO_ROOT)}")


def check_tasks_json(report):
    if not TASKS_JSON.is_file():
        report.fail("tasks data", "missing cowork/tasks.json")
        return

    try:
        with TASKS_JSON.open("r", encoding="utf-8") as handle:
            json.load(handle)
    except (OSError, json.JSONDecodeError) as exc:
        report.fail("tasks data", f"could not parse cowork/tasks.json: {exc}")
        return

    report.pass_("tasks data", "cowork/tasks.json exists and parses")


def check_git(report):
    status = run_read_check(["git", "status", "--short"], REPO_ROOT)
    if status.returncode == 0:
        detail = "git status readable"
        if status.stdout.strip():
            detail += "; working tree has changes"
        else:
            detail += "; working tree clean"
        report.pass_("git status", detail)
    else:
        report.fail("git status", status.stderr.strip() or "git status failed")

    commits = run_read_check(["git", "log", "-3", "--oneline"], REPO_ROOT)
    if commits.returncode == 0:
        lines = [line for line in commits.stdout.splitlines() if line.strip()]
        report.pass_("recent commits", f"read {len(lines)} commit(s)")
    else:
        report.fail("recent commits", commits.stderr.strip() or "git log failed")


def check_obsidian_config(report):
    env_values = load_env_file(ENV_FILE)
    raw_vault_dir = os.environ.get(VAULT_ENV_KEY, env_values.get(VAULT_ENV_KEY, "")).strip()
    if not raw_vault_dir:
        report.fail("Obsidian vault", f"{VAULT_ENV_KEY} is not set")
        return

    vault_dir = Path(raw_vault_dir).expanduser()
    if vault_dir.is_dir():
        report.pass_("Obsidian vault", f"configured directory exists: {vault_dir}")
    else:
        report.fail("Obsidian vault", f"configured directory does not exist: {vault_dir}")


def check_command(report, label, command, cwd):
    result = run_read_check(command, cwd)
    if result.returncode == 0:
        report.pass_(label, "passed")
    else:
        detail = result.stderr.strip() or result.stdout.strip() or f"exit {result.returncode}"
        report.fail(label, detail)


def main():
    report = Report()

    print("Session Verification")
    print("")
    print("Repository")
    check_repo_root(report)
    check_file_exists(report, "current state", CURRENT_STATE_MD)
    check_tasks_json(report)
    check_git(report)

    print("")
    print("Obsidian")
    check_file_exists(report, "check helper", CHECK_OBSIDIAN_PY)
    check_file_exists(report, "sync helper", SYNC_OBSIDIAN_PY)
    check_obsidian_config(report)
    report.pass_("write check", "dedicated scripts/check_obsidian.py exists; not run by this read-only verifier")

    print("")
    print("Rust Verification")
    check_command(report, "cargo test --workspace", ["cargo", "test", "--workspace"], REPO_ROOT / "rust")
    check_command(
        report,
        "cargo run -p cowork-cli -- doctor",
        ["cargo", "run", "-p", "cowork-cli", "--", "doctor"],
        REPO_ROOT / "rust",
    )

    print("")
    if report.failed:
        print("Session verification failed.")
        return 1

    print("Session verification passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
