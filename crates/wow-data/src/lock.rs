// Copyright (c) 2026 alseif0x
// RustyCore — WoW WotLK 3.4.3 server in Rust
// Based on TrinityCore protocol research (https://github.com/TrinityCore/TrinityCore)
// Licensed under GPL v3 — https://www.gnu.org/licenses/gpl-3.0.html

//! Lock.db2 reader.

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use tracing::info;

use crate::wdc4::Wdc4Reader;

pub const MAX_LOCK_CASE: usize = 8;

/// C++ `LockEntry`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LockEntry {
    pub id: u32,
    pub index: [i32; MAX_LOCK_CASE],
    pub skill: [u16; MAX_LOCK_CASE],
    pub lock_type: [u8; MAX_LOCK_CASE],
    pub action: [u8; MAX_LOCK_CASE],
}

/// In-memory store for `Lock.db2`.
pub struct LockStore {
    entries: HashMap<u32, LockEntry>,
}

impl LockStore {
    pub fn from_entries(entries: impl IntoIterator<Item = LockEntry>) -> Self {
        Self {
            entries: entries.into_iter().map(|entry| (entry.id, entry)).collect(),
        }
    }

    /// Load Lock.db2 from `{data_dir}/dbc/{locale}/Lock.db2`.
    ///
    /// C++ refs:
    /// - `DB2Structure.h::LockEntry`
    /// - `DB2LoadInfo.h::LockLoadInfo`
    pub fn load(data_dir: &str, locale: &str) -> Result<Self> {
        let path = Path::new(data_dir)
            .join("dbc")
            .join(locale)
            .join("Lock.db2");

        let reader = Wdc4Reader::open(&path)
            .with_context(|| format!("failed to open {}", path.display()))?;

        let mut entries = HashMap::with_capacity(reader.total_count());
        for (id, idx) in reader.iter_records() {
            let mut index = [0i32; MAX_LOCK_CASE];
            let mut skill = [0u16; MAX_LOCK_CASE];
            let mut lock_type = [0u8; MAX_LOCK_CASE];
            let mut action = [0u8; MAX_LOCK_CASE];

            for case in 0..MAX_LOCK_CASE {
                index[case] = reader.get_array_element(idx, 0, case, 32) as i32;
                skill[case] = reader.get_array_element(idx, 1, case, 16) as u16;
                lock_type[case] = reader.get_array_element(idx, 2, case, 8) as u8;
                action[case] = reader.get_array_element(idx, 3, case, 8) as u8;
            }

            entries.insert(
                id,
                LockEntry {
                    id,
                    index,
                    skill,
                    lock_type,
                    action,
                },
            );
        }

        info!("Loaded {} lock rows from {}", entries.len(), path.display());
        Ok(Self { entries })
    }

    pub fn get(&self, id: u32) -> Option<&LockEntry> {
        self.entries.get(&id)
    }

    pub fn contains(&self, id: u32) -> bool {
        self.entries.contains_key(&id)
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
    fn lock_store_indexes_by_id_like_cpp_slockstore() {
        let store = LockStore::from_entries([LockEntry {
            id: 99,
            index: [1, 2, 0, 0, 0, 0, 0, 0],
            skill: [100, 0, 0, 0, 0, 0, 0, 0],
            lock_type: [1, 0, 0, 0, 0, 0, 0, 0],
            action: [1, 0, 0, 0, 0, 0, 0, 0],
        }]);

        assert!(store.contains(99));
        assert_eq!(store.get(99).unwrap().index[1], 2);
        assert!(!store.contains(100));
    }
}
