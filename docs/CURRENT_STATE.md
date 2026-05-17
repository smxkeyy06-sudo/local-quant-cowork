# Current State

## Phase
Local workflow stabilization.

## Last Completed Pass
Documented standard start-of-session workflow using verify_session.py.

## Verification Baseline
- `cargo test --workspace` passes.
- `cargo run -p cowork-cli -- doctor` passes.
- `scripts/check_obsidian.py` works.
- `scripts/sync_obsidian.py` works.
- working tree should be clean after commit/push.

## Next Small Pass
Review local workflow docs for any stale pre-verifier commands.

## Not Now
- no model/provider integration
- no multi-agent behavior
- no task schema changes
- no framework redesign
