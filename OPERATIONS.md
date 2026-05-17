# OPERATIONS

## Start Of Session
```bash
cd ~/local-quant-cowork
git pull
python3 scripts/verify_session.py
python3 scripts/check_obsidian.py
python3 scripts/sync_obsidian.py
git status
```

`scripts/verify_session.py` is read-only. `scripts/check_obsidian.py` verifies
Obsidian vault write access, and `scripts/sync_obsidian.py` regenerates the
Obsidian-readable view. `docs/CURRENT_STATE.md` remains the source checkpoint;
Obsidian `CurrentState.md` is generated from it. After checkpointing, pushing
remains manual.

## Workflow
1. run `cowork doctor`
2. append context notes with `cowork context "<note>"`
3. append tasks with `cowork task "<goal>"`
4. append task notes with `cowork task-note <task-id> "<note>"`
5. update status with `cowork task-status <task-id> <status>`
6. list tasks with `cowork tasks` or `cowork tasks --status queued`
7. commit working changes

Context append, task append, task notes, and status updates auto-run
`scripts/sync_obsidian.py` when present. Obsidian sync writes `Dashboard.md`,
`Tasks.md`, `Context.md`, and `CurrentState.md`; `CurrentState.md` is generated
from `docs/CURRENT_STATE.md`. The dashboard summarizes status counts and the
current task queues.

Run `python3 scripts/generate_daily.py` to create or refresh today's Obsidian
journal note at `Journal/YYYY-MM-DD.md`.

Run `python3 scripts/check_obsidian.py` to verify that `OBSIDIAN_VAULT_DIR` is
set, exists, and allows temporary write/delete access without modifying vault
notes.

## Checkpoint
Run `python3 scripts/checkpoint.py --last-pass "<description>" --next-pass "<description>" --commit-message "<message>"`
from the repository root at the end of a pass. The checkpoint helper verifies
Obsidian access, syncs the vault, runs `git diff --check`, runs the Rust test
and doctor checks, updates `docs/CURRENT_STATE.md`, refreshes Obsidian
`CurrentState.md`, stages intended repo files, and creates a local git commit.
It does not push; review the commit and run `git push` manually.
