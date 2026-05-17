# Current Session Handoff

Generated: 2026-05-17

This file is a continuity snapshot for the RustyCore C++ -> Rust migration.
It records the current working state, recent completed slices, validation
status, and next recommended steps.

## Repository State

Workspace:

```text
/home/server/rustycore
```

Branch:

```text
develop
```

Current HEAD:

```text
02761d6 Wire fishing base skill lookup
```

Remote relation at the time of this handoff:

```text
develop...origin/develop
```

Current tree status:

```text
 M crates/world-server/src/main.rs
 M crates/wow-constants/src/opcodes.rs
 M crates/wow-data/src/skill_talent.rs
 M crates/wow-entities/src/game_object.rs
 M crates/wow-entities/src/lib.rs
 M crates/wow-map/src/map.rs
 M crates/wow-network/src/accept.rs
 M crates/wow-packet/src/packets/misc.rs
 M crates/wow-world/src/handlers/loot.rs
 M crates/wow-world/src/session.rs
 M docs/migration/inventory/r8-entities-miniphase.md
 M docs/migration/inventory/r8-entities-miniphase.tsv
 M docs/migration/loot.md
 M docs/migration/skills.md
?? docs/migration/harness-agent-strategy.md
?? docs/migration/current-session-handoff.md
```

Approximate diff size before this handoff file:

```text
14 files changed, 3048 insertions(+), 304 deletions(-)
```

This is an accumulated development batch and has not been committed yet.

## Critical Rules

Do not trust existing Rust implementation.

Do not trust previous AI output, roadmap checkboxes, comments, or tests as
proof.

Every migrated behavior must be contrasted against C++ first:

```text
/home/server/woltk-trinity-legacy
```

Use archived TrinityCore 3.3.5 only as a secondary logic reference when the
legacy source is incomplete or suspicious. Do not copy code from it.

Do not start live servers unless the user explicitly requests it.

Do not push, merge, or commit unless the user explicitly asks or a
solidification step has been agreed.

Do not revert unrelated dirty-tree changes.

## Progress Estimate

Overall migration estimate at this handoff:

```text
~80%
```

This is overall project progress, not just the current GameObject runtime
microphase.

Manual test point:

```text
No new manual test point from the latest changes.
```

Previous simple login/client smoke was proven earlier by the user. The current
work since then is represented runtime and unit/integration checked, not a new
client-facing milestone.

## Most Recent Completed Slices

The current batch continues the R8 entities GameObject runtime work. The latest
closed slices are recorded in:

```text
docs/migration/inventory/r8-entities-miniphase.tsv
docs/migration/inventory/r8-entities-miniphase.md
```

Recent items completed in this batch:

- `#NEXT.R8.ENTITIES.339`
  - Non-bomb trap `GO_NOT_READY` branch.
  - C++ anchor: `GameObject.cpp`.
  - Behavior: non-bomb traps move to `GO_READY`; owner-in-combat arms
    `trap.startDelay`.

- `#NEXT.R8.ENTITIES.340`
  - Generic `GO_NOT_READY -> GO_READY` default branch.
  - Excludes trap, fishing bobber and chest, which have special C++ branches.

- `#NEXT.R8.ENTITIES.341`
  - Shared chest restock.
  - Behavior: partial non-consumable chests stay `GO_ACTIVATED` with restock
    timer; fully looted shared non-consumable chests move to `GO_NOT_READY`;
    expired restock returns to `GO_READY` and clears represented loot.

- `#NEXT.R8.ENTITIES.342`
  - Gathering-node `ObjectDespawnDelay`.
  - Behavior: first activation arms represented despawn timer; expired timer
    moves object to `GO_NOT_READY`, resets `GO_STATE_READY` for non-transports,
    clears represented loot/visibility and sends represented despawn/destroy
    packets.
  - Added `GAMEOBJECT_TYPE_TRANSPORT = 11` from C++ `SharedDefines.h`.

- `#NEXT.R8.ENTITIES.343`
  - Fishing bobber `GO_NOT_READY` ready branch.
  - Behavior: represented bobbers move to `GO_READY` when ready timestamp
    expires; records owner splash-animation hook.

- `#NEXT.R8.ENTITIES.344`
  - Non-bomb trap `GO_READY/GO_ACTIVATED` target path.
  - Behavior: ready traps respect cooldown, use `radius / 2` for environmental
    current-player activation or explicit represented target GUID for
    owner/check-all-units traps, store target in `loot_state_unit_guid`, then
    activated traps record target spell casts with original caster, arm
    template-or-4s cooldown, and apply C++ charges split.

