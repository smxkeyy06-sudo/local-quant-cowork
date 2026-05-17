# Current State

## Phase
Local workflow stabilization.

## Last Completed Pass
Strengthened read-only session verifier repo root detection.

## Verification Baseline
- `cargo test --workspace` passes.
- `cargo run -p cowork-cli -- doctor` passes.
- `scripts/check_obsidian.py` works.
- `scripts/sync_obsidian.py` works.
- working tree should be clean after commit/push.

## Next Small Pass
Document the standard start-of-session workflow using verify_session.py.

## Not Now
- no model/provider integration
- no multi-agent behavior
- no task schema changes
- no framework redesign
