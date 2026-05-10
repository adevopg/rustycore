// Copyright (c) 2026 alseif0x
// RustyCore - WoW WotLK 3.4.3 server in Rust
// Based on TrinityCore protocol research (https://github.com/TrinityCore/TrinityCore)
// Licensed under GPL v3 - https://www.gnu.org/licenses/gpl-3.0.html

//! ImportPrice*.db2 readers used by C++ `Item::GetBuyPrice`.

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use tracing::info;

use crate::wdc4::Wdc4Reader;

/// C++ `ImportPriceArmorEntry`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImportPriceArmorEntry {
    pub id: u32,
    pub cloth_modifier: f32,
    pub leather_modifier: f32,
    pub chain_modifier: f32,
    pub plate_modifier: f32,
}

/// C++ `ImportPriceQualityEntry`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImportPriceQualityEntry {
    pub id: u32,
    pub data: f32,
}

/// C++ `ImportPriceShieldEntry`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImportPriceShieldEntry {
    pub id: u32,
    pub data: f32,
}

/// C++ `ImportPriceWeaponEntry`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImportPriceWeaponEntry {
    pub id: u32,
    pub data: f32,
}

pub struct ImportPriceArmorStore {
    entries: HashMap<u32, ImportPriceArmorEntry>,
}

pub struct ImportPriceQualityStore {
    entries: HashMap<u32, ImportPriceQualityEntry>,
}

pub struct ImportPriceShieldStore {
    entries: HashMap<u32, ImportPriceShieldEntry>,
}

pub struct ImportPriceWeaponStore {
    entries: HashMap<u32, ImportPriceWeaponEntry>,
}

/// Shared aggregate matching the four C++ global import-price stores.
pub struct ImportPriceStores {
    pub armor: ImportPriceArmorStore,
    pub quality: ImportPriceQualityStore,
    pub shield: ImportPriceShieldStore,
    pub weapon: ImportPriceWeaponStore,
}

impl ImportPriceStores {
    pub fn load(data_dir: &str, locale: &str) -> Result<Self> {
        Ok(Self {
            armor: ImportPriceArmorStore::load(data_dir, locale)?,
            quality: ImportPriceQualityStore::load(data_dir, locale)?,
            shield: ImportPriceShieldStore::load(data_dir, locale)?,
            weapon: ImportPriceWeaponStore::load(data_dir, locale)?,
        })
    }
}

impl ImportPriceArmorStore {
    pub fn from_entries(entries: impl IntoIterator<Item = ImportPriceArmorEntry>) -> Self {
        Self {
            entries: entries.into_iter().map(|entry| (entry.id, entry)).collect(),
        }
    }

    /// Load ImportPriceArmor.db2 from `{data_dir}/dbc/{locale}/ImportPriceArmor.db2`.
    ///
    /// C++ refs:
    /// - `DB2Structure.h::ImportPriceArmorEntry`
    /// - `DB2LoadInfo.h::ImportPriceArmorLoadInfo`
    /// - `Item::GetBuyPrice`
    pub fn load(data_dir: &str, locale: &str) -> Result<Self> {
        let reader = open_import_price_reader(data_dir, locale, "ImportPriceArmor.db2")?;
        let base = field_base(&reader);
        let mut entries = HashMap::with_capacity(reader.total_count());

        for (id, idx) in reader.iter_records() {
            entries.insert(
                id,
                ImportPriceArmorEntry {
                    id,
                    cloth_modifier: read_f32(&reader, idx, base),
                    leather_modifier: read_f32(&reader, idx, base + 1),
                    chain_modifier: read_f32(&reader, idx, base + 2),
                    plate_modifier: read_f32(&reader, idx, base + 3),
                },
            );
        }

        info!("Loaded {} import price armor rows", entries.len());
        Ok(Self { entries })
    }

