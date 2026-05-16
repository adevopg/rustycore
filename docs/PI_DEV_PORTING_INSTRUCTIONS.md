# pi.dev Porting Instructions

Concise operating rules for RustyCore C++ -> Rust work in pi.dev.
Full methodology: `docs/CPP_TO_RUST_PORTING_METHODOLOGY.md`.

## Source of truth

- Canonical reference: `/home/server/woltk-trinity-legacy`.
- Existing Rust, previous AI work, tests, and roadmap entries are not proof.
- Do not start the server or push unless explicitly requested.
- Every testable implementation/hito must be committed separately.

## Required loop for every change

1. **Find C++ first**
   - Record exact path, function, and line range.
   - Inspect callees when behavior lives outside the top-level handler.

2. **Define scope**
   - Label as `wire`, `state`, `runtime`, `side_effect`, `bridge`, or `scaffold`.
   - Only `runtime`/real side effects can close gameplay behavior.
   - `scaffold`/snapshots must be documented as partial.

3. **Map ownership**
   - Prefer canonical `wow_entities` / `wow_map` state.
   - Use `WorldSession`/legacy mirrors only as explicit fallback.
   - If canonical state exists and says `None`, never fallback to stale mirrors.

4. **Implement narrowly**
   - Preserve C++ branch order, early returns, state writes, packets, callbacks,
     timers, cleanup, and side effects.
   - Do not fake missing runtime with defaults that only make tests pass.

5. **Test C++ behavior**
   - Add positive tests and negative tests for representable C++ early returns.
   - Assert non-effects when C++ returns early: no damage, no packet, no timer
     reset, no threat, no target/attacker mutation, as applicable.

6. **Update roadmap honestly**
   - `[x]` only for full parity of the stated scope.
   - Use `represented`, `partial`, `blocked`, or TODO for incomplete runtime.

7. **Check and commit**
   ```bash
   cargo test -p <crate> --lib <focused_filter>
   cargo fmt --check
   git diff --check
   cargo check -p world-server
   git status --short
   ```
   Commit immediately after the hito passes.

## C++ anchor note template

```text
C++ anchor:
/home/server/woltk-trinity-legacy/src/server/game/...
Function::Name, lines X-Y
Ported branch/side effect: ...
Scope: runtime|side_effect|state|wire|bridge|scaffold
```

## Audit before new work

Current audit base:

```text
trusted base: f7eb5dc
review range: f7eb5dc..HEAD
```

Before further porting, review recent work by behavior:

```bash
git diff --stat f7eb5dc..HEAD
git diff --name-status f7eb5dc..HEAD
git diff f7eb5dc..HEAD -- docs/MIGRATION_ROADMAP.md
rg -n "tick_combat_sync|DoMeleeAttackIfReady|AttackerStateUpdate|IsValidAttackTarget|UnitAttackContextLikeCpp|Threat|AttackSwingError" crates docs
```

Review format:

```text
Claim:
C++ anchor:
Rust location:
Verdict: accept / partial / bug
Finding:
Required fix:
Checks:
```

## Immediate audit targets

- `tick_combat_sync`
- `start_player_attack_like_cpp` / `stop_player_attack_like_cpp`
- `UnitAttackContextLikeCpp` and represented `IsValidAttackTarget`
- threat accounting and cleanup
- attackers tracking
- `CURRENT_MELEE_SPELL`
- swing errors
- `MELEE_ATTACKING` and `CHARGING` guards

## Reporting to user

```text
Done/audited:
C++ anchors:
Findings/fixes:
Checks:
Commit:
Progress estimate:
```
