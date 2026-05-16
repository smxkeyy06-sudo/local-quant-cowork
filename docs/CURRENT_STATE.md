# Current State

## Phase
Local workflow stabilization.

## Last Completed Pass
Added read-only start-of-session verification helper.

## Verification Baseline
- `cargo test --workspace` passes.
- `cargo run -p cowork-cli -- doctor` passes.
- `scripts/check_obsidian.py` works.
- `scripts/sync_obsidian.py` works.
- working tree should be clean after commit/push.

## Next Small Pass
Use verify_session.py at the start of the next work session and tighten any weak checks found in practice.

## Not Now
- no model/provider integration
- no multi-agent behavior
- no task schema changes
- no framework redesign
