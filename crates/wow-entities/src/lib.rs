//! Canonical entity model.
//!
//! C++ refs:
//! - `game/Entities/Object/Object.h`
//! - `game/Entities/Object/Object.cpp`
//! - `game/Entities/Object/ObjectGuid.h`

mod object;
mod object_accessor;
mod unit;
mod update_fields;
mod world_object;

pub use object::{CreateObjectFlags, EntityObject, EntityObjectState, ObjectChangedFields};
pub use object_accessor::{
    AccessorObjectKind, AccessorPlayer, MapObjectRecord, ObjectAccessor, ObjectAccessorError,
    normalize_player_name,
};
pub use unit::{
    BASE_MAXDAMAGE, BASE_MINDAMAGE, BASE_MOVE_SPEED, DEFAULT_PLAYER_DISPLAY_SCALE, MAX_ATTACK,
    MAX_MOVE_TYPE, MAX_POWERS, MAX_POWERS_PER_CLASS, UNIT_DATA_BOUNDING_RADIUS_BIT,
    UNIT_DATA_COMBAT_REACH_BIT, UNIT_DATA_DISPLAY_ID_BIT, UNIT_DATA_DISPLAY_POWER_BIT,
    UNIT_DATA_DISPLAY_SCALE_BIT, UNIT_DATA_FACTION_TEMPLATE_BIT, UNIT_DATA_FLAGS_BIT,
    UNIT_DATA_FLAGS2_BIT, UNIT_DATA_FLAGS3_BIT, UNIT_DATA_HEALTH_BIT, UNIT_DATA_LEVEL_BIT,
    UNIT_DATA_MAX_HEALTH_BIT, UNIT_DATA_MAX_POWER_FIRST_BIT, UNIT_DATA_NATIVE_DISPLAY_ID_BIT,
    UNIT_DATA_NATIVE_DISPLAY_SCALE_BIT, UNIT_DATA_PARENT_BIT, UNIT_DATA_POWER_FIRST_BIT,
    UNIT_DATA_POWER_PARENT_BIT, Unit, UnitDataUpdate, UnitDataValues, UnitValuesUpdate,
};
pub use update_fields::{
    NUM_CLIENT_OBJECT_TYPES, OBJECT_DATA_BITS, OBJECT_DATA_DYNAMIC_FLAGS_BIT,
    OBJECT_DATA_ENTRY_ID_BIT, OBJECT_DATA_PARENT_BIT, OBJECT_DATA_SCALE_BIT, ObjectDataUpdate,
    ObjectDataValues, TYPEID_OBJECT, TYPEID_UNIT, UNIT_DATA_BITS, UpdateMask, ValuesUpdate,
};
pub use world_object::{MAPID_INVALID, MapBindingError, PhaseShift, WorldLocation, WorldObject};
