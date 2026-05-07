//! Grid unload helper pass.
//!
//! C++ references:
//! - `game/Grids/ObjectGridLoader.h`
//! - `game/Grids/ObjectGridLoader.cpp`
//! - `game/Maps/Map.cpp::UnloadGrid`

use wow_core::ObjectGuid;

use crate::cell::GridObjectGuids;
use crate::grid::NGrid;
use crate::map::GridLifecycle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridObjectKind {
    Creature,
    GameObject,
    DynamicObject,
    Corpse,
    AreaTrigger,
    SceneObject,
    Conversation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridUnloadAction {
    RemoveAllDynObjects(ObjectGuid),
    RemoveAllAreaTriggers(ObjectGuid),
    CombatStop(ObjectGuid),
    CreatureRespawnRelocation(ObjectGuid),
    GameObjectRespawnRelocation(ObjectGuid),
    SetDestroyedObject(GridObjectKind, ObjectGuid),
    CleanupsBeforeDelete(GridObjectKind, ObjectGuid),
    DeleteObject(GridObjectKind, ObjectGuid),
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GuidGridUnloadLifecycle {
    actions: Vec<GridUnloadAction>,
}

impl GuidGridUnloadLifecycle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn actions(&self) -> &[GridUnloadAction] {
        &self.actions
    }

    pub fn into_actions(self) -> Vec<GridUnloadAction> {
        self.actions
    }
}

impl GridLifecycle for GuidGridUnloadLifecycle {
    fn load_grid_objects(&mut self, _grid: &mut NGrid, _cell: &crate::cell::Cell) {}

    fn stop_grid_objects(&mut self, grid: &NGrid) {
        object_grid_stoper(grid, &mut self.actions);
    }

    fn evacuate_grid(&mut self, grid: &mut NGrid) {
        object_grid_evacuator(grid, &mut self.actions);
    }

    fn clean_grid(&mut self, grid: &mut NGrid) {
        object_grid_cleaner(grid, &mut self.actions);
    }

    fn unload_grid_objects(&mut self, grid: &mut NGrid) {
        object_grid_unloader(grid, &mut self.actions);
    }
}

pub fn object_grid_stoper(grid: &NGrid, actions: &mut Vec<GridUnloadAction>) {
    grid.visit_all_grids(|cell| {
        for guid in &cell.grid_objects.creatures {
            actions.push(GridUnloadAction::RemoveAllDynObjects(*guid));
            actions.push(GridUnloadAction::RemoveAllAreaTriggers(*guid));
            actions.push(GridUnloadAction::CombatStop(*guid));
        }
    });
}

pub fn object_grid_evacuator(grid: &NGrid, actions: &mut Vec<GridUnloadAction>) {
    grid.visit_all_grids(|cell| {
        for guid in &cell.grid_objects.creatures {
            actions.push(GridUnloadAction::CreatureRespawnRelocation(*guid));
        }

        for guid in &cell.grid_objects.gameobjects {
            actions.push(GridUnloadAction::GameObjectRespawnRelocation(*guid));
        }
    });
}

pub fn object_grid_cleaner(grid: &NGrid, actions: &mut Vec<GridUnloadAction>) {
    grid.visit_all_grids(|cell| {
        for_grid_object(
            cell.grid_objects.creatures.iter().copied(),
            GridObjectKind::Creature,
            actions,
        );
        for_grid_object(
            cell.grid_objects.gameobjects.iter().copied(),
            GridObjectKind::GameObject,
            actions,
        );
        for_grid_object(
            cell.grid_objects.dynamic_objects.iter().copied(),
            GridObjectKind::DynamicObject,
            actions,
        );
        for_grid_object(
            cell.grid_objects.corpses.iter().copied(),
            GridObjectKind::Corpse,
            actions,
        );
        for_grid_object(
            cell.grid_objects.area_triggers.iter().copied(),
            GridObjectKind::AreaTrigger,
            actions,
        );
        for_grid_object(
            cell.grid_objects.scene_objects.iter().copied(),
            GridObjectKind::SceneObject,
            actions,
        );
        for_grid_object(
            cell.grid_objects.conversations.iter().copied(),
            GridObjectKind::Conversation,
            actions,
        );
    });
}

fn for_grid_object<I>(guids: I, kind: GridObjectKind, actions: &mut Vec<GridUnloadAction>)
where
    I: IntoIterator<Item = ObjectGuid>,
{
    for guid in guids {
        actions.push(GridUnloadAction::SetDestroyedObject(kind, guid));
        actions.push(GridUnloadAction::CleanupsBeforeDelete(kind, guid));
    }
}

pub fn object_grid_unloader(grid: &mut NGrid, actions: &mut Vec<GridUnloadAction>) {
    grid.visit_all_grids_mut(|cell| {
        unload_guid_set(
            &mut cell.grid_objects.creatures,
            GridObjectKind::Creature,
            actions,
        );
        unload_guid_set(
            &mut cell.grid_objects.gameobjects,
            GridObjectKind::GameObject,
            actions,
        );
        unload_guid_set(
            &mut cell.grid_objects.dynamic_objects,
            GridObjectKind::DynamicObject,
            actions,
        );
        unload_guid_set(
            &mut cell.grid_objects.area_triggers,
            GridObjectKind::AreaTrigger,
            actions,
        );
        unload_guid_set(
            &mut cell.grid_objects.scene_objects,
            GridObjectKind::SceneObject,
            actions,
        );
        unload_guid_set(
            &mut cell.grid_objects.conversations,
            GridObjectKind::Conversation,
            actions,
        );
        cell.grid_objects.corpses.clear();
    });
}

fn unload_guid_set(
    guids: &mut std::collections::HashSet<ObjectGuid>,
    kind: GridObjectKind,
    actions: &mut Vec<GridUnloadAction>,
) {
    for guid in guids.drain() {
        actions.push(GridUnloadAction::CleanupsBeforeDelete(kind, guid));
        actions.push(GridUnloadAction::DeleteObject(kind, guid));
    }
}

pub fn grid_object_count(grid_objects: &GridObjectGuids) -> usize {
    grid_objects.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::NGrid;
    use wow_core::guid::HighGuid;

    fn guid(kind: HighGuid, counter: i64) -> ObjectGuid {
        ObjectGuid::create_world_object(kind, 0, 1, 571, 1, counter as u32, counter)
    }

    #[test]
    fn stoper_emits_creature_only_combat_cleanup_actions() {
        let creature = guid(HighGuid::Creature, 1);
        let gameobject = guid(HighGuid::GameObject, 2);
        let mut grid = NGrid::from_coords(32, 32, 1000, true);
        let cell = grid.get_grid_type_mut(0, 0).unwrap();
        cell.grid_objects.creatures.insert(creature);
        cell.grid_objects.gameobjects.insert(gameobject);

        let mut actions = Vec::new();
        object_grid_stoper(&grid, &mut actions);

        assert_eq!(
            actions,
            vec![
                GridUnloadAction::RemoveAllDynObjects(creature),
                GridUnloadAction::RemoveAllAreaTriggers(creature),
                GridUnloadAction::CombatStop(creature),
            ]
        );
    }

    #[test]
    fn evacuator_emits_creature_and_gameobject_respawn_relocation_actions() {
        let creature = guid(HighGuid::Creature, 1);
        let gameobject = guid(HighGuid::GameObject, 2);
        let mut grid = NGrid::from_coords(32, 32, 1000, true);
        let cell = grid.get_grid_type_mut(0, 0).unwrap();
        cell.grid_objects.creatures.insert(creature);
        cell.grid_objects.gameobjects.insert(gameobject);

        let mut actions = Vec::new();
        object_grid_evacuator(&grid, &mut actions);

        assert_eq!(
            actions,
            vec![
                GridUnloadAction::CreatureRespawnRelocation(creature),
                GridUnloadAction::GameObjectRespawnRelocation(gameobject),
            ]
        );
    }

    #[test]
    fn cleaner_marks_and_cleans_every_grid_object_type_in_place() {
        let creature = guid(HighGuid::Creature, 1);
        let corpse = guid(HighGuid::Corpse, 2);
        let mut grid = NGrid::from_coords(32, 32, 1000, true);
        let cell = grid.get_grid_type_mut(0, 0).unwrap();
        cell.grid_objects.creatures.insert(creature);
        cell.grid_objects.corpses.insert(corpse);

        let mut actions = Vec::new();
        object_grid_cleaner(&grid, &mut actions);

        assert_eq!(
            actions,
            vec![
                GridUnloadAction::SetDestroyedObject(GridObjectKind::Creature, creature),
                GridUnloadAction::CleanupsBeforeDelete(GridObjectKind::Creature, creature),
                GridUnloadAction::SetDestroyedObject(GridObjectKind::Corpse, corpse),
                GridUnloadAction::CleanupsBeforeDelete(GridObjectKind::Corpse, corpse),
            ]
        );
    }

    #[test]
    fn unloader_deletes_non_corpse_grid_objects_and_clears_grid_sets() {
        let creature = guid(HighGuid::Creature, 1);
        let corpse = guid(HighGuid::Corpse, 2);
        let mut grid = NGrid::from_coords(32, 32, 1000, true);
        let cell = grid.get_grid_type_mut(0, 0).unwrap();
        cell.grid_objects.creatures.insert(creature);
        cell.grid_objects.corpses.insert(corpse);

        let mut actions = Vec::new();
        object_grid_unloader(&mut grid, &mut actions);

        assert_eq!(
            actions,
            vec![
                GridUnloadAction::CleanupsBeforeDelete(GridObjectKind::Creature, creature),
                GridUnloadAction::DeleteObject(GridObjectKind::Creature, creature),
            ]
        );
        assert!(grid.get_grid_type(0, 0).unwrap().grid_objects.is_empty());
    }

    #[test]
    fn lifecycle_runs_cpp_unload_order_for_normal_unload() {
        let creature = guid(HighGuid::Creature, 1);
        let mut lifecycle = GuidGridUnloadLifecycle::new();
        let mut grid = NGrid::from_coords(32, 32, 1000, true);
        grid.get_grid_type_mut(0, 0)
            .unwrap()
            .grid_objects
            .creatures
            .insert(creature);

        lifecycle.evacuate_grid(&mut grid);
        lifecycle.clean_grid(&mut grid);
        lifecycle.unload_grid_objects(&mut grid);

        assert_eq!(
            lifecycle.actions(),
            &[
                GridUnloadAction::CreatureRespawnRelocation(creature),
                GridUnloadAction::SetDestroyedObject(GridObjectKind::Creature, creature),
                GridUnloadAction::CleanupsBeforeDelete(GridObjectKind::Creature, creature),
                GridUnloadAction::CleanupsBeforeDelete(GridObjectKind::Creature, creature),
                GridUnloadAction::DeleteObject(GridObjectKind::Creature, creature),
            ]
        );
        assert!(grid.get_grid_type(0, 0).unwrap().grid_objects.is_empty());
    }
}
