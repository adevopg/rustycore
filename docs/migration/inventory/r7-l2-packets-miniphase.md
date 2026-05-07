# R7 L2 Packets/Dispatch Mini-Phase

> Generated: 2026-05-07
> Rule: every packet metadata change is contrasted against `/home/server/woltk-trinity-legacy/src/server/game/Server/Protocol/Opcodes.cpp` and `Opcodes.h`.

## Closed Tasks

- [x] **#NEXT.L2.DISPATCH.001** Restore C++ packet-processing metadata for touched runtime opcodes.
  C++ refs: `/home/server/woltk-trinity-legacy/src/server/game/Server/Protocol/Opcodes.h:2184`, `:2194`; `/home/server/woltk-trinity-legacy/src/server/game/Server/Protocol/Opcodes.cpp:160`, `:911`, `:966`, `:970`, `:971`, `:972`, `:978`, `:979`.
  Rust targets: `crates/wow-handler/src/lib.rs`, `crates/wow-world/src/handlers/{character,misc,trainer}.rs`, `crates/wow-world/src/session.rs`.
  Acceptance: Rust represents `PROCESS_THREADSAFE`; duplicate `TrainerList` registration is removed; `TimeSyncResponseDropped` and `TimeSyncResponseFailed` dispatch to the same handler as C++; focused tests assert C++ status/processing for the touched opcodes and reject duplicate handler registrations.

## Follow-Up Work Items

- [ ] **#NEXT.L2.DISPATCH.002** Generate or audit a complete client opcode metadata table from C++ `Opcodes.cpp` so every Rust handler registration has tested `SessionStatus` and `PacketProcessing`.
- [ ] **#NEXT.L2.PACKET.WIRE.001** Audit parsers/serializers for the login-to-world packet path against `Server/Packets/*.h`, starting with movement, object update, gossip, trainer, vendor and query packets.
