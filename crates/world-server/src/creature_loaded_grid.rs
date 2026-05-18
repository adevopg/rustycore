// Copyright (c) 2026 alseif0x
// RustyCore — WoW WotLK 3.4.3 server in Rust
// Licensed under GPL v3 — https://www.gnu.org/licenses/gpl-3.0.html

//! Pure loaded-grid Creature lifecycle resolver for the real map insertion path.
//!
//! C++ anchors:
//! - `/home/server/woltk-trinity-legacy/src/server/game/Entities/Creature/Creature.cpp:1770-1813`
//!   `Creature::CreateFromProto`: template lookup/original entry, creature/vehicle high GUID,
//!   `UpdateEntry`, optional vehicle kit.
//! - `/home/server/woltk-trinity-legacy/src/server/game/Entities/Creature/Creature.cpp:1815-1923`
//!   `Creature::LoadFromDB`: caller/Map ownership handles duplicate/alive guard; resolved
//!   `CreatureData` drives spawn id, respawn compatibility, creature data, wander/respawn,
//!   `Create`, home position, inactive group gates, `SetSpawnHealth`, movement/string id,
//!   optional `AddToMap`.
//! - `/home/server/woltk-trinity-legacy/src/server/game/Entities/Creature/Creature.cpp:333-350`
//!   `Creature::AddToWorld`: map object store/spawn-id multimap plus formation/AI/vehicle/script hooks;
//!   this resolver only produces the typed record for that owner.
//! - `/home/server/woltk-trinity-legacy/src/server/game/Grids/ObjectGridLoader.cpp:44-78`
//!   loaded grid helper creates an object and calls `LoadFromDB`.
//! - `/home/server/woltk-trinity-legacy/src/server/game/Maps/Map.cpp:519-542`
//!   `Map::AddToMap`: Map creates/binds/adds object and runs object-level `AddToWorld`.
//!
//! Ownership: DB/template/spawn caches are resolved by the caller before taking a `MapManager`/`Map`
//! lock. This module performs no async work, no DB lookups, no live-map mutation, and no fanout.
//! Sync direction is DB/template/spawn-store -> lifecycle record -> `Creature` -> `MapObjectRecord`.

use std::collections::BTreeMap;

