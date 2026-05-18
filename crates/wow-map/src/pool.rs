//! Pure C++-shaped pool data helpers.
//!
//! Source of truth: TrinityCore `PoolMgr.h` / `PoolMgr.cpp` pool data and
//! `PoolGroup<T>` helpers. This module intentionally does not implement live
//! `PoolMgr` runtime, RNG, DB loading, entity creation, `SpawnPool`, or
//! `DespawnPool`. `Map::pool_data` remains the map-owned source of truth for
//! spawned pool state; `PoolGroupLikeCpp` is only foundation data for later
//! layers.

/// C++ `PoolTemplateData { uint32 MaxLimit; int32 MapId; }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolTemplateDataLikeCpp {
    pub max_limit: u32,
    pub map_id: i32,
}

impl PoolTemplateDataLikeCpp {
    #[must_use]
    pub const fn new(max_limit: u32, map_id: i32) -> Self {
        Self { max_limit, map_id }
    }
}

/// C++ `PoolObject { uint64 guid; float chance; }`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PoolObjectLikeCpp {
    pub guid: u64,
    pub chance: f32,
}

impl PoolObjectLikeCpp {
    #[must_use]
    pub const fn new(guid: u64, chance: f32) -> Self {
        Self { guid, chance }
    }
}

/// Tag for the C++ template parameter of `PoolGroup<T>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PoolMemberKindLikeCpp {
    Creature,
    GameObject,
    Pool,
}

/// Evidence returned by `PoolGroup<Pool>::RemoveOneRelation` representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PoolRelationRemovalLikeCpp {
    pub removed_explicit: bool,
    pub removed_equal: bool,
}

/// C++-shaped `PoolGroup<T>` buckets and pure helpers.
#[derive(Debug, Clone, PartialEq)]
pub struct PoolGroupLikeCpp {
    pool_id: u32,
    member_kind: PoolMemberKindLikeCpp,
    explicitly_chanced: Vec<PoolObjectLikeCpp>,
    equal_chanced: Vec<PoolObjectLikeCpp>,
}

impl PoolGroupLikeCpp {
    /// C++ constructor initializes `poolId` to zero.
    #[must_use]
    pub const fn new(member_kind: PoolMemberKindLikeCpp) -> Self {
        Self {
            pool_id: 0,
            member_kind,
            explicitly_chanced: Vec::new(),
            equal_chanced: Vec::new(),
        }
    }

    #[must_use]
    pub const fn with_pool_id(member_kind: PoolMemberKindLikeCpp, pool_id: u32) -> Self {
        Self {
            pool_id,
            member_kind,
            explicitly_chanced: Vec::new(),
            equal_chanced: Vec::new(),
        }
    }

    pub const fn set_pool_id_like_cpp(&mut self, pool_id: u32) {
        self.pool_id = pool_id;
    }

    #[must_use]
    pub const fn pool_id_like_cpp(&self) -> u32 {
        self.pool_id
    }

    #[must_use]
    pub const fn member_kind(&self) -> PoolMemberKindLikeCpp {
        self.member_kind
    }

    #[must_use]
    pub fn explicitly_chanced_like_cpp(&self) -> &[PoolObjectLikeCpp] {
        &self.explicitly_chanced
    }

    #[must_use]
    pub fn equal_chanced_like_cpp(&self) -> &[PoolObjectLikeCpp] {
        &self.equal_chanced
    }

    /// C++ `isEmpty()`: both chance buckets are empty.
    #[must_use]
    pub fn is_empty_like_cpp(&self) -> bool {
        self.explicitly_chanced.is_empty() && self.equal_chanced.is_empty()
    }

    /// C++ `isEmptyDeepCheck()`.
    ///
    /// For Creature/GameObject groups this is the normal `isEmpty()` helper and
    /// the child-pool closure is not called. For Pool-of-Pools groups this
    /// represents `sPoolMgr->IsEmpty(child_guid)`. Child GUIDs above `u32::MAX`
    /// are treated as non-empty rather than truncated silently.
    pub fn is_empty_deep_check_like_cpp(
        &self,
        mut is_child_pool_empty: impl FnMut(u32) -> bool,
    ) -> bool {
        if self.member_kind != PoolMemberKindLikeCpp::Pool {
            return self.is_empty_like_cpp();
        }

        for child in self
            .explicitly_chanced
            .iter()
            .chain(self.equal_chanced.iter())
        {
            let Ok(child_pool_id) = u32::try_from(child.guid) else {
                return false;
            };
            if !is_child_pool_empty(child_pool_id) {
                return false;
            }
        }

        true
    }