- `#NEXT.R8.ENTITIES.345`
  - Generic non-goober `GO_JUST_DEACTIVATED` cleanup.
  - Behavior: clear represented loot, reset `use_count` and
    `loot_state_unit_guid`, move to `GO_NOT_READY`, delete owner/summoned
    represented objects with despawn/destroy packets.

- `#NEXT.R8.ENTITIES.346`
  - `respawnDelayTime` represented handling.
  - Behavior: spawned-by-default objects leave represented visibility with a
    respawn timer and return to `GO_READY`/`GO_STATE_READY`; non-spawned
    represented objects delete instead of scheduling respawn.

Also created:

```text
docs/migration/harness-agent-strategy.md
```

That document defines the proposed multi-agent harness strategy through the
rest of the migration.

## Validation Already Run

Focused validations run successfully during the latest work:

```bash
cargo test -p wow-world chest_restock
cargo test -p wow-world gameobject_despawn_delay
cargo test -p wow-world gathering_node_runtime_state
cargo test -p wow-world fishing_bobber_ready
cargo test -p wow-world non_bomb_trap
cargo test -p wow-world generic_just_deactivated_gameobject
cargo test -p wow-world gameobject_respawn_delay
```

Integration validation run successfully after the latest code changes:

```bash
cargo check -p world-server
git diff --check
```

Expected warnings remain in existing crates. They were not introduced as
blocking errors during this session.

Before committing or continuing after a context switch, rerun:

```bash
cargo fmt
cargo check -p world-server
git diff --check
git status --short --branch
```

If touching GameObject/trap/loot runtime again, also rerun a focused subset:

```bash
cargo test -p wow-world trap
cargo test -p wow-world chest_restock
cargo test -p wow-world gameobject_respawn_delay
```

## Important C++ Anchors Used Recently

Main C++ file:

```text
/home/server/woltk-trinity-legacy/src/server/game/Entities/GameObject/GameObject.cpp
```

Key branches:

- `GameObject::Update`
  - `GO_NOT_READY`
  - `GO_READY`
  - `GO_ACTIVATED`
  - `GO_JUST_DEACTIVATED`
- `GameObject::SetLootState`
- `GameObject::DespawnOrUnsummon`
- `GameObject::Delete`

Additional C++ files:

```text
/home/server/woltk-trinity-legacy/src/server/game/Entities/GameObject/GameObject.h
/home/server/woltk-trinity-legacy/src/server/game/Miscellaneous/SharedDefines.h
/home/server/woltk-trinity-legacy/src/server/game/Entities/GameObject/GameObjectData.h
```

Relevant constants confirmed:

- `FISHING_BOBBER_READY_TIME = 5`
- `GAMEOBJECT_TYPE_TRANSPORT = 11`
- `GAMEOBJECT_TYPE_FISHINGNODE = 17`
- `GAMEOBJECT_TYPE_GATHERING_NODE = 50`

## Current Implementation Shape

Most current GameObject runtime work is still represented/session bridge work,
centered in:

```text
crates/wow-world/src/session.rs
crates/wow-world/src/handlers/loot.rs
crates/wow-entities/src/game_object.rs
```

The bridge is intentionally honest:

- It mirrors C++ state transitions where canonical ownership is not complete.
- It records hooks/effects where real cross-session fanout, ObjectAccessor,
  spell execution, AI, or map removal is not fully present.
- It does not claim full canonical GameObject runtime ownership.

Major represented fields added or extended recently include:

- `owner_in_combat`
- `chest_restock_until`
- `despawn_delay_until`
- `respawn_delay_secs`
- `respawn_until`
- `spawned_by_default`
- `fishing_bobber_ready_at`
- `trap_target_guid`

Recent represented effects include:

- `FishingBobberReady`
- `TrapTargetActivated`
- `TrapTargetSpellCast`
- `GameObjectJustDeactivatedCleared`

## Known Remaining Gaps in This Area

Do not mark GameObject runtime complete yet.

Important remaining gaps include:

