// Copyright (c) 2026 alseif0x
// RustyCore - WoW WotLK 3.4.3 server in Rust
// Based on TrinityCore protocol research (https://github.com/TrinityCore/TrinityCore)
// Licensed under GPL v3 - https://www.gnu.org/licenses/gpl-3.0.html

//! Runtime side of C++ `ConditionMgr` evaluation context.

use wow_constants::MAX_CONDITION_TARGETS;
use wow_data::Condition;
use wow_entities::WorldObject;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConditionMapRef {
    pub map_id: u32,
    pub instance_id: u32,
}

impl ConditionMapRef {
    pub const fn new(map_id: u32, instance_id: u32) -> Self {
        Self {
            map_id,
            instance_id,
        }
    }
}

#[derive(Debug)]
pub struct ConditionSourceInfo<'a> {
    pub condition_targets: [Option<&'a WorldObject>; MAX_CONDITION_TARGETS],
    pub condition_map: Option<ConditionMapRef>,
    pub last_failed_condition: Option<&'a Condition>,
}

impl<'a> ConditionSourceInfo<'a> {
    /// C++ `ConditionSourceInfo(WorldObject const*, WorldObject const*, WorldObject const*)`.
    pub fn from_targets(
        target0: Option<&'a WorldObject>,
        target1: Option<&'a WorldObject>,
        target2: Option<&'a WorldObject>,
    ) -> Self {
        let condition_targets = [target0, target1, target2];
        let condition_map = condition_targets
            .iter()
            .flatten()
            .next()
            .map(|target| ConditionMapRef::new(target.map_id(), target.instance_id()));

        Self {
            condition_targets,
            condition_map,
            last_failed_condition: None,
        }
    }

    /// C++ `ConditionSourceInfo(Map const*)`.
    pub const fn from_map(condition_map: ConditionMapRef) -> Self {
        Self {
            condition_targets: [None; MAX_CONDITION_TARGETS],
            condition_map: Some(condition_map),
            last_failed_condition: None,
        }
    }

    pub fn mark_failed_like_cpp(&mut self, condition: &'a Condition) {
        self.last_failed_condition = Some(condition);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wow_constants::{TypeId, TypeMask};

    fn world_object(map_id: u32, instance_id: u32) -> WorldObject {
        let mut object = WorldObject::new(false, TypeId::Unit, TypeMask::UNIT);
        object.set_map(map_id, instance_id).unwrap();
        object
    }

    #[test]
    fn condition_source_info_uses_first_non_null_target_map_like_cpp() {
        let target1 = world_object(571, 2);
        let target2 = world_object(1, 9);

        let info = ConditionSourceInfo::from_targets(None, Some(&target1), Some(&target2));

        assert_eq!(info.condition_targets[0].map(WorldObject::map_id), None);
        assert_eq!(
            info.condition_targets[1].map(WorldObject::map_id),
            Some(571)
        );
        assert_eq!(info.condition_map, Some(ConditionMapRef::new(571, 2)));
        assert!(info.last_failed_condition.is_none());
    }

    #[test]
    fn condition_source_info_map_constructor_matches_cpp() {
        let info = ConditionSourceInfo::from_map(ConditionMapRef::new(530, 7));

        assert!(info.condition_targets.iter().all(Option::is_none));
        assert_eq!(info.condition_map, Some(ConditionMapRef::new(530, 7)));
        assert!(info.last_failed_condition.is_none());
    }

    #[test]
    fn condition_source_info_tracks_last_failed_condition_like_cpp() {
        let condition = Condition::default();
        let mut info = ConditionSourceInfo::from_targets(None, None, None);

        info.mark_failed_like_cpp(&condition);

        assert!(std::ptr::eq(
            info.last_failed_condition.unwrap(),
            &condition
        ));
    }
}
