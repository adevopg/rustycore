// Copyright (c) 2026 alseif0x
// RustyCore - WoW WotLK 3.4.3 server in Rust
// Based on TrinityCore protocol research (https://github.com/TrinityCore/TrinityCore)
// Licensed under GPL v3 - https://www.gnu.org/licenses/gpl-3.0.html

//! C++ `ObjectMgr` graveyard-zone links.

use std::collections::HashMap;

use anyhow::Result;
use wow_constants::ConditionSourceType;
use wow_database::{WorldDatabase, WorldStatements};

use crate::{ConditionEntriesByTypeStore, ConditionId, ConditionsReference};

#[derive(Debug, Clone, Default)]
pub struct GraveyardData {
    pub safe_loc_id: u32,
    pub conditions: ConditionsReference,
}

#[derive(Debug, Clone, Default)]
pub struct GraveyardStore {
    by_zone: HashMap<u32, Vec<GraveyardData>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraveyardZoneRow {
    pub safe_loc_id: u32,
    pub ghost_zone_id: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GraveyardLoadReport {
    pub loaded: usize,
    pub missing_safe_locs: Vec<GraveyardZoneRow>,
    pub missing_zones: Vec<GraveyardZoneRow>,
    pub duplicates: Vec<GraveyardZoneRow>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GraveyardConditionAttachmentReport {
    pub attached_condition_count: usize,
    pub missing_graveyards: Vec<ConditionId>,
}

impl GraveyardStore {
    pub fn graveyards_for_zone(&self, zone_id: u32) -> Option<&[GraveyardData]> {
        self.by_zone.get(&zone_id).map(Vec::as_slice)
    }

    pub fn find_graveyard_data_like_cpp(
        &self,
        safe_loc_id: u32,
        zone_id: u32,
    ) -> Option<&GraveyardData> {
        self.by_zone
            .get(&zone_id)?
            .iter()
            .find(|data| data.safe_loc_id == safe_loc_id)
    }

    fn find_graveyard_data_mut_like_cpp(
        &mut self,
        safe_loc_id: u32,
        zone_id: u32,
    ) -> Option<&mut GraveyardData> {
        self.by_zone
            .get_mut(&zone_id)?
            .iter_mut()
            .find(|data| data.safe_loc_id == safe_loc_id)
    }

    /// C++ `ObjectMgr::AddGraveyardLink`, without DB persistence side effects.
    pub fn add_graveyard_link_like_cpp(&mut self, safe_loc_id: u32, zone_id: u32) -> bool {
        if self
            .find_graveyard_data_like_cpp(safe_loc_id, zone_id)
            .is_some()
        {
            return false;
        }

        self.by_zone
            .entry(zone_id)
            .or_default()
            .push(GraveyardData {
                safe_loc_id,
                conditions: ConditionsReference::default(),
            });
        true
    }

    /// C++ `ObjectMgr::LoadGraveyardZones`.
    pub fn load_graveyard_zones_from_rows_like_cpp(
        &mut self,
        rows: impl IntoIterator<Item = GraveyardZoneRow>,
        mut world_safe_loc_exists: impl FnMut(u32) -> bool,
        mut area_exists: impl FnMut(u32) -> bool,
    ) -> GraveyardLoadReport {
        self.by_zone.clear();
        let mut report = GraveyardLoadReport::default();

        for row in rows {
            if !world_safe_loc_exists(row.safe_loc_id) {
                report.missing_safe_locs.push(row);
                continue;
            }

            if !area_exists(row.ghost_zone_id) {
                report.missing_zones.push(row);
                continue;
            }

            if self.add_graveyard_link_like_cpp(row.safe_loc_id, row.ghost_zone_id) {
                report.loaded += 1;
            } else {
                report.duplicates.push(row);
            }
        }

        report
    }

    pub async fn load_graveyard_zones_like_cpp(
        &mut self,
        db: &WorldDatabase,
        world_safe_loc_exists: impl FnMut(u32) -> bool,
        area_exists: impl FnMut(u32) -> bool,
    ) -> Result<GraveyardLoadReport> {
        let stmt = db.prepare(WorldStatements::SEL_GRAVEYARD_ZONE);
        let mut result = db.query(&stmt).await?;
        if result.is_empty() {
            self.by_zone.clear();
            return Ok(GraveyardLoadReport::default());
        }

        let mut rows = Vec::new();
        loop {
            rows.push(GraveyardZoneRow {
                safe_loc_id: result.read(0),
                ghost_zone_id: result.read(1),
            });
            if !result.next_row() {
                break;
            }
        }

        Ok(self.load_graveyard_zones_from_rows_like_cpp(rows, world_safe_loc_exists, area_exists))
    }

    /// C++ `ConditionMgr::addToGraveyardData`.
    pub fn attach_graveyard_conditions_like_cpp(
        &mut self,
        conditions: &ConditionEntriesByTypeStore,
    ) -> GraveyardConditionAttachmentReport {
        let mut report = GraveyardConditionAttachmentReport::default();
        let Some(graveyard_conditions) =
            conditions.entries_for_source_type_like_cpp(ConditionSourceType::Graveyard)
        else {
            return report;
        };

        for (id, condition_bucket) in graveyard_conditions {
            let mut found = false;
            if let Ok(safe_loc_id) = u32::try_from(id.source_entry)
                && let Some(graveyard) =
                    self.find_graveyard_data_mut_like_cpp(safe_loc_id, id.source_group)
            {
                graveyard.conditions = ConditionsReference::new(condition_bucket);
                report.attached_condition_count += condition_bucket.len();
                found = true;
            }

            if !found {
                report.missing_graveyards.push(*id);
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Condition;
    use wow_constants::ConditionType;

    fn row(safe_loc_id: u32, ghost_zone_id: u32) -> GraveyardZoneRow {
        GraveyardZoneRow {
            safe_loc_id,
            ghost_zone_id,
        }
    }

    #[test]
    fn graveyard_zones_load_validates_and_skips_like_cpp() {
        let mut store = GraveyardStore::default();

        let report = store.load_graveyard_zones_from_rows_like_cpp(
            [row(1, 10), row(2, 10), row(1, 10), row(3, 10), row(1, 30)],
            |safe_loc_id| safe_loc_id != 3,
            |zone_id| zone_id != 30,
        );

        assert_eq!(report.loaded, 2);
        assert_eq!(report.duplicates, vec![row(1, 10)]);
        assert_eq!(report.missing_safe_locs, vec![row(3, 10)]);
        assert_eq!(report.missing_zones, vec![row(1, 30)]);
        assert!(store.find_graveyard_data_like_cpp(1, 10).is_some());
        assert!(store.find_graveyard_data_like_cpp(3, 10).is_none());
    }

    #[test]
    fn graveyard_conditions_attach_by_zone_and_safe_loc_like_cpp() {
        let mut graveyards = GraveyardStore::default();
        graveyards.load_graveyard_zones_from_rows_like_cpp([row(1, 10)], |_| true, |_| true);
        let condition = Condition {
            source_type: ConditionSourceType::Graveyard,
            source_group: 10,
            source_entry: 1,
            condition_type: ConditionType::Team,
            condition_value1: 469,
            ..Condition::default()
        };
        let store = ConditionEntriesByTypeStore::from_conditions_like_cpp([condition]);

        let report = graveyards.attach_graveyard_conditions_like_cpp(&store);

        assert_eq!(report.attached_condition_count, 1);
        assert!(report.missing_graveyards.is_empty());
        let graveyard = graveyards.find_graveyard_data_like_cpp(1, 10).unwrap();
        assert_eq!(graveyard.conditions.upgrade().unwrap().len(), 1);
    }

    #[test]
    fn graveyard_conditions_report_missing_links_like_cpp() {
        let mut graveyards = GraveyardStore::default();
        graveyards.load_graveyard_zones_from_rows_like_cpp([row(1, 10)], |_| true, |_| true);
        let condition = Condition {
            source_type: ConditionSourceType::Graveyard,
            source_group: 10,
            source_entry: 2,
            condition_type: ConditionType::Team,
            condition_value1: 469,
            ..Condition::default()
        };
        let missing_id = condition.id_like_cpp();
        let store = ConditionEntriesByTypeStore::from_conditions_like_cpp([condition]);

        let report = graveyards.attach_graveyard_conditions_like_cpp(&store);

        assert_eq!(report.attached_condition_count, 0);
        assert_eq!(report.missing_graveyards, vec![missing_id]);
    }
}
