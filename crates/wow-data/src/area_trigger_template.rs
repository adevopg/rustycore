// Copyright (c) 2026 alseif0x
// RustyCore - WoW WotLK 3.4.3 server in Rust
// Based on TrinityCore protocol research (https://github.com/TrinityCore/TrinityCore)
// Licensed under GPL v3 - https://www.gnu.org/licenses/gpl-3.0.html

//! Minimal C++ `AreaTriggerDataStore` template-key index.

use std::collections::HashSet;

use anyhow::Result;
use wow_database::{WorldDatabase, WorldStatements};

#[derive(Debug, Clone, Default)]
pub struct AreaTriggerTemplateStore {
    keys: HashSet<(u32, bool)>,
}

impl AreaTriggerTemplateStore {
    pub fn from_keys(keys: impl IntoIterator<Item = (u32, bool)>) -> Self {
        Self {
            keys: keys.into_iter().collect(),
        }
    }

    pub async fn load_like_cpp(db: &WorldDatabase) -> Result<Self> {
        let stmt = db.prepare(WorldStatements::SEL_AREA_TRIGGER_TEMPLATE_IDS);
        let mut result = db.query(&stmt).await?;
        if result.is_empty() {
            return Ok(Self::default());
        }

        let mut keys = HashSet::new();
        loop {
            keys.insert((result.read(0), result.read(1)));
            if !result.next_row() {
                break;
            }
        }

        Ok(Self { keys })
    }

    pub fn contains(&self, id: u32, is_custom: bool) -> bool {
        self.keys.contains(&(id, is_custom))
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn area_trigger_template_store_keys_by_id_and_custom_flag_like_cpp() {
        let store = AreaTriggerTemplateStore::from_keys([(7, false), (7, true)]);

        assert!(store.contains(7, false));
        assert!(store.contains(7, true));
        assert!(!store.contains(8, false));
        assert_eq!(store.len(), 2);
    }
}