- Real canonical `GameObject::m_loot` ownership.
- Real `ObjectAccessor` unit/player search for traps.
- Hostility filtering for owner/check-all-units trap target selection.
- Totem and LOS filtering for trap target search.
- Real `GameObject::CastSpell` semantics.
- Cross-session packet fanout for despawn, destroy, visibility, dynamic flags.
- Real linked-trap lookup and despawn.
- `GameObjectOverride` flag restoration.
- `IsDespawnAtAction` and `goAnimProgress` despawn visuals.
- Dynamic respawn scaling.
- `SaveRespawnTime` persistence.
- `respawnCompatibilityMode` visibility destroy path.
- Map `AddObjectToRemoveList` and canonical map removal.
- Pool refresh.
- Spell-created bobber full lifecycle, owner/channel provenance and actual
  `SendCustomAnim` fanout.
- Full canonical GameObject update ownership outside `WorldSession`.

## Recommended Next Step

Continue from `#NEXT.R8.ENTITIES.021` GameObject runtime lifecycle.

Recommended next microphase:

1. Re-open `GameObject.cpp::GO_JUST_DEACTIVATED`.
2. Contrast unported branches:
   - linked trap lookup/despawn
   - `IsDespawnAtAction`
   - `goAnimProgress > 0`
   - respawn compatibility
   - dynamic respawn scaling / save respawn
3. Decide whether each is representable now or must stay as a canonical map
   ownership TODO.
4. If implementable, add one narrow represented branch plus test.
5. Update `r8-entities-miniphase.tsv` with the next ID after `#NEXT.R8.ENTITIES.346`.

Alternative next microphase:

Move from represented GameObject update bridge toward canonical ownership:

- inspect `crates/wow-entities/src/game_object.rs`
- inspect `crates/wow-map/src/map.rs`
- identify the smallest step to let canonical `GameObject` own one timer/state
  currently stored in `WorldSession`.

This is higher risk but moves closer to full port closure.

## Harness Usage Recommendation

Use:

```text
Explorers in parallel, coordinator writes, reviewer audits.
```

Reason:

The next likely work still touches `crates/wow-world/src/session.rs`, which is
large and conflict-prone. Multiple writers in that file are inefficient.

Suggested subagent layout:

- C++ Explorer high:
  - read only `/home/server/woltk-trinity-legacy`
  - extract exact behavior and line anchors

- Rust Explorer high:
  - read current Rust implementation
  - identify current bridge fields/tests/gaps

- Reviewer high:
  - after patch, compare final diff against C++ anchors

Coordinator should implement the patch unless the write scope is isolated.

Full harness methodology:

```text
docs/migration/harness-agent-strategy.md
```

## Solidification Option

The current batch is large. It may be worth solidifying before more work.

Suggested solidification sequence:

```bash
cargo fmt
cargo check -p world-server
git diff --check
git status --short --branch
git diff --stat
```

Then review the diff by domain:

```bash
git diff -- crates/wow-world/src/session.rs
git diff -- crates/wow-world/src/handlers/loot.rs
git diff -- crates/wow-entities/src/game_object.rs
git diff -- docs/migration/inventory/r8-entities-miniphase.tsv
```

If the user asks to commit:

- Commit the current port batch and docs together only after validation passes.
- Use a message like:

```text
port: extend represented gameobject update lifecycle
```

Do not push or merge unless explicitly requested.

## User Preferences to Preserve

- Spanish communication.
- Keep reporting progress percentage periodically.
- Report when a manual test point is ready.
- Do not trust other AI work without audit.
- Always contrast with C++ before accepting Rust behavior.
- Complete port means no gaps; represented bridges must be documented as
  partial until canonical ownership exists.
- User accepts harness/subagents when useful, but principal agent should
  coordinate and verify.

## Quick Resume Prompt

Use this if continuing in another session:

```text
Estamos en /home/server/rustycore, rama develop. Lee primero
docs/migration/current-session-handoff.md,
docs/CPP_TO_RUST_PORTING_METHODOLOGY.md y
docs/migration/harness-agent-strategy.md.

Regla crítica: no te fíes de Rust ni de trabajo previo de IA; contrasta siempre
contra /home/server/woltk-trinity-legacy antes de cerrar cualquier behavior.

Hay un batch grande sin commit con GameObject runtime representado hasta
#NEXT.R8.ENTITIES.346. Revalida con cargo check -p world-server y git
diff --check antes de seguir. Si continúas, sigue con el siguiente gap de
GameObject::Update / GO_JUST_DEACTIVATED o plantea solidificar el árbol.
```