    pub fn get(&self, id: u32) -> Option<&ImportPriceArmorEntry> {
        self.entries.get(&id)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl ImportPriceQualityStore {
    pub fn from_entries(entries: impl IntoIterator<Item = ImportPriceQualityEntry>) -> Self {
        Self {
            entries: entries.into_iter().map(|entry| (entry.id, entry)).collect(),
        }
    }

    pub fn load(data_dir: &str, locale: &str) -> Result<Self> {
        let reader = open_import_price_reader(data_dir, locale, "ImportPriceQuality.db2")?;
        let data_field = single_data_field(&reader);
        let mut entries = HashMap::with_capacity(reader.total_count());

        for (id, idx) in reader.iter_records() {
            entries.insert(
                id,
                ImportPriceQualityEntry {
                    id,
                    data: read_f32(&reader, idx, data_field),
                },
            );
        }

        info!("Loaded {} import price quality rows", entries.len());
        Ok(Self { entries })
    }

    pub fn get(&self, id: u32) -> Option<&ImportPriceQualityEntry> {
        self.entries.get(&id)
    }
}

impl ImportPriceShieldStore {
    pub fn from_entries(entries: impl IntoIterator<Item = ImportPriceShieldEntry>) -> Self {
        Self {
            entries: entries.into_iter().map(|entry| (entry.id, entry)).collect(),
        }
    }

    pub fn load(data_dir: &str, locale: &str) -> Result<Self> {
        let reader = open_import_price_reader(data_dir, locale, "ImportPriceShield.db2")?;
        let data_field = single_data_field(&reader);
        let mut entries = HashMap::with_capacity(reader.total_count());

        for (id, idx) in reader.iter_records() {
            entries.insert(
                id,
                ImportPriceShieldEntry {
                    id,
                    data: read_f32(&reader, idx, data_field),
                },
            );
        }

        info!("Loaded {} import price shield rows", entries.len());
        Ok(Self { entries })
    }

    pub fn get(&self, id: u32) -> Option<&ImportPriceShieldEntry> {
        self.entries.get(&id)
    }
}

impl ImportPriceWeaponStore {
    pub fn from_entries(entries: impl IntoIterator<Item = ImportPriceWeaponEntry>) -> Self {
        Self {
            entries: entries.into_iter().map(|entry| (entry.id, entry)).collect(),
        }
    }

    pub fn load(data_dir: &str, locale: &str) -> Result<Self> {
        let reader = open_import_price_reader(data_dir, locale, "ImportPriceWeapon.db2")?;
        let data_field = single_data_field(&reader);
        let mut entries = HashMap::with_capacity(reader.total_count());

        for (id, idx) in reader.iter_records() {
            entries.insert(
                id,
                ImportPriceWeaponEntry {
                    id,
                    data: read_f32(&reader, idx, data_field),
                },
            );
        }

        info!("Loaded {} import price weapon rows", entries.len());
        Ok(Self { entries })
    }

    pub fn get(&self, id: u32) -> Option<&ImportPriceWeaponEntry> {
        self.entries.get(&id)
    }
}

fn open_import_price_reader(data_dir: &str, locale: &str, file_name: &str) -> Result<Wdc4Reader> {
    let path = Path::new(data_dir).join("dbc").join(locale).join(file_name);
    Wdc4Reader::open(&path).with_context(|| format!("failed to open {}", path.display()))
}

fn field_base(reader: &Wdc4Reader) -> usize {
    if reader.field_count() >= 5 { 1 } else { 0 }
}

fn single_data_field(reader: &Wdc4Reader) -> usize {
    if reader.field_count() >= 2 { 1 } else { 0 }
}

fn read_f32(reader: &Wdc4Reader, record_idx: usize, field: usize) -> f32 {
    f32::from_bits(reader.get_field_u32(record_idx, field))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_price_stores_index_entries_like_cpp_globals() {
        let armor = ImportPriceArmorStore::from_entries([ImportPriceArmorEntry {
            id: 5,
            cloth_modifier: 1.0,
            leather_modifier: 1.1,
            chain_modifier: 1.2,
            plate_modifier: 1.3,
        }]);
        let quality =
            ImportPriceQualityStore::from_entries([ImportPriceQualityEntry { id: 4, data: 2.5 }]);
        let shield =
            ImportPriceShieldStore::from_entries([ImportPriceShieldEntry { id: 2, data: 0.75 }]);
        let weapon =
            ImportPriceWeaponStore::from_entries([ImportPriceWeaponEntry { id: 3, data: 1.75 }]);

        assert_eq!(armor.get(5).unwrap().plate_modifier, 1.3);
        assert_eq!(quality.get(4).unwrap().data, 2.5);
        assert_eq!(shield.get(2).unwrap().data, 0.75);
        assert_eq!(weapon.get(3).unwrap().data, 1.75);
        assert!(armor.get(99).is_none());
    }
}
