#!/usr/bin/env python3
import os
import sys
from pathlib import Path
from uuid import uuid4

REPO_ROOT = Path(__file__).resolve().parents[1]
ENV_FILE = REPO_ROOT / ".env"
ENV_KEY = "OBSIDIAN_VAULT_DIR"


def load_env_file(path: Path) -> None:
    if not path.exists():
        return

    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError as exc:
        print(f"check_obsidian: error reading {path}: {exc}", file=sys.stderr)
        raise SystemExit(1)

    for raw_line in lines:
        line = raw_line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        key = key.strip()
        value = value.strip().strip("'\"")
        if key and key not in os.environ:
            os.environ[key] = value


def main() -> int:
    load_env_file(ENV_FILE)

    raw_vault_dir = os.environ.get(ENV_KEY, "").strip()
    if not raw_vault_dir:
        print(f"check_obsidian: error: {ENV_KEY} is not set.", file=sys.stderr)
        return 1

    vault_dir = Path(raw_vault_dir).expanduser()
    if not vault_dir.exists():
        print(f"check_obsidian: error: vault directory does not exist: {vault_dir}", file=sys.stderr)
        return 1
    if not vault_dir.is_dir():
        print(f"check_obsidian: error: vault path is not a directory: {vault_dir}", file=sys.stderr)
        return 1

    check_path = vault_dir / f".local-quant-cowork-write-check-{uuid4().hex}.tmp"
    try:
        check_path.write_text("local-quant-cowork obsidian write check\n", encoding="utf-8")
        check_path.unlink()
    except OSError as exc:
        print(f"check_obsidian: error: vault is not writable or accessible: {vault_dir}: {exc}", file=sys.stderr)
        try:
            if check_path.exists():
                check_path.unlink()
        except OSError:
            pass
        return 1

    print(f"Obsidian vault is writable: {vault_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
