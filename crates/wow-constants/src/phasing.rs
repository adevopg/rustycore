// Copyright (c) 2026 alseif0x
// RustyCore - WoW WotLK 3.4.3 server in Rust
// Based on TrinityCore protocol research (https://github.com/TrinityCore/TrinityCore)
// Licensed under GPL v3 - https://www.gnu.org/licenses/gpl-3.0.html

//! Phasing flags ported from TrinityCore `PhaseShift.h`.

use bitflags::bitflags;

bitflags! {
    /// C++ `PhaseShiftFlags`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PhaseShiftFlags: u32 {
        const NONE = 0x00;
        const ALWAYS_VISIBLE = 0x01;
        const INVERSE = 0x02;
        const INVERSE_UNPHASED = 0x04;
        const UNPHASED = 0x08;
        const NO_COSMETIC = 0x10;
    }
}

bitflags! {
    /// C++ `PhaseFlags`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PhaseFlags: u16 {
        const NONE = 0x0;
        const COSMETIC = 0x1;
        const PERSONAL = 0x2;
    }
}
