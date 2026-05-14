use std::collections::HashMap;

use anyhow::Result;
use rand::Rng;
use wow_database::WorldDatabase;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CreatureTemplateMountModelLikeCpp {
    pub display_id: u32,
    pub display_scale: f32,
    pub probability: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreatureTemplateMountEntryLikeCpp {
    pub entry: u32,
    pub vehicle_id: u32,
    pub models: Vec<CreatureTemplateMountModelLikeCpp>,
}

#[derive(Debug, Clone, Default)]
pub struct CreatureTemplateMountStoreLikeCpp {
    entries: HashMap<u32, CreatureTemplateMountEntryLikeCpp>,
}

impl CreatureTemplateMountStoreLikeCpp {
    pub fn from_entries(
        entries: impl IntoIterator<Item = CreatureTemplateMountEntryLikeCpp>,
    ) -> Self {
        Self {
            entries: entries
                .into_iter()
                .map(|entry| (entry.entry, entry))
                .collect(),
        }
    }

    pub async fn load_like_cpp(db: &WorldDatabase) -> Result<Self> {
        let mut result = db
            .direct_query(
                "SELECT ct.entry, ct.VehicleId, ctm.CreatureDisplayID, ctm.DisplayScale, ctm.Probability \
                 FROM creature_template ct \
                 LEFT JOIN creature_template_model ctm ON ct.entry = ctm.CreatureID \
                 ORDER BY ct.entry, ctm.Idx",
            )
            .await?;

        if result.is_empty() {
            return Ok(Self::default());
        }

        let mut entries = HashMap::new();
        loop {
            let entry_id = result.read::<u32>(0);
            let vehicle_id = result.try_read::<u32>(1).unwrap_or(0);
            let display_id = result.try_read::<u32>(2).unwrap_or(0);
            let display_scale = result.try_read::<f32>(3).unwrap_or(0.0);
            let probability = result.try_read::<f32>(4).unwrap_or(0.0);

            let entry =
                entries
                    .entry(entry_id)
                    .or_insert_with(|| CreatureTemplateMountEntryLikeCpp {
                        entry: entry_id,
                        vehicle_id,
                        models: Vec::new(),
                    });
            entry.vehicle_id = vehicle_id;
            if display_id != 0 {
                entry.models.push(CreatureTemplateMountModelLikeCpp {
                    display_id,
                    display_scale,
                    probability,
                });
            }

            if !result.next_row() {
                break;
            }
        }

        Ok(Self { entries })
    }

    pub fn get(&self, entry: u32) -> Option<&CreatureTemplateMountEntryLikeCpp> {
        self.entries.get(&entry)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl CreatureTemplateMountEntryLikeCpp {
    pub fn choose_display_id_like_cpp<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<u32> {
        match self.models.as_slice() {
            [] => None,
            [model] => Some(model.display_id),
            models => {
                let total: f32 = models.iter().map(|model| model.probability.max(0.0)).sum();
                if total <= f32::EPSILON {
                    return models.first().map(|model| model.display_id);
                }

                let mut roll = rng.gen_range(0.0..total);
                for model in models {
                    roll -= model.probability.max(0.0);
                    if roll <= 0.0 {
                        return Some(model.display_id);
                    }
                }

                models.last().map(|model| model.display_id)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, rngs::StdRng};

    use super::*;

    #[test]
    fn creature_template_mount_model_selection_matches_cpp_shape() {
        let entry = CreatureTemplateMountEntryLikeCpp {
            entry: 10,
            vehicle_id: 77,
            models: vec![CreatureTemplateMountModelLikeCpp {
                display_id: 1234,
                display_scale: 1.0,
                probability: 0.0,
            }],
        };

        assert_eq!(
            entry.choose_display_id_like_cpp(&mut StdRng::seed_from_u64(1)),
            Some(1234)
        );

        let entry = CreatureTemplateMountEntryLikeCpp {
            entry: 11,
            vehicle_id: 0,
            models: vec![
                CreatureTemplateMountModelLikeCpp {
                    display_id: 1,
                    display_scale: 1.0,
                    probability: 0.0,
                },
                CreatureTemplateMountModelLikeCpp {
                    display_id: 2,
                    display_scale: 1.0,
                    probability: 100.0,
                },
            ],
        };

        assert_eq!(
            entry.choose_display_id_like_cpp(&mut StdRng::seed_from_u64(2)),
            Some(2)
        );
    }
}
