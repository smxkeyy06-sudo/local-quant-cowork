#!/usr/bin/env python3
import argparse
import os
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
CURRENT_STATE_MD = REPO_ROOT / "docs" / "CURRENT_STATE.md"

EXCLUDED_STAGE_PARTS = (
    ".env",
    "__pycache__",
    ".pytest_cache",
    "target",
    "build",
    "dist",
)
EXCLUDED_STAGE_SUFFIXES = (".pyc", ".pyo", ".pyd")


def run(command, cwd=REPO_ROOT):
    print(f"+ {' '.join(command)}")
    result = subprocess.run(command, cwd=cwd, text=True)
    if result.returncode != 0:
        raise SystemExit(f"checkpoint: command failed with exit {result.returncode}: {' '.join(command)}")


def parse_args():
    parser = argparse.ArgumentParser(description="Verify, sync, and commit a local workflow checkpoint.")
    parser.add_argument("--last-pass", required=True, help="Description of the completed pass.")
    parser.add_argument("--next-pass", required=True, help="Description of the next small pass.")
    parser.add_argument("--commit-message", required=True, help="Local git commit message.")
    return parser.parse_args()


def current_state_content(last_pass, next_pass):
    return "\n".join(
        [
            "# Current State",
            "",
            "## Phase",
            "Local workflow stabilization.",
            "",
            "## Last Completed Pass",
            f"{last_pass}.",
            "",
            "## Verification Baseline",
            "- `cargo test --workspace` passes.",
            "- `cargo run -p cowork-cli -- doctor` passes.",
            "- `scripts/check_obsidian.py` works.",
            "- `scripts/sync_obsidian.py` works.",
            "- working tree should be clean after commit/push.",
            "",
            "## Next Small Pass",
            f"{next_pass}.",
            "",
            "## Not Now",
            "- no model/provider integration",
            "- no multi-agent behavior",
            "- no task schema changes",
            "- no framework redesign",
            "",
        ]
    )


def normalize_sentence(text):
    return text.strip().rstrip(".")


def write_current_state(last_pass, next_pass):
    CURRENT_STATE_MD.write_text(
        current_state_content(normalize_sentence(last_pass), normalize_sentence(next_pass)),
        encoding="utf-8",
    )
    print(f"Updated {CURRENT_STATE_MD.relative_to(REPO_ROOT)}")


def status_paths():
    result = subprocess.run(
        ["git", "status", "--porcelain"],
        cwd=REPO_ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        print(result.stderr, file=sys.stderr)
        raise SystemExit("checkpoint: failed to read git status")

    paths = []
    for line in result.stdout.splitlines():
        if not line:
            continue
        path_text = line[3:]
        if " -> " in path_text:
            path_text = path_text.split(" -> ", 1)[1]
        paths.append(Path(path_text))
    return paths


def should_stage(path):
    parts = path.parts
    if any(part in EXCLUDED_STAGE_PARTS for part in parts):
        return False
    if path.suffix in EXCLUDED_STAGE_SUFFIXES:
        return False
    return True


def stage_changes():
    paths = status_paths()
    stageable = [str(path) for path in paths if should_stage(path)]
    skipped = [str(path) for path in paths if not should_stage(path)]

    if skipped:
        print("checkpoint: skipped local/generated files:")
        for path in skipped:
            print(f"- {path}")

    if not stageable:
        raise SystemExit("checkpoint: no intended repo files to stage")

    run(["git", "add", "--", *stageable])


def ensure_staged_changes():
    result = subprocess.run(
        ["git", "diff", "--cached", "--quiet"],
        cwd=REPO_ROOT,
        text=True,
    )
    if result.returncode == 0:
        raise SystemExit("checkpoint: no staged changes to commit")
    if result.returncode != 1:
        raise SystemExit("checkpoint: failed to inspect staged changes")


def main():
    args = parse_args()
    os.chdir(REPO_ROOT)

    run(["python3", "scripts/check_obsidian.py"])
    run(["python3", "scripts/sync_obsidian.py"])
    run(["git", "diff", "--check"])
    run(["cargo", "test", "--workspace"], cwd=REPO_ROOT / "rust")
    run(["cargo", "run", "-p", "cowork-cli", "--", "doctor"], cwd=REPO_ROOT / "rust")

    write_current_state(args.last_pass, args.next_pass)
    run(["python3", "scripts/sync_obsidian.py"])

    stage_changes()
    ensure_staged_changes()
    run(["git", "commit", "-m", args.commit_message])

    print("Checkpoint committed locally. Review the commit, then run `git push` manually when ready.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