use wow_core::{ObjectGuid, Position, guid::HighGuid};
use wow_entities::{
    Creature, CreatureCreateLifecycleRecord, CreatureLifecycleStats,
    CreatureLoadFromDbLifecycleRecord, CreatureModelDimensions, CreatureSpawnLifecycleRecord,
    CreatureTemplateLifecycleRecord, MapObjectRecord, MovementGeneratorType,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedCreatureTemplateLikeCpp {
    pub entry: u32,
    pub original_entry: u32,
    pub difficulty_id: u8,
    pub name: String,
    pub unit_class: u8,
    pub faction: u32,
    pub display_id: u32,
    pub model_dimensions: Option<CreatureModelDimensions>,
    pub scale: f32,
    pub speed_walk: f32,
    pub speed_run: f32,
    pub spells: [u32; 8],
    pub classification: u32,
    pub flags_extra: u32,
    pub type_flags: u32,
    pub movement_type: MovementGeneratorType,
    pub min_level: u8,
    pub max_level: u8,
    pub equipment_id: u8,
    pub original_equipment_id: i8,
    pub vehicle_id: Option<u32>,
    pub corpse_delay: u32,
    pub ignore_corpse_decay_ratio: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedCreatureSpawnLikeCpp {
    /// Caller-resolved map-generated object GUID.
    ///
    /// C++ `Creature::LoadFromDB` calls `map->GenerateLowGuid<HighGuid::Creature>()`, while
    /// `Creature::CreateFromProto` chooses `HighGuid::Vehicle` when a vehicle kit exists. This
    /// identity must therefore come from the future Map-owned caller, not from `spawn_id`.
    pub map_object_guid: ObjectGuid,
    pub spawn_id: u64,
    pub entry: u32,
    pub map_id: u32,
    pub instance_id: u32,
    pub position: Position,
    pub home_position: Position,
    pub phase_id: Option<u32>,
    pub phase_group: Option<u32>,
    pub terrain_swap_map: Option<u32>,
    pub spawn_group_id: Option<u32>,
    pub spawn_group_name: Option<String>,
    pub pool_id: Option<u32>,
    pub equipment_id: Option<u8>,
    pub original_equipment_id: Option<i8>,
    pub wander_distance: f32,
    pub respawn_delay: u32,
    pub respawn_time: i64,
    pub movement_type: MovementGeneratorType,
    pub string_id: Option<String>,
    pub is_active: bool,
    pub inactive_by_spawn_group: bool,
    pub duplicate_spawn_found: bool,
    pub add_to_map: bool,
    pub respawn_compatibility_mode: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedCreatureRuntimeSelectionLikeCpp {
    pub selected_level: u8,
    pub stats: CreatureLifecycleStats,
    pub selected_display_id: u32,
    /// Explicit fallback seam for model data not yet available in a complete live store.
    /// `None` is preserved honestly; no dummy dimensions are invented.
    pub selected_model_dimensions: Option<CreatureModelDimensions>,
    pub selected_equipment_id: u8,
    pub selected_original_equipment_id: i8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreatureLoadedGridResolvedLikeCpp {
    pub lifecycle_record: CreatureLoadFromDbLifecycleRecord,
    pub creature: Creature,
    pub map_object_record: Option<MapObjectRecord>,
    pub map_insertion_requested: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureLoadedGridResolveErrorLikeCpp {
    MissingSpawnData {
        spawn_id: u64,
    },
    MissingTemplate {
        entry: u32,
    },
    MissingRuntimeSelection {
        entry: u32,
    },
    InvalidMapObjectGuid {
        guid: ObjectGuid,
        expected_high: HighGuid,
        expected_map_id: u32,
    },
    MapObjectRecord(String),
}

#[derive(Debug, Clone, Default)]
pub struct CreatureLoadedGridLifecycleResolverLikeCpp {
    templates: BTreeMap<u32, ResolvedCreatureTemplateLikeCpp>,
    spawns: BTreeMap<u64, ResolvedCreatureSpawnLikeCpp>,
    runtime_selections: BTreeMap<u32, ResolvedCreatureRuntimeSelectionLikeCpp>,
}

impl CreatureLoadedGridLifecycleResolverLikeCpp {
    pub fn new(
        templates: impl IntoIterator<Item = ResolvedCreatureTemplateLikeCpp>,
        spawns: impl IntoIterator<Item = ResolvedCreatureSpawnLikeCpp>,
        runtime_selections: impl IntoIterator<Item = (u32, ResolvedCreatureRuntimeSelectionLikeCpp)>,
    ) -> Self {
        Self {
            templates: templates
                .into_iter()
                .map(|template| (template.entry, template))
                .collect(),
            spawns: spawns
                .into_iter()
                .map(|spawn| (spawn.spawn_id, spawn))
                .collect(),
            runtime_selections: runtime_selections.into_iter().collect(),
        }
    }

    pub fn resolve_loaded_grid_creature_like_cpp(
        &self,
        spawn_id: u64,
    ) -> Result<CreatureLoadedGridResolvedLikeCpp, CreatureLoadedGridResolveErrorLikeCpp> {
        let spawn = self
            .spawns
            .get(&spawn_id)
            .ok_or(CreatureLoadedGridResolveErrorLikeCpp::MissingSpawnData { spawn_id })?;
        let template = self
            .templates
            .get(&spawn.entry)
            .ok_or(CreatureLoadedGridResolveErrorLikeCpp::MissingTemplate { entry: spawn.entry })?;
        let selection = self.runtime_selections.get(&spawn.entry).ok_or(
            CreatureLoadedGridResolveErrorLikeCpp::MissingRuntimeSelection { entry: spawn.entry },
        )?;
        validate_map_object_guid_like_cpp(spawn, template)?;

        let lifecycle_record = CreatureLoadFromDbLifecycleRecord {
            create: CreatureCreateLifecycleRecord {
                guid: spawn.map_object_guid,
                entry: template.entry,
                map_id: spawn.map_id,
                instance_id: spawn.instance_id,
                position: spawn.position,
                dynamic: false,
                vehicle_id: template.vehicle_id,
                template: template_lifecycle_record(template),
                spawn: Some(spawn_lifecycle_record(spawn)),
                selected_level: selection.selected_level,
                stats: selection.stats,
                selected_display_id: selection.selected_display_id,
                selected_model_dimensions: selection.selected_model_dimensions,
                selected_equipment_id: selection.selected_equipment_id,
                selected_original_equipment_id: selection.selected_original_equipment_id,
                corpse_delay: template.corpse_delay,
                ignore_corpse_decay_ratio: template.ignore_corpse_decay_ratio,
            },
            spawn: spawn_lifecycle_record(spawn),
        };

        let creature = Creature::load_from_db_lifecycle(lifecycle_record.clone());
        let map_insertion_requested = spawn.add_to_map;
        let map_object_record = if map_insertion_requested {
            Some(
                MapObjectRecord::new_creature(creature.clone()).map_err(|error| {
                    CreatureLoadedGridResolveErrorLikeCpp::MapObjectRecord(format!("{error:?}"))
                })?,
            )
        } else {
            None
        };

        Ok(CreatureLoadedGridResolvedLikeCpp {
            lifecycle_record,
            creature,
            map_object_record,
            map_insertion_requested,
        })
    }
}

fn validate_map_object_guid_like_cpp(
    spawn: &ResolvedCreatureSpawnLikeCpp,
    template: &ResolvedCreatureTemplateLikeCpp,
) -> Result<(), CreatureLoadedGridResolveErrorLikeCpp> {
    let expected_high = if template.vehicle_id.is_some() {
        HighGuid::Vehicle
    } else {
        HighGuid::Creature
    };

    if spawn.map_object_guid.high_type() != expected_high
        || u32::from(spawn.map_object_guid.map_id()) != spawn.map_id
    {
        return Err(
            CreatureLoadedGridResolveErrorLikeCpp::InvalidMapObjectGuid {
                guid: spawn.map_object_guid,
                expected_high,
                expected_map_id: spawn.map_id,
            },
        );
    }

    Ok(())
}

fn template_lifecycle_record(
    template: &ResolvedCreatureTemplateLikeCpp,
) -> CreatureTemplateLifecycleRecord {
    CreatureTemplateLifecycleRecord {
        entry: template.entry,
        original_entry: template.original_entry,
        difficulty_id: template.difficulty_id,
        name: template.name.clone(),
        unit_class: template.unit_class,
        faction: template.faction,
        display_id: template.display_id,
        model_dimensions: template.model_dimensions,
        scale: template.scale,
        speed_walk: template.speed_walk,
        speed_run: template.speed_run,
        spells: template.spells,
        classification: template.classification,
        flags_extra: template.flags_extra,
        type_flags: template.type_flags,
        movement_type: template.movement_type,
        min_level: template.min_level,
        max_level: template.max_level,
        equipment_id: template.equipment_id,
        original_equipment_id: template.original_equipment_id,
    }
}

fn spawn_lifecycle_record(spawn: &ResolvedCreatureSpawnLikeCpp) -> CreatureSpawnLifecycleRecord {
    CreatureSpawnLifecycleRecord {
        spawn_id: spawn.spawn_id,
        map_id: spawn.map_id,
        instance_id: spawn.instance_id,
        position: spawn.position,
        home_position: spawn.home_position,
        phase_id: spawn.phase_id,
        phase_group: spawn.phase_group,
        terrain_swap_map: spawn.terrain_swap_map,
        spawn_group_id: spawn.spawn_group_id,
        spawn_group_name: spawn.spawn_group_name.clone(),
        pool_id: spawn.pool_id,
        equipment_id: spawn.equipment_id,
        original_equipment_id: spawn.original_equipment_id,
        wander_distance: spawn.wander_distance,
        respawn_delay: spawn.respawn_delay,
        respawn_time: spawn.respawn_time,
        movement_type: spawn.movement_type,
        string_id: spawn.string_id.clone(),
        is_active: spawn.is_active,
        inactive_by_spawn_group: spawn.inactive_by_spawn_group,
        duplicate_spawn_found: spawn.duplicate_spawn_found,
        add_to_map: spawn.add_to_map,
        respawn_compatibility_mode: spawn.respawn_compatibility_mode,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wow_constants::PowerType;

    fn position(x: f32, y: f32, z: f32, orientation: f32) -> Position {
        Position {
            x,
            y,
            z,
            orientation,
        }
    }

    fn template(entry: u32) -> ResolvedCreatureTemplateLikeCpp {
        ResolvedCreatureTemplateLikeCpp {
            entry,
            original_entry: entry - 1,
            difficulty_id: 2,
            name: "Loaded Grid Test Creature".to_string(),
            unit_class: 1,
            faction: 35,
            display_id: 9001,
            model_dimensions: Some(CreatureModelDimensions {
                bounding_radius: 0.7,
                combat_reach: 1.5,
            }),
            scale: 1.25,
            speed_walk: 1.0,
            speed_run: 1.14286,
            spells: [11, 22, 33, 44, 55, 66, 77, 88],
            classification: 4,
            flags_extra: 0x10,
            type_flags: 0x20,
            movement_type: MovementGeneratorType::Idle,
            min_level: 18,
            max_level: 20,
            equipment_id: 3,
            original_equipment_id: -2,
            vehicle_id: Some(77),
            corpse_delay: 61,
            ignore_corpse_decay_ratio: true,
        }
    }

    fn spawn(spawn_id: u64, entry: u32, add_to_map: bool) -> ResolvedCreatureSpawnLikeCpp {
        ResolvedCreatureSpawnLikeCpp {
            map_object_guid: ObjectGuid::create_world_object(
                HighGuid::Vehicle,
                0,
                1,
                571,
                1,
                entry,
                spawn_id as i64,
            ),
            spawn_id,
            entry,
            map_id: 571,
            instance_id: 9,
            position: position(100.0, 200.0, 30.0, 1.57),
            home_position: position(101.0, 201.0, 31.0, 2.57),
            phase_id: Some(5),
            phase_group: Some(6),
            terrain_swap_map: Some(7),
            spawn_group_id: Some(8),
            spawn_group_name: Some("wintergrasp-test".to_string()),
            pool_id: Some(9),
            equipment_id: Some(4),
            original_equipment_id: Some(-4),
            wander_distance: 12.5,
            respawn_delay: 300,
            respawn_time: 123_456,
            movement_type: MovementGeneratorType::Idle,
            string_id: Some("loaded_grid_string".to_string()),
            is_active: false,
            inactive_by_spawn_group: true,
            duplicate_spawn_found: true,
            add_to_map,
            respawn_compatibility_mode: true,
        }
    }

    fn selection(entry: u32) -> (u32, ResolvedCreatureRuntimeSelectionLikeCpp) {
        (
            entry,
            ResolvedCreatureRuntimeSelectionLikeCpp {
                selected_level: 19,
                stats: CreatureLifecycleStats {
                    max_health: 1_234,
                    health: 777,
                    power_type: PowerType::Mana,
                    max_mana: 456,
                    mana: 123,
                    min_damage: 12.0,
                    max_damage: 34.0,
                },
                selected_display_id: 9002,
                selected_model_dimensions: None,
                selected_equipment_id: 6,
                selected_original_equipment_id: -6,
            },
        )
    }

    #[test]
    fn loaded_grid_creature_lifecycle_resolver_maps_spawn_template_and_selection_like_cpp() {
        let entry = 12_345;
        let resolver = CreatureLoadedGridLifecycleResolverLikeCpp::new(
            [template(entry)],
            [spawn(55, entry, true)],
            [selection(entry)],
        );

        let resolved = resolver
            .resolve_loaded_grid_creature_like_cpp(55)
            .expect("resolver should build lifecycle record");
        let record = &resolved.lifecycle_record;
        let creature = &resolved.creature;
        let metadata = creature.lifecycle_metadata();

        assert_eq!(record.create.entry, entry);
        assert_eq!(record.create.template.original_entry, entry - 1);
        assert_eq!(record.create.map_id, 571);
        assert_eq!(record.create.instance_id, 9);
        assert_eq!(record.spawn.spawn_id, 55);
        assert_eq!(record.spawn.position, position(100.0, 200.0, 30.0, 1.57));
        assert_eq!(
            record.spawn.home_position,
            position(101.0, 201.0, 31.0, 2.57)
        );
        assert_eq!(record.spawn.respawn_delay, 300);
        assert_eq!(record.spawn.respawn_time, 123_456);
        assert_eq!(record.spawn.movement_type, MovementGeneratorType::Idle);
        assert_eq!(
            record.spawn.string_id.as_deref(),
            Some("loaded_grid_string")
        );
        assert_eq!(record.spawn.spawn_group_id, Some(8));
        assert_eq!(record.spawn.pool_id, Some(9));
        assert!(record.spawn.inactive_by_spawn_group);
        assert!(record.spawn.duplicate_spawn_found);
        assert_eq!(record.spawn.equipment_id, Some(4));
        assert_eq!(record.spawn.original_equipment_id, Some(-4));
        assert_eq!(record.create.selected_level, 19);
        assert_eq!(record.create.stats.health, 777);
        assert_eq!(record.create.selected_display_id, 9002);
        assert_eq!(record.create.selected_model_dimensions, None);

        assert_eq!(metadata.spawn_id, 55);
        assert_eq!(metadata.spawn_map_id, 571);
        assert_eq!(metadata.spawn_instance_id, 9);
        assert_eq!(metadata.spawn_position, position(100.0, 200.0, 30.0, 1.57));
        assert_eq!(metadata.home_position, position(101.0, 201.0, 31.0, 2.57));
        assert_eq!(metadata.phase_id, Some(5));
        assert_eq!(metadata.terrain_swap_map, Some(7));
        assert_eq!(
            metadata.spawn_group_name.as_deref(),
            Some("wintergrasp-test")
        );
        assert_eq!(metadata.pool_id, Some(9));
        assert!(!metadata.is_spawn_active);
        assert!(metadata.inactive_by_spawn_group);
        assert!(metadata.duplicate_spawn_found);
        assert_eq!(metadata.equipment_id, 4);
        assert_eq!(metadata.original_equipment_id, -4);
        assert_eq!(creature.ai_current_health(), 777);
        assert_eq!(creature.ai_max_health(), 1_234);
        assert_eq!(creature.ai_level(), 19);
        assert!(resolved.map_insertion_requested);
        assert!(resolved.map_object_record.is_some());
        assert!(
            resolved
                .map_object_record
                .as_ref()
                .and_then(MapObjectRecord::creature)
                .is_some()
        );
    }

    #[test]
    fn loaded_grid_creature_lifecycle_resolver_respects_add_to_map_request_flag() {
        let entry = 12_346;
        let resolver = CreatureLoadedGridLifecycleResolverLikeCpp::new(
            [template(entry)],
            [spawn(56, entry, false)],
            [selection(entry)],
        );

        let resolved = resolver
            .resolve_loaded_grid_creature_like_cpp(56)
            .expect("resolver should build creature without insertion request");

        assert!(!resolved.map_insertion_requested);
        assert!(resolved.map_object_record.is_none());
        assert!(
            !resolved
                .creature
                .lifecycle_metadata()
                .map_insertion_requested
        );
        assert!(!resolved.creature.lifecycle_metadata().add_to_map_requested);
    }

    #[test]
    fn loaded_grid_creature_lifecycle_resolver_errors_without_dummy_for_missing_inputs() {
        let entry = 12_347;
        let missing_spawn = CreatureLoadedGridLifecycleResolverLikeCpp::new(
            [template(entry)],
            [],
            [selection(entry)],
        );
        assert_eq!(
            missing_spawn.resolve_loaded_grid_creature_like_cpp(57),
            Err(CreatureLoadedGridResolveErrorLikeCpp::MissingSpawnData { spawn_id: 57 })
        );

        let missing_template = CreatureLoadedGridLifecycleResolverLikeCpp::new(
            [],
            [spawn(58, entry, true)],
            [selection(entry)],
        );
        assert_eq!(
            missing_template.resolve_loaded_grid_creature_like_cpp(58),
            Err(CreatureLoadedGridResolveErrorLikeCpp::MissingTemplate { entry })
        );

        let missing_selection = CreatureLoadedGridLifecycleResolverLikeCpp::new(
            [template(entry)],
            [spawn(59, entry, true)],
            [],
        );
        assert_eq!(
            missing_selection.resolve_loaded_grid_creature_like_cpp(59),
            Err(CreatureLoadedGridResolveErrorLikeCpp::MissingRuntimeSelection { entry })
        );
    }

    #[test]
    fn loaded_grid_creature_lifecycle_resolver_is_pure_ordered_bridge_like_cpp() {
        let plan = wow_entities::CreatureLifecyclePlan::trinity_create_load_from_db();
        assert!(plan.occurs_before(
            wow_entities::CreatureLifecycleStep::LookupTemplateAndDifficulty,
            wow_entities::CreatureLifecycleStep::InitEntryAndCreateFromProto,
        ));
        assert!(plan.occurs_before(
            wow_entities::CreatureLifecycleStep::LoadFromDbSpawnHomeRespawnInactiveChecks,
            wow_entities::CreatureLifecycleStep::AddToMap,
        ));

        let entry = 12_348;
        let resolver = CreatureLoadedGridLifecycleResolverLikeCpp::new(
            [template(entry)],
            [spawn(60, entry, true)],
            [selection(entry)],
        );
        let first = resolver.resolve_loaded_grid_creature_like_cpp(60).unwrap();
        let second = resolver.resolve_loaded_grid_creature_like_cpp(60).unwrap();

        assert_eq!(first.lifecycle_record, second.lifecycle_record);
        assert_eq!(
            first.creature.lifecycle_metadata(),
            second.creature.lifecycle_metadata()
        );
        assert!(first.map_insertion_requested);
        assert!(second.map_insertion_requested);
    }
}