    /// C++ `AddEntry`: non-zero chance with maxentries one is explicit;
    /// everything else is equal-chanced.
    pub fn add_entry_like_cpp(&mut self, pool_object: PoolObjectLikeCpp, maxentries: u32) {
        if pool_object.chance != 0.0 && maxentries == 1 {
            self.explicitly_chanced.push(pool_object);
        } else {
            self.equal_chanced.push(pool_object);
        }
    }

    /// C++ `CheckPool`: validate explicit total only when equal-chanced is empty.
    #[must_use]
    pub fn check_pool_like_cpp(&self) -> bool {
        if self.equal_chanced.is_empty() {
            let chance = self
                .explicitly_chanced
                .iter()
                .map(|entry| entry.chance)
                .sum::<f32>();
            if chance != 100.0 && chance != 0.0 {
                return false;
            }
        }

        true
    }

    /// C++ specialization `PoolGroup<Pool>::RemoveOneRelation`.
    ///
    /// Creature/GameObject groups have no specialization in C++; this pure Rust
    /// helper treats them as an explicit no-op. For Pool groups, it removes the
    /// first matching child from `ExplicitlyChanced` and then the first matching
    /// child from `EqualChanced`, so one match can be removed from each bucket.
    pub fn remove_one_relation_like_cpp(
        &mut self,
        child_pool_id: u32,
    ) -> PoolRelationRemovalLikeCpp {
        if self.member_kind != PoolMemberKindLikeCpp::Pool {
            return PoolRelationRemovalLikeCpp::default();
        }

        let mut removal = PoolRelationRemovalLikeCpp::default();
        let child_pool_id = u64::from(child_pool_id);

        if let Some(index) = self
            .explicitly_chanced
            .iter()
            .position(|entry| entry.guid == child_pool_id)
        {
            self.explicitly_chanced.remove(index);
            removal.removed_explicit = true;
        }

        if let Some(index) = self
            .equal_chanced
            .iter()
            .position(|entry| entry.guid == child_pool_id)
        {
            self.equal_chanced.remove(index);
            removal.removed_equal = true;
        }

        removal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_group_add_entry_buckets_match_cpp() {
        let mut group = PoolGroupLikeCpp::new(PoolMemberKindLikeCpp::Creature);

        group.add_entry_like_cpp(PoolObjectLikeCpp::new(1, 25.0), 1);
        group.add_entry_like_cpp(PoolObjectLikeCpp::new(2, 0.0), 1);
        group.add_entry_like_cpp(PoolObjectLikeCpp::new(3, 25.0), 2);

        assert_eq!(
            group.explicitly_chanced_like_cpp(),
            &[PoolObjectLikeCpp::new(1, 25.0)]
        );
        assert_eq!(
            group.equal_chanced_like_cpp(),
            &[
                PoolObjectLikeCpp::new(2, 0.0),
                PoolObjectLikeCpp::new(3, 25.0),
            ]
        );
    }

    #[test]
    fn pool_group_check_pool_matches_cpp() {
        let mut valid_explicit = PoolGroupLikeCpp::new(PoolMemberKindLikeCpp::GameObject);
        valid_explicit.add_entry_like_cpp(PoolObjectLikeCpp::new(1, 60.0), 1);
        valid_explicit.add_entry_like_cpp(PoolObjectLikeCpp::new(2, 40.0), 1);
        assert!(valid_explicit.check_pool_like_cpp());

        let mut invalid_explicit = PoolGroupLikeCpp::new(PoolMemberKindLikeCpp::GameObject);
        invalid_explicit.add_entry_like_cpp(PoolObjectLikeCpp::new(1, 60.0), 1);
        assert!(!invalid_explicit.check_pool_like_cpp());

        let mut zero_total = PoolGroupLikeCpp::new(PoolMemberKindLikeCpp::GameObject);
        zero_total.add_entry_like_cpp(PoolObjectLikeCpp::new(1, 0.0), 1);
        assert!(zero_total.check_pool_like_cpp());

        let mut equal_present = PoolGroupLikeCpp::new(PoolMemberKindLikeCpp::GameObject);
        equal_present.add_entry_like_cpp(PoolObjectLikeCpp::new(1, 60.0), 1);
        equal_present.add_entry_like_cpp(PoolObjectLikeCpp::new(2, 0.0), 1);
        assert!(equal_present.check_pool_like_cpp());
    }

    #[test]
    fn pool_group_empty_deep_check_matches_cpp() {
        let empty_creature = PoolGroupLikeCpp::new(PoolMemberKindLikeCpp::Creature);
        let mut creature_closure_calls = 0;
        assert!(empty_creature.is_empty_deep_check_like_cpp(|_| {
            creature_closure_calls += 1;
            false
        }));
        assert_eq!(creature_closure_calls, 0);

        let mut gameobject = PoolGroupLikeCpp::new(PoolMemberKindLikeCpp::GameObject);
        gameobject.add_entry_like_cpp(PoolObjectLikeCpp::new(1, 0.0), 1);
        let mut gameobject_closure_calls = 0;
        assert!(!gameobject.is_empty_deep_check_like_cpp(|_| {
            gameobject_closure_calls += 1;
            true
        }));
        assert_eq!(gameobject_closure_calls, 0);

        let mut pool = PoolGroupLikeCpp::new(PoolMemberKindLikeCpp::Pool);
        pool.add_entry_like_cpp(PoolObjectLikeCpp::new(10, 50.0), 1);
        pool.add_entry_like_cpp(PoolObjectLikeCpp::new(20, 0.0), 1);
        let mut visited = Vec::new();
        assert!(!pool.is_empty_deep_check_like_cpp(|child_pool_id| {
            visited.push(child_pool_id);
            child_pool_id != 20
        }));
        assert_eq!(visited, vec![10, 20]);

        let mut overflowing_pool = PoolGroupLikeCpp::new(PoolMemberKindLikeCpp::Pool);
        overflowing_pool
            .add_entry_like_cpp(PoolObjectLikeCpp::new(u64::from(u32::MAX) + 1, 1.0), 1);
        assert!(!overflowing_pool.is_empty_deep_check_like_cpp(|_| true));
    }

    #[test]
    fn pool_group_remove_one_relation_matches_cpp() {
        let mut pool = PoolGroupLikeCpp::new(PoolMemberKindLikeCpp::Pool);
        pool.add_entry_like_cpp(PoolObjectLikeCpp::new(10, 50.0), 1);
        pool.add_entry_like_cpp(PoolObjectLikeCpp::new(10, 40.0), 1);
        pool.add_entry_like_cpp(PoolObjectLikeCpp::new(10, 0.0), 1);
        pool.add_entry_like_cpp(PoolObjectLikeCpp::new(10, 0.0), 2);
        pool.add_entry_like_cpp(PoolObjectLikeCpp::new(11, 0.0), 1);

        let removal = pool.remove_one_relation_like_cpp(10);
        assert_eq!(
            removal,
            PoolRelationRemovalLikeCpp {
                removed_explicit: true,
                removed_equal: true,
            }
        );
        assert_eq!(
            pool.explicitly_chanced_like_cpp(),
            &[PoolObjectLikeCpp::new(10, 40.0)]
        );
        assert_eq!(
            pool.equal_chanced_like_cpp(),
            &[
                PoolObjectLikeCpp::new(10, 0.0),
                PoolObjectLikeCpp::new(11, 0.0),
            ]
        );

        let mut creature = PoolGroupLikeCpp::new(PoolMemberKindLikeCpp::Creature);
        creature.add_entry_like_cpp(PoolObjectLikeCpp::new(10, 50.0), 1);
        assert_eq!(
            creature.remove_one_relation_like_cpp(10),
            PoolRelationRemovalLikeCpp::default()
        );
        assert_eq!(
            creature.explicitly_chanced_like_cpp(),
            &[PoolObjectLikeCpp::new(10, 50.0)]
        );
    }
}
