// Copyright (c) 2026 alseif0x
// RustyCore — WoW WotLK 3.4.3 server in Rust
// Based on TrinityCore protocol research (https://github.com/TrinityCore/TrinityCore)
// Licensed under GPL v3 — https://www.gnu.org/licenses/gpl-3.0.html

//! RandPropPoints.db2 reader.

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use tracing::info;

use crate::wdc4::Wdc4Reader;

pub const RAND_PROP_POINTS_COLUMNS: usize = 5;

/// C++ `RandPropPointsEntry`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RandPropPointsEntry {
    pub id: u32,
    pub damage_replace_stat: i32,
    pub epic: [u32; RAND_PROP_POINTS_COLUMNS],
    pub superior: [u32; RAND_PROP_POINTS_COLUMNS],
    pub good: [u32; RAND_PROP_POINTS_COLUMNS],
}

/// In-memory store for `RandPropPoints.db2`.
pub struct RandPropPointsStore {
    entries: HashMap<u32, RandPropPointsEntry>,
}

impl RandPropPointsStore {
    pub fn from_entries(entries: impl IntoIterator<Item = RandPropPointsEntry>) -> Self {
        Self {
            entries: entries.into_iter().map(|entry| (entry.id, entry)).collect(),
        }
    }

    /// Load RandPropPoints.db2 from `{data_dir}/dbc/{locale}/RandPropPoints.db2`.
    ///
    /// C++ refs:
    /// - `DB2Structure.h::RandPropPointsEntry`
    /// - `DB2LoadInfo.h::RandPropPointsLoadInfo`
    pub fn load(data_dir: &str, locale: &str) -> Result<Self> {
        let path = Path::new(data_dir)
            .join("dbc")
            .join(locale)
            .join("RandPropPoints.db2");

        let reader = Wdc4Reader::open(&path)
            .with_context(|| format!("failed to open {}", path.display()))?;

        let mut entries = HashMap::with_capacity(reader.total_count());
        for (id, idx) in reader.iter_records() {
            let record = RandPropPointsEntry {
                id,
                damage_replace_stat: reader.get_field_i32(idx, 0),
                epic: [
                    reader.get_array_element(idx, 1, 0, 32),
                    reader.get_array_element(idx, 1, 1, 32),
                    reader.get_array_element(idx, 1, 2, 32),
                    reader.get_array_element(idx, 1, 3, 32),
                    reader.get_array_element(idx, 1, 4, 32),
                ],
                superior: [
                    reader.get_array_element(idx, 2, 0, 32),
                    reader.get_array_element(idx, 2, 1, 32),
                    reader.get_array_element(idx, 2, 2, 32),
                    reader.get_array_element(idx, 2, 3, 32),
                    reader.get_array_element(idx, 2, 4, 32),
                ],
                good: [
                    reader.get_array_element(idx, 3, 0, 32),
                    reader.get_array_element(idx, 3, 1, 32),
                    reader.get_array_element(idx, 3, 2, 32),
                    reader.get_array_element(idx, 3, 3, 32),
                    reader.get_array_element(idx, 3, 4, 32),
                ],
            };
            entries.insert(id, record);
        }

        info!(
            "Loaded {} random property point rows from {}",
            entries.len(),
            path.display()
        );
        Ok(Self { entries })
    }

    pub fn get(&self, item_level: u32) -> Option<&RandPropPointsEntry> {
        self.entries.get(&item_level)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rand_prop_points_store_indexes_by_item_level_like_cpp() {
        let store = RandPropPointsStore::from_entries([RandPropPointsEntry {
            id: 11,
            damage_replace_stat: 0,
            epic: [900, 901, 902, 903, 904],
            superior: [500, 501, 502, 503, 504],
            good: [100, 101, 102, 103, 104],
        }]);

        let entry = store.get(11).unwrap();
        assert_eq!(entry.good[0], 100);
        assert_eq!(entry.superior[3], 503);
        assert_eq!(entry.epic[4], 904);
        assert!(store.get(12).is_none());
    }
}
