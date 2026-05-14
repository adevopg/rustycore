// Copyright (c) 2026 alseif0x
// RustyCore - WoW WotLK 3.4.3 server in Rust
// Based on TrinityCore protocol research (https://github.com/TrinityCore/TrinityCore)
// Licensed under GPL v3 - https://www.gnu.org/licenses/gpl-3.0.html

//! Minimal world-table ID stores for C++ `ObjectMgr` existence checks.

use std::collections::HashSet;

use anyhow::Result;
use wow_database::{WorldDatabase, WorldStatements};

#[derive(Debug, Clone)]
pub struct WorldIdStore {
    name: &'static str,
    ids: HashSet<u32>,
}

impl WorldIdStore {
    pub fn from_ids(name: &'static str, ids: impl IntoIterator<Item = u32>) -> Self {
        Self {
            name,
            ids: ids.into_iter().collect(),
        }
    }

    pub async fn load_like_cpp(
        db: &WorldDatabase,
        name: &'static str,
        statement: WorldStatements,
    ) -> Result<Self> {
        let stmt = db.prepare(statement);
        let mut result = db.query(&stmt).await?;
        if result.is_empty() {
            return Ok(Self::from_ids(name, []));
        }

        let mut ids = HashSet::new();
        loop {
            ids.insert(result.read(0));
            if !result.next_row() {
                break;
            }
        }

        Ok(Self { name, ids })
    }

    pub fn contains(&self, id: u32) -> bool {
        self.ids.contains(&id)
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_id_store_indexes_ids_like_object_mgr_store() {
        let store = WorldIdStore::from_ids("creature_template", [1, 42]);

        assert_eq!(store.name(), "creature_template");
        assert!(store.contains(42));
        assert!(!store.contains(43));
        assert_eq!(store.len(), 2);
    }
}
