// Copyright (c) 2026 alseif0x
// RustyCore - WoW WotLK 3.4.3 server in Rust
// Based on TrinityCore protocol research (https://github.com/TrinityCore/TrinityCore)
// Licensed under GPL v3 - https://www.gnu.org/licenses/gpl-3.0.html

//! C++ `PhasingHandler` façade slices that are independent of runtime unit graphs.

use wow_entities::WorldObject;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhaseVisibilityUpdate {
    pub update_visibility: bool,
    pub changed: bool,
}

impl PhaseVisibilityUpdate {
    pub const fn new(update_visibility: bool, changed: bool) -> Self {
        Self {
            update_visibility,
            changed,
        }
    }
}

/// C++ `PhasingHandler::ResetPhaseShift`.
pub fn reset_phase_shift_like_cpp(object: &mut WorldObject) {
    object.phase_shift_mut().clear();
    object.suppressed_phase_shift_mut().clear();
}

/// C++ `PhasingHandler::InheritPhaseShift`.
pub fn inherit_phase_shift_like_cpp(target: &mut WorldObject, source: &WorldObject) {
    *target.phase_shift_mut() = source.phase_shift().clone();
    *target.suppressed_phase_shift_mut() = source.suppressed_phase_shift().clone();
}

/// C++ `PhasingHandler::SetAlwaysVisible`.
pub fn set_always_visible_like_cpp(
    object: &mut WorldObject,
    apply: bool,
    update_visibility: bool,
) -> PhaseVisibilityUpdate {
    object.phase_shift_mut().set_always_visible_like_cpp(apply);
    PhaseVisibilityUpdate::new(update_visibility, true)
}

/// C++ `PhasingHandler::SetInversed`.
pub fn set_inversed_like_cpp(
    object: &mut WorldObject,
    apply: bool,
    update_visibility: bool,
) -> PhaseVisibilityUpdate {
    object.phase_shift_mut().set_inversed_like_cpp(apply);
    PhaseVisibilityUpdate::new(update_visibility, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wow_constants::{PhaseFlags, PhaseShiftFlags, TypeId, TypeMask};
    use wow_core::{ObjectGuid, guid::HighGuid};

    fn world_object() -> WorldObject {
        let mut object = WorldObject::new(false, TypeId::Unit, TypeMask::UNIT);
        object.object_mut().create(ObjectGuid::create_world_object(
            HighGuid::Creature,
            0,
            0,
            571,
            0,
            1,
            1,
        ));
        object
    }

    #[test]
    fn reset_phase_shift_clears_active_and_suppressed_like_cpp() {
        let mut object = world_object();
        object
            .phase_shift_mut()
            .add_phase_like_cpp(10, PhaseFlags::NONE, 1);
        object
            .suppressed_phase_shift_mut()
            .add_phase_like_cpp(20, PhaseFlags::NONE, 1);

        reset_phase_shift_like_cpp(&mut object);

        assert!(
            object
                .phase_shift()
                .flags_like_cpp()
                .contains(PhaseShiftFlags::UNPHASED)
        );
        assert!(
            object
                .suppressed_phase_shift()
                .flags_like_cpp()
                .contains(PhaseShiftFlags::UNPHASED)
        );
        assert!(!object.phase_shift().has_phase_like_cpp(10));
        assert!(!object.suppressed_phase_shift().has_phase_like_cpp(20));
    }

    #[test]
    fn inherit_phase_shift_copies_active_and_suppressed_like_cpp() {
        let mut source = world_object();
        let mut target = world_object();
        source
            .phase_shift_mut()
            .add_phase_like_cpp(10, PhaseFlags::NONE, 1);
        source
            .suppressed_phase_shift_mut()
            .add_phase_like_cpp(20, PhaseFlags::NONE, 1);

        inherit_phase_shift_like_cpp(&mut target, &source);

        assert!(target.phase_shift().has_phase_like_cpp(10));
        assert!(target.suppressed_phase_shift().has_phase_like_cpp(20));
    }

    #[test]
    fn set_visibility_flags_match_cpp_and_report_update_request() {
        let mut object = world_object();

        let update = set_always_visible_like_cpp(&mut object, true, true);
        assert_eq!(update, PhaseVisibilityUpdate::new(true, true));
        assert!(
            object
                .phase_shift()
                .flags_like_cpp()
                .contains(PhaseShiftFlags::ALWAYS_VISIBLE)
        );

        let update = set_inversed_like_cpp(&mut object, true, false);
        assert_eq!(update, PhaseVisibilityUpdate::new(false, true));
        assert!(
            object
                .phase_shift()
                .flags_like_cpp()
                .contains(PhaseShiftFlags::INVERSE)
        );
        assert!(
            object
                .phase_shift()
                .flags_like_cpp()
                .contains(PhaseShiftFlags::INVERSE_UNPHASED)
        );
        assert!(
            !object
                .phase_shift()
                .flags_like_cpp()
                .contains(PhaseShiftFlags::UNPHASED)
        );
    }
}
