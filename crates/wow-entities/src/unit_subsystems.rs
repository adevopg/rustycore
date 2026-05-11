use std::collections::{HashMap, HashSet, VecDeque};

use wow_constants::{SpellState, UnitState};
use wow_core::ObjectGuid;

/// Minimal bridge for TrinityCore `Unit` aura containers.
///
/// This is metadata/state only: it does not run aura scripts, periodic ticks, proc logic,
/// packet emission, or update-field masking by itself.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AuraSubsystem {
    pub owned_auras: Vec<OwnedAuraRef>,
    pub applied_auras: Vec<AppliedAuraRef>,
    pub visible_auras: HashMap<u8, AuraRef>,
    pub removed_auras: Vec<AuraRef>,
    pub interrupt_flags: u32,
    pub interrupt_flags2: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AuraRef {
    pub spell_id: u32,
    pub caster_guid: ObjectGuid,
}

impl AuraRef {
    pub const fn new(spell_id: u32, caster_guid: ObjectGuid) -> Self {
        Self {
            spell_id,
            caster_guid,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OwnedAuraRef {
    pub spell_id: u32,
    pub caster_guid: ObjectGuid,
    pub item_caster_guid: Option<ObjectGuid>,
}

impl OwnedAuraRef {
    pub const fn new(
        spell_id: u32,
        caster_guid: ObjectGuid,
        item_caster_guid: Option<ObjectGuid>,
    ) -> Self {
        Self {
            spell_id,
            caster_guid,
            item_caster_guid,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AppliedAuraRef {
    pub spell_id: u32,
    pub caster_guid: ObjectGuid,
    pub slot: u8,
    pub effect_mask: u32,
}

impl AppliedAuraRef {
    pub const fn new(spell_id: u32, caster_guid: ObjectGuid, slot: u8, effect_mask: u32) -> Self {
        Self {
            spell_id,
            caster_guid,
            slot,
            effect_mask,
        }
    }
}

impl AuraSubsystem {
    pub fn add_owned(&mut self, aura: OwnedAuraRef) {
        if !self.owned_auras.contains(&aura) {
            self.owned_auras.push(aura);
        }
    }

    pub fn remove_owned(&mut self, aura: OwnedAuraRef) -> bool {
        let before = self.owned_auras.len();
        self.owned_auras.retain(|known| *known != aura);
        before != self.owned_auras.len()
    }

    pub fn has_owned(&self, aura: OwnedAuraRef) -> bool {
        self.owned_auras.contains(&aura)
    }

    pub fn add_applied(&mut self, aura: AppliedAuraRef) {
        if !self.applied_auras.contains(&aura) {
            self.applied_auras.push(aura);
        }
    }

    pub fn remove_applied(&mut self, aura: AppliedAuraRef) -> bool {
        let before = self.applied_auras.len();
        self.applied_auras.retain(|known| *known != aura);
        before != self.applied_auras.len()
    }

    pub fn has_applied(&self, aura: AppliedAuraRef) -> bool {
        self.applied_auras.contains(&aura)
    }

    pub fn set_visible(&mut self, slot: u8, aura: AuraRef) {
        self.visible_auras.insert(slot, aura);
    }

    pub fn clear_visible(&mut self, slot: u8) -> Option<AuraRef> {
        self.visible_auras.remove(&slot)
    }

    pub fn mark_removed(&mut self, aura: AuraRef) {
        self.removed_auras.push(aura);
    }

    pub fn clear_removed(&mut self) {
        self.removed_auras.clear();
    }

    pub fn removed_count(&self) -> usize {
        self.removed_auras.len()
    }
}

/// Trinity-compatible current spell slots represented in RustyCore state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CurrentSpellSlot {
    Melee = 0,
    Generic = 1,
    Channeled = 2,
    Autorepeat = 3,
}

pub const CURRENT_FIRST_NON_MELEE_SPELL: u8 = 1;
pub const CURRENT_MAX_SPELL: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentSpellRef {
    pub spell_id: u32,
    pub caster_guid: Option<ObjectGuid>,
    pub cast_id: Option<ObjectGuid>,
    pub cast_time_ms: u32,
    pub state: SpellState,
    pub interruptible: bool,
    pub allow_actions_during_channel: bool,
}

impl CurrentSpellRef {
    pub const fn new(
        spell_id: u32,
        caster_guid: Option<ObjectGuid>,
        cast_id: Option<ObjectGuid>,
    ) -> Self {
        Self {
            spell_id,
            caster_guid,
            cast_id,
            cast_time_ms: 0,
            state: SpellState::None,
            interruptible: true,
            allow_actions_during_channel: false,
        }
    }

    pub const fn with_cast_time_ms(mut self, cast_time_ms: u32) -> Self {
        self.cast_time_ms = cast_time_ms;
        self
    }

    pub const fn with_state(mut self, state: SpellState) -> Self {
        self.state = state;
        self
    }

    pub const fn with_interruptible(mut self, interruptible: bool) -> Self {
        self.interruptible = interruptible;
        self
    }

    pub const fn with_allow_actions_during_channel(
        mut self,
        allow_actions_during_channel: bool,
    ) -> Self {
        self.allow_actions_during_channel = allow_actions_during_channel;
        self
    }
}

pub const MAX_SPELL_SCHOOL: usize = 7;
pub const INFINITY_COOLDOWN_DELAY_MS: u64 = 30 * 24 * 60 * 60 * 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpellCooldown {
    pub spell_id: u32,
    pub item_id: u32,
    pub cooldown_end_ms: u64,
    pub category_id: u32,
    pub category_end_ms: u64,
    pub on_hold: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpellChargeState {
    pub recharge_start_ms: u64,
    pub recharge_end_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SpellHistory {
    pub cooldowns: HashMap<u32, SpellCooldown>,
    pub cooldowns_before_duel: HashMap<u32, SpellCooldown>,
    pub category_cooldowns: HashMap<u32, u32>,
    pub school_lockouts: [u64; MAX_SPELL_SCHOOL],
    pub charges: HashMap<u32, VecDeque<SpellChargeState>>,
    pub global_cooldowns: HashMap<u32, u64>,
}

impl SpellHistory {
    pub fn start_cooldown(
        &mut self,
        now_ms: u64,
        spell_id: u32,
        item_id: u32,
        cooldown_ms: u64,
        category_id: u32,
        category_cooldown_ms: u64,
        on_hold: bool,
    ) -> bool {
        let (cooldown_end_ms, category_end_ms) = if on_hold {
            (
                if cooldown_ms > 0 {
                    now_ms + INFINITY_COOLDOWN_DELAY_MS
                } else if category_cooldown_ms > 0 {
                    now_ms + INFINITY_COOLDOWN_DELAY_MS
                } else {
                    now_ms
                },
                if category_cooldown_ms > 0 {
                    now_ms + INFINITY_COOLDOWN_DELAY_MS
                } else {
                    now_ms
                },
            )
        } else {
            (
                if cooldown_ms > 0 {
                    now_ms + cooldown_ms
                } else if category_cooldown_ms > 0 {
                    now_ms + category_cooldown_ms
                } else {
                    now_ms
                },
                if category_cooldown_ms > 0 {
                    now_ms + category_cooldown_ms
                } else {
                    now_ms
                },
            )
        };

        if cooldown_end_ms == now_ms && category_end_ms == now_ms {
            return false;
        }

        self.add_cooldown(
            spell_id,
            item_id,
            cooldown_end_ms,
            category_id,
            category_end_ms,
            on_hold,
        )
    }

    pub fn set_cooldown(&mut self, spell_id: u32, started_at_ms: u64, duration_ms: u32) {
        self.start_cooldown(
            started_at_ms,
            spell_id,
            0,
            u64::from(duration_ms),
            0,
            0,
            false,
        );
    }

    pub fn add_cooldown(
        &mut self,
        spell_id: u32,
        item_id: u32,
        cooldown_end_ms: u64,
        category_id: u32,
        category_end_ms: u64,
        on_hold: bool,
    ) -> bool {
        let should_replace = self.cooldowns.get(&spell_id).is_none_or(|current| {
            cooldown_end_ms > current.cooldown_end_ms
                || category_end_ms > current.category_end_ms
                || on_hold
        });

        if !should_replace {
            return false;
        }

        self.cooldowns.insert(
            spell_id,
            SpellCooldown {
                spell_id,
                item_id,
                cooldown_end_ms,
                category_id,
                category_end_ms,
                on_hold,
            },
        );

        if category_id != 0 {
            self.category_cooldowns.insert(category_id, spell_id);
        }

        true
    }

    pub fn cooldown(&self, spell_id: u32) -> Option<SpellCooldown> {
        self.cooldowns.get(&spell_id).copied()
    }

    pub fn has_cooldown(&self, spell_id: u32, category_id: u32, now_ms: u64) -> bool {
        self.cooldowns
            .get(&spell_id)
            .is_some_and(|cooldown| cooldown.on_hold || cooldown.cooldown_end_ms > now_ms)
            || (category_id != 0
                && self
                    .category_cooldowns
                    .get(&category_id)
                    .and_then(|spell_id| self.cooldowns.get(spell_id))
                    .is_some_and(|cooldown| cooldown.on_hold || cooldown.category_end_ms > now_ms))
    }

    pub fn remaining_cooldown_ms(&self, spell_id: u32, category_id: u32, now_ms: u64) -> u64 {
        if let Some(cooldown) = self.cooldowns.get(&spell_id) {
            return cooldown.cooldown_end_ms.saturating_sub(now_ms);
        }

        self.remaining_category_cooldown_ms(category_id, now_ms)
    }

    pub fn remaining_category_cooldown_ms(&self, category_id: u32, now_ms: u64) -> u64 {
        self.category_cooldowns
            .get(&category_id)
            .and_then(|spell_id| self.cooldowns.get(spell_id))
            .map_or(0, |cooldown| {
                cooldown.category_end_ms.saturating_sub(now_ms)
            })
    }

    pub fn modify_cooldown(
        &mut self,
        spell_id: u32,
        cooldown_delta_ms: i64,
        without_category_cooldown: bool,
        now_ms: u64,
    ) -> bool {
        if cooldown_delta_ms == 0 {
            return false;
        }

        let Some(cooldown) = self.cooldowns.get_mut(&spell_id) else {
            return false;
        };

        cooldown.cooldown_end_ms = apply_ms_delta(cooldown.cooldown_end_ms, cooldown_delta_ms);
        if cooldown.category_id != 0 {
            if !without_category_cooldown {
                cooldown.category_end_ms =
                    apply_ms_delta(cooldown.category_end_ms, cooldown_delta_ms);
            }
            if cooldown.cooldown_end_ms < cooldown.category_end_ms {
                cooldown.cooldown_end_ms = cooldown.category_end_ms;
            }
        }

        if cooldown.cooldown_end_ms <= now_ms && !cooldown.on_hold {
            self.clear_cooldown(spell_id);
        }

        true
    }

    pub fn clear_cooldown(&mut self, spell_id: u32) -> bool {
        let Some(cooldown) = self.cooldowns.remove(&spell_id) else {
            return false;
        };
        if cooldown.category_id != 0 {
            self.category_cooldowns.remove(&cooldown.category_id);
        }
        true
    }

    pub fn reset_all_cooldowns(&mut self) {
        self.cooldowns.clear();
        self.category_cooldowns.clear();
    }

    pub fn set_charges(
        &mut self,
        charge_category_id: u32,
        charges: u8,
        started_at_ms: u64,
        recharge_ms: u32,
    ) {
        let queue = self.charges.entry(charge_category_id).or_default();
        queue.clear();
        let mut start = started_at_ms;
        for _ in 0..charges {
            let end = start + u64::from(recharge_ms);
            queue.push_back(SpellChargeState {
                recharge_start_ms: start,
                recharge_end_ms: end,
            });
            start = end;
        }
    }

    pub fn charges(&self, charge_category_id: u32) -> Option<&VecDeque<SpellChargeState>> {
        self.charges.get(&charge_category_id)
    }

    pub fn consumed_charges(&self, charge_category_id: u32) -> u8 {
        self.charges
            .get(&charge_category_id)
            .map_or(0, |charges| charges.len().min(u8::MAX as usize) as u8)
    }

    pub fn has_charge(&self, charge_category_id: u32, max_charges: i32) -> bool {
        charge_category_id == 0
            || max_charges <= 0
            || self
                .charges
                .get(&charge_category_id)
                .is_none_or(|charges| charges.len() < max_charges as usize)
    }

    pub fn consume_charge(
        &mut self,
        charge_category_id: u32,
        now_ms: u64,
        recovery_ms: u32,
        max_charges: i32,
    ) -> bool {
        if charge_category_id == 0 || recovery_ms == 0 || max_charges <= 0 {
            return false;
        }

        let queue = self.charges.entry(charge_category_id).or_default();
        let recharge_start_ms = queue.back().map_or(now_ms, |charge| charge.recharge_end_ms);
        queue.push_back(SpellChargeState {
            recharge_start_ms,
            recharge_end_ms: recharge_start_ms + u64::from(recovery_ms),
        });
        true
    }

    pub fn modify_charge_recovery_time(
        &mut self,
        charge_category_id: u32,
        cooldown_delta_ms: i64,
        now_ms: u64,
    ) -> bool {
        let Some(queue) = self.charges.get_mut(&charge_category_id) else {
            return false;
        };
        if queue.is_empty() {
            return false;
        }

        for charge in queue.iter_mut() {
            charge.recharge_start_ms = apply_ms_delta(charge.recharge_start_ms, cooldown_delta_ms);
            charge.recharge_end_ms = apply_ms_delta(charge.recharge_end_ms, cooldown_delta_ms);
        }

        while queue
            .front()
            .is_some_and(|charge| charge.recharge_end_ms < now_ms)
        {
            queue.pop_front();
        }

        true
    }

    pub fn restore_charge(&mut self, charge_category_id: u32) -> bool {
        self.charges
            .get_mut(&charge_category_id)
            .and_then(VecDeque::pop_back)
            .is_some()
    }

    pub fn clear_charges(&mut self, charge_category_id: u32) -> bool {
        self.charges.remove(&charge_category_id).is_some()
    }

    pub fn reset_all_charges(&mut self) {
        self.charges.clear();
    }

    pub fn lock_spell_school(&mut self, school_mask: u32, now_ms: u64, lockout_ms: u64) {
        let lockout_end = now_ms + lockout_ms;
        for school in 0..MAX_SPELL_SCHOOL {
            if (school_mask & (1 << school)) != 0 {
                self.school_lockouts[school] = lockout_end;
            }
        }
    }

    pub fn is_school_locked(&self, school_mask: u32, now_ms: u64) -> bool {
        (0..MAX_SPELL_SCHOOL).any(|school| {
            (school_mask & (1 << school)) != 0 && self.school_lockouts[school] > now_ms
        })
    }

    pub fn add_global_cooldown(
        &mut self,
        recovery_category_id: u32,
        now_ms: u64,
        duration_ms: u64,
    ) {
        self.global_cooldowns
            .insert(recovery_category_id, now_ms + duration_ms);
    }

    pub fn has_global_cooldown(&self, recovery_category_id: u32, now_ms: u64) -> bool {
        self.global_cooldowns
            .get(&recovery_category_id)
            .is_some_and(|end_ms| *end_ms > now_ms)
    }

    pub fn cancel_global_cooldown(&mut self, recovery_category_id: u32) {
        self.global_cooldowns.insert(recovery_category_id, 0);
    }

    pub fn remaining_global_cooldown_ms(&self, recovery_category_id: u32, now_ms: u64) -> u64 {
        self.global_cooldowns
            .get(&recovery_category_id)
            .map_or(0, |end_ms| end_ms.saturating_sub(now_ms))
    }

    pub fn save_cooldown_state_before_duel(&mut self) {
        self.cooldowns_before_duel = self.cooldowns.clone();
    }

    pub fn restore_cooldown_state_after_duel(&mut self) {
        self.cooldowns = self.cooldowns_before_duel.clone();
        self.category_cooldowns.clear();
        for (spell_id, cooldown) in &self.cooldowns {
            if cooldown.category_id != 0 {
                self.category_cooldowns
                    .insert(cooldown.category_id, *spell_id);
            }
        }
    }

    pub fn update(&mut self, now_ms: u64) {
        self.category_cooldowns.retain(|_, spell_id| {
            self.cooldowns
                .get(spell_id)
                .is_some_and(|cooldown| cooldown.on_hold || cooldown.category_end_ms >= now_ms)
        });

        let expired: Vec<u32> = self
            .cooldowns
            .iter()
            .filter_map(|(spell_id, cooldown)| {
                (!cooldown.on_hold && cooldown.cooldown_end_ms < now_ms).then_some(*spell_id)
            })
            .collect();
        for spell_id in expired {
            self.clear_cooldown(spell_id);
        }

        for queue in self.charges.values_mut() {
            while queue
                .front()
                .is_some_and(|charge| charge.recharge_end_ms <= now_ms)
            {
                queue.pop_front();
            }
        }
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

fn apply_ms_delta(value: u64, delta: i64) -> u64 {
    if delta.is_negative() {
        value.saturating_sub(delta.unsigned_abs())
    } else {
        value.saturating_add(delta as u64)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SpellSubsystem {
    pub current_spells: HashMap<CurrentSpellSlot, CurrentSpellRef>,
    pub history: SpellHistory,
}

impl SpellSubsystem {
    pub fn set_current_spell(&mut self, slot: CurrentSpellSlot, spell: CurrentSpellRef) {
        self.current_spells.insert(slot, spell);
    }

    pub fn current_spell(&self, slot: CurrentSpellSlot) -> Option<CurrentSpellRef> {
        self.current_spells.get(&slot).copied()
    }

    pub fn clear_current_spell(&mut self, slot: CurrentSpellSlot) -> Option<CurrentSpellRef> {
        self.current_spells.remove(&slot)
    }

    pub fn clear_current_spells(&mut self) {
        self.current_spells.clear();
    }

    pub fn find_current_spell_by_spell_id(&self, spell_id: u32) -> Option<CurrentSpellRef> {
        self.current_spells
            .values()
            .find(|spell| spell.spell_id == spell_id)
            .copied()
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CombatSubsystem {
    pub threat: HashMap<ObjectGuid, f32>,
    pub attackers: HashSet<ObjectGuid>,
    pub attacking_guid: Option<ObjectGuid>,
    pub combat_disallowed: bool,
}

impl CombatSubsystem {
    pub fn add_threat(&mut self, target: ObjectGuid, amount: f32) -> f32 {
        let value = self.threat.entry(target).or_insert(0.0);
        *value += amount;
        *value
    }

    pub fn set_threat(&mut self, target: ObjectGuid, value: f32) {
        self.threat.insert(target, value);
    }

    pub fn threat_value(&self, target: ObjectGuid) -> Option<f32> {
        self.threat.get(&target).copied()
    }

    pub fn remove_threat(&mut self, target: ObjectGuid) -> Option<f32> {
        self.threat.remove(&target)
    }

    pub fn clear_threat(&mut self) {
        self.threat.clear();
    }

    pub fn is_threatened_by(&self, target: ObjectGuid) -> bool {
        self.threat.contains_key(&target)
    }

    pub fn add_attacker(&mut self, attacker: ObjectGuid) -> bool {
        self.attackers.insert(attacker)
    }

    pub fn remove_attacker(&mut self, attacker: ObjectGuid) -> bool {
        self.attackers.remove(&attacker)
    }

    pub fn clear_attackers(&mut self) {
        self.attackers.clear();
        self.attacking_guid = None;
    }

    pub fn set_attacking(&mut self, victim: Option<ObjectGuid>) {
        self.attacking_guid = victim;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MovementGeneratorKind {
    Idle,
    Random,
    Waypoint,
    Confused,
    Chase,
    Home,
    Flight,
    Point,
    Fleeing,
    Distract,
    Assistance,
    AssistanceDistract,
    TimedFleeing,
    Follow,
    Rotate,
    Effect,
    SplineChain,
    Formation,
    Custom(u32),
}

impl MovementGeneratorKind {
    pub const fn trinity_id(self) -> u8 {
        match self {
            Self::Idle => 0,
            Self::Random => 1,
            Self::Waypoint => 2,
            Self::Confused => 4,
            Self::Chase => 5,
            Self::Home => 6,
            Self::Flight => 7,
            Self::Point => 8,
            Self::Fleeing => 9,
            Self::Distract => 10,
            Self::Assistance => 11,
            Self::AssistanceDistract => 12,
            Self::TimedFleeing => 13,
            Self::Follow => 14,
            Self::Rotate => 15,
            Self::Effect => 16,
            Self::SplineChain => 17,
            Self::Formation => 18,
            Self::Custom(value) => value as u8,
        }
    }

    pub const fn from_trinity_id(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Idle),
            1 => Some(Self::Random),
            2 => Some(Self::Waypoint),
            3 | 19..=u8::MAX => None,
            4 => Some(Self::Confused),
            5 => Some(Self::Chase),
            6 => Some(Self::Home),
            7 => Some(Self::Flight),
            8 => Some(Self::Point),
            9 => Some(Self::Fleeing),
            10 => Some(Self::Distract),
            11 => Some(Self::Assistance),
            12 => Some(Self::AssistanceDistract),
            13 => Some(Self::TimedFleeing),
            14 => Some(Self::Follow),
            15 => Some(Self::Rotate),
            16 => Some(Self::Effect),
            17 => Some(Self::SplineChain),
            18 => Some(Self::Formation),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum MovementGeneratorMode {
    Default = 0,
    Override = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum MovementGeneratorPriority {
    None = 0,
    Normal = 1,
    Highest = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MovementSlot {
    Default = 0,
    Active = 1,
}

pub const MOVEMENTGENERATOR_FLAG_NONE: u16 = 0x000;
pub const MOVEMENTGENERATOR_FLAG_INITIALIZATION_PENDING: u16 = 0x001;
pub const MOVEMENTGENERATOR_FLAG_INITIALIZED: u16 = 0x002;
pub const MOVEMENTGENERATOR_FLAG_SPEED_UPDATE_PENDING: u16 = 0x004;
pub const MOVEMENTGENERATOR_FLAG_INTERRUPTED: u16 = 0x008;
pub const MOVEMENTGENERATOR_FLAG_PAUSED: u16 = 0x010;
pub const MOVEMENTGENERATOR_FLAG_TIMED_PAUSED: u16 = 0x020;
pub const MOVEMENTGENERATOR_FLAG_DEACTIVATED: u16 = 0x040;
pub const MOVEMENTGENERATOR_FLAG_INFORM_ENABLED: u16 = 0x080;
pub const MOVEMENTGENERATOR_FLAG_FINALIZED: u16 = 0x100;
pub const MOVEMENTGENERATOR_FLAG_PERSIST_ON_DEATH: u16 = 0x200;
pub const MOTIONMASTER_FLAG_INITIALIZATION_PENDING: u8 = 0x4;
pub const MOTIONMASTER_FLAG_STATIC_INITIALIZATION_PENDING: u8 = 0x2;
pub const MOTIONMASTER_FLAG_UPDATE: u8 = 0x1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MovementGeneratorRef {
    pub kind: MovementGeneratorKind,
    pub mode: MovementGeneratorMode,
    pub priority: MovementGeneratorPriority,
    pub slot: MovementSlot,
    pub flags: u16,
    pub base_unit_state: u32,
    pub target_guid: Option<ObjectGuid>,
    pub movement_id: u32,
    pub duration_ms: Option<u32>,
}

impl MovementGeneratorRef {
    pub const fn new(kind: MovementGeneratorKind, slot: MovementSlot) -> Self {
        Self {
            kind,
            mode: MovementGeneratorMode::Default,
            priority: MovementGeneratorPriority::None,
            slot,
            flags: MOVEMENTGENERATOR_FLAG_NONE,
            base_unit_state: 0,
            target_guid: None,
            movement_id: 0,
            duration_ms: None,
        }
    }

    pub const fn with_mode(mut self, mode: MovementGeneratorMode) -> Self {
        self.mode = mode;
        self
    }

    pub const fn with_priority(mut self, priority: MovementGeneratorPriority) -> Self {
        self.priority = priority;
        self
    }

    pub const fn with_flags(mut self, flags: u16) -> Self {
        self.flags = flags;
        self
    }

    pub const fn with_base_unit_state(mut self, base_unit_state: u32) -> Self {
        self.base_unit_state = base_unit_state;
        self
    }

    pub const fn with_target_guid(mut self, target_guid: ObjectGuid) -> Self {
        self.target_guid = Some(target_guid);
        self
    }

    pub const fn with_movement_id(mut self, movement_id: u32) -> Self {
        self.movement_id = movement_id;
        self
    }

    pub const fn with_duration_ms(mut self, duration_ms: u32) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    pub const fn has_flag(&self, flag: u16) -> bool {
        (self.flags & flag) != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveSplineState {
    pub enabled: bool,
    pub spline_id: u32,
    pub progress_ms: u32,
    pub duration_ms: u32,
}

impl Default for MoveSplineState {
    fn default() -> Self {
        Self {
            enabled: false,
            spline_id: 0,
            progress_ms: 0,
            duration_ms: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MotionSubsystem {
    pub default_generator: MovementGeneratorRef,
    pub active_generators: Vec<MovementGeneratorRef>,
    pub current_generator: MovementGeneratorKind,
    pub base_unit_states: HashMap<u32, usize>,
    pub flags: u8,
    pub delayed_actions: Vec<u8>,
    pub paused: bool,
    pub stopped: bool,
    pub spline: MoveSplineState,
}

impl Default for MotionSubsystem {
    fn default() -> Self {
        let default_generator =
            MovementGeneratorRef::new(MovementGeneratorKind::Idle, MovementSlot::Default);
        Self {
            default_generator,
            active_generators: Vec::new(),
            current_generator: MovementGeneratorKind::Idle,
            base_unit_states: HashMap::new(),
            flags: MOTIONMASTER_FLAG_INITIALIZATION_PENDING,
            delayed_actions: Vec::new(),
            paused: false,
            stopped: false,
            spline: MoveSplineState::default(),
        }
    }
}

impl MotionSubsystem {
    pub fn set_current_generator(&mut self, generator: MovementGeneratorKind) {
        self.add_generator(MovementGeneratorRef::new(generator, MovementSlot::Active));
    }

    pub fn add_to_world(&mut self) {
        self.flags &= !MOTIONMASTER_FLAG_INITIALIZATION_PENDING;
        self.current_generator = self.current_movement_generator().kind;
    }

    pub fn is_empty(&self) -> bool {
        self.active_generators.is_empty()
            && self.default_generator.kind == MovementGeneratorKind::Custom(u32::MAX)
    }

    pub fn size(&self) -> usize {
        1 + self.active_generators.len()
    }

    pub fn current_slot(&self) -> MovementSlot {
        if self.active_generators.is_empty() {
            MovementSlot::Default
        } else {
            MovementSlot::Active
        }
    }

    pub fn current_movement_generator(&self) -> MovementGeneratorRef {
        self.active_generators
            .first()
            .copied()
            .unwrap_or(self.default_generator)
    }

    pub fn add_generator(&mut self, mut generator: MovementGeneratorRef) {
        match generator.slot {
            MovementSlot::Default => {
                generator.slot = MovementSlot::Default;
                self.default_generator = generator;
                if generator.kind == MovementGeneratorKind::Idle {
                    self.flags |= MOTIONMASTER_FLAG_STATIC_INITIALIZATION_PENDING;
                }
            }
            MovementSlot::Active => {
                generator.slot = MovementSlot::Active;
                if let Some(top) = self.active_generators.first().copied() {
                    if generator.priority >= top.priority {
                        if generator.priority == top.priority {
                            self.remove_generator_at(0);
                        } else if let Some(top) = self.active_generators.first_mut() {
                            top.flags |= MOVEMENTGENERATOR_FLAG_DEACTIVATED;
                        }
                    } else if let Some(index) = self
                        .active_generators
                        .iter()
                        .position(|known| known.priority == generator.priority)
                    {
                        self.remove_generator_at(index);
                    }
                }

                self.add_base_unit_state(generator.base_unit_state);
                self.active_generators.push(generator);
                self.sort_active_generators();
            }
        }
        self.current_generator = self.current_movement_generator().kind;
        self.stopped = false;
    }

    pub fn remove_generator_kind(
        &mut self,
        kind: MovementGeneratorKind,
        slot: MovementSlot,
    ) -> Option<MovementGeneratorRef> {
        let removed = match slot {
            MovementSlot::Default if self.default_generator.kind == kind => {
                let previous = self.default_generator;
                self.move_idle();
                Some(previous)
            }
            MovementSlot::Default => None,
            MovementSlot::Active => self
                .active_generators
                .iter()
                .position(|generator| generator.kind == kind)
                .map(|index| self.remove_generator_at(index)),
        };
        self.current_generator = self.current_movement_generator().kind;
        removed
    }

    pub fn clear_active(&mut self) -> Vec<MovementGeneratorRef> {
        let removed = std::mem::take(&mut self.active_generators);
        self.base_unit_states.clear();
        self.current_generator = self.default_generator.kind;
        removed
    }

    pub fn clear_slot(&mut self, slot: MovementSlot) -> Vec<MovementGeneratorRef> {
        match slot {
            MovementSlot::Default => {
                let previous = self.default_generator;
                self.move_idle();
                vec![previous]
            }
            MovementSlot::Active => self.clear_active(),
        }
    }

    pub fn clear_by_priority(
        &mut self,
        priority: MovementGeneratorPriority,
    ) -> Vec<MovementGeneratorRef> {
        let mut removed = Vec::new();
        let mut index = 0;
        while index < self.active_generators.len() {
            if self.active_generators[index].priority == priority {
                removed.push(self.remove_generator_at(index));
            } else {
                index += 1;
            }
        }
        self.current_generator = self.current_movement_generator().kind;
        removed
    }

    pub fn move_idle(&mut self) {
        self.default_generator =
            MovementGeneratorRef::new(MovementGeneratorKind::Idle, MovementSlot::Default);
        self.flags |= MOTIONMASTER_FLAG_STATIC_INITIALIZATION_PENDING;
        if self.active_generators.is_empty() {
            self.current_generator = MovementGeneratorKind::Idle;
        }
    }

    pub fn move_point(&mut self, movement_id: u32) {
        self.add_generator(
            MovementGeneratorRef::new(MovementGeneratorKind::Point, MovementSlot::Active)
                .with_priority(MovementGeneratorPriority::Normal)
                .with_movement_id(movement_id),
        );
    }

    pub fn move_charge(&mut self, movement_id: u32) {
        self.add_generator(
            MovementGeneratorRef::new(MovementGeneratorKind::Point, MovementSlot::Active)
                .with_priority(MovementGeneratorPriority::Highest)
                .with_base_unit_state(UnitState::CHARGING.bits())
                .with_movement_id(movement_id),
        );
    }

    pub fn move_follow(&mut self, target_guid: ObjectGuid, duration_ms: Option<u32>) {
        let mut generator =
            MovementGeneratorRef::new(MovementGeneratorKind::Follow, MovementSlot::Active)
                .with_priority(MovementGeneratorPriority::Normal)
                .with_target_guid(target_guid);
        if let Some(duration_ms) = duration_ms {
            generator = generator.with_duration_ms(duration_ms);
        }
        self.add_generator(generator);
    }

    pub fn stop_on_death(&mut self) -> bool {
        if self
            .active_generators
            .first()
            .is_some_and(|generator| generator.has_flag(MOVEMENTGENERATOR_FLAG_PERSIST_ON_DEATH))
        {
            return false;
        }

        self.clear_active();
        self.move_idle();
        self.stop_moving();
        true
    }

    pub fn pause_movement(&mut self) {
        self.paused = true;
    }

    pub fn resume_movement(&mut self) {
        self.paused = false;
    }

    pub fn stop_moving(&mut self) {
        self.stopped = true;
        self.spline.enabled = false;
        self.spline.progress_ms = 0;
    }

    pub fn start_spline(&mut self, spline_id: u32, duration_ms: u32) {
        self.spline = MoveSplineState {
            enabled: true,
            spline_id,
            progress_ms: 0,
            duration_ms,
        };
        self.stopped = false;
    }

    pub fn set_spline_progress(&mut self, progress_ms: u32) {
        self.spline.progress_ms = progress_ms.min(self.spline.duration_ms);
    }

    fn sort_active_generators(&mut self) {
        self.active_generators.sort_by(|left, right| {
            right
                .mode
                .cmp(&left.mode)
                .then_with(|| right.priority.cmp(&left.priority))
        });
    }

    fn remove_generator_at(&mut self, index: usize) -> MovementGeneratorRef {
        let removed = self.active_generators.remove(index);
        self.clear_base_unit_state(removed.base_unit_state);
        removed
    }

    fn add_base_unit_state(&mut self, base_unit_state: u32) {
        if base_unit_state != 0 {
            *self.base_unit_states.entry(base_unit_state).or_insert(0) += 1;
        }
    }

    fn clear_base_unit_state(&mut self, base_unit_state: u32) {
        if base_unit_state == 0 {
            return;
        }
        if let Some(count) = self.base_unit_states.get_mut(&base_unit_state) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.base_unit_states.remove(&base_unit_state);
            }
        }
    }
}

pub const SUMMON_SLOT_PET: usize = 0;
pub const SUMMON_SLOT_TOTEM: usize = 1;
pub const SUMMON_SLOT_TOTEM_2: usize = 2;
pub const SUMMON_SLOT_TOTEM_3: usize = 3;
pub const SUMMON_SLOT_TOTEM_4: usize = 4;
pub const SUMMON_SLOT_MINIPET: usize = 5;
pub const SUMMON_SLOT_QUEST: usize = 6;
pub const MAX_SUMMON_SLOT: usize = 7;
pub const MAX_TOTEM_SLOT: usize = 5;
pub const MAX_UNIT_ACTION_BAR_INDEX: usize = 10;
pub const MAX_SPELL_CHARM: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CharmType {
    Charm = 0,
    Possess = 1,
    Vehicle = 2,
    Convert = 3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CharmInfoState {
    pub pet_number: u32,
    pub command_state: u8,
    pub action_bar: [u32; MAX_UNIT_ACTION_BAR_INDEX],
    pub charm_spells: [u32; MAX_SPELL_CHARM],
    pub is_command_attack: bool,
    pub is_command_follow: bool,
    pub is_at_stay: bool,
    pub is_following: bool,
    pub is_returning: bool,
    pub stay_position: Option<(f32, f32, f32)>,
}

impl Default for CharmInfoState {
    fn default() -> Self {
        Self {
            pet_number: 0,
            command_state: 0,
            action_bar: [0; MAX_UNIT_ACTION_BAR_INDEX],
            charm_spells: [0; MAX_SPELL_CHARM],
            is_command_attack: false,
            is_command_follow: false,
            is_at_stay: false,
            is_following: false,
            is_returning: false,
            stay_position: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ControlSubsystem {
    pub owner_guid: Option<ObjectGuid>,
    pub minion_guid: Option<ObjectGuid>,
    pub summon_slots: [ObjectGuid; MAX_SUMMON_SLOT],
    pub last_charmer_guid: Option<ObjectGuid>,
    pub charmer_guid: Option<ObjectGuid>,
    pub charmed_guid: Option<ObjectGuid>,
    pub controlled_guids: HashSet<ObjectGuid>,
    pub controlled_by_player: bool,
    pub charm_type: Option<CharmType>,
    pub unit_moved_by_me: Option<ObjectGuid>,
    pub player_moving_me: Option<ObjectGuid>,
    pub shared_vision_guids: HashSet<ObjectGuid>,
    pub charm_info: Option<CharmInfoState>,
    pub old_faction_id: Option<u32>,
    pub walking_before_charm: bool,
}

impl Default for ControlSubsystem {
    fn default() -> Self {
        Self {
            owner_guid: None,
            minion_guid: None,
            summon_slots: [ObjectGuid::EMPTY; MAX_SUMMON_SLOT],
            last_charmer_guid: None,
            charmer_guid: None,
            charmed_guid: None,
            controlled_guids: HashSet::new(),
            controlled_by_player: false,
            charm_type: None,
            unit_moved_by_me: None,
            player_moving_me: None,
            shared_vision_guids: HashSet::new(),
            charm_info: None,
            old_faction_id: None,
            walking_before_charm: false,
        }
    }
}

impl ControlSubsystem {
    pub fn set_owner_guid(&mut self, owner: Option<ObjectGuid>) {
        self.owner_guid = owner;
    }

    pub fn set_minion_guid(&mut self, minion: Option<ObjectGuid>) {
        self.minion_guid = minion;
    }

    pub fn pet_guid(&self) -> ObjectGuid {
        self.summon_slots[SUMMON_SLOT_PET]
    }

    pub fn set_pet_guid(&mut self, pet: ObjectGuid) {
        self.summon_slots[SUMMON_SLOT_PET] = pet;
    }

    pub fn set_summon_slot(&mut self, slot: usize, guid: ObjectGuid) -> bool {
        let Some(target) = self.summon_slots.get_mut(slot) else {
            return false;
        };
        *target = guid;
        true
    }

    pub fn clear_summon_slot(&mut self, slot: usize) -> Option<ObjectGuid> {
        let target = self.summon_slots.get_mut(slot)?;
        let previous = *target;
        *target = ObjectGuid::EMPTY;
        Some(previous)
    }

    pub fn set_charmer(&mut self, charmer: ObjectGuid, controlled_by_player: bool) {
        self.last_charmer_guid = self.charmer_guid;
        self.charmer_guid = Some(charmer);
        self.controlled_by_player = controlled_by_player;
        self.init_charm_info();
    }

    pub fn remove_charmer(&mut self) {
        self.last_charmer_guid = self.charmer_guid;
        self.charmer_guid = None;
        self.controlled_by_player = false;
        self.charm_type = None;
        self.old_faction_id = None;
        self.delete_charm_info();
    }

    pub fn set_charmed(&mut self, charmed: ObjectGuid) {
        self.charmed_guid = Some(charmed);
        self.controlled_guids.insert(charmed);
    }

    pub fn remove_charmed(&mut self) {
        if let Some(charmed) = self.charmed_guid.take() {
            self.controlled_guids.remove(&charmed);
        }
    }

    pub fn apply_charm_as_controller(&mut self, charmed: ObjectGuid, controller_is_player: bool) {
        if controller_is_player {
            self.charmed_guid = Some(charmed);
        }
        self.controlled_guids.insert(charmed);
    }

    pub fn remove_charm_as_controller(
        &mut self,
        charmed: ObjectGuid,
        controlled_has_same_owner: bool,
        controlled_is_minion: bool,
        controlled_is_player: bool,
    ) {
        if self.charmed_guid == Some(charmed) {
            self.charmed_guid = None;
        }
        if controlled_is_player || !controlled_is_minion || !controlled_has_same_owner {
            self.controlled_guids.remove(&charmed);
        }
    }

    pub fn apply_charmed_by(
        &mut self,
        charmer: ObjectGuid,
        charm_type: CharmType,
        controlled_by_player: bool,
        old_faction_id: Option<u32>,
        was_walking: bool,
    ) -> bool {
        if self.charmer_guid.is_some() {
            return false;
        }
        self.charmer_guid = Some(charmer);
        self.controlled_by_player = controlled_by_player;
        self.charm_type = Some(charm_type);
        self.old_faction_id = old_faction_id;
        self.walking_before_charm = was_walking;
        if charm_type != CharmType::Vehicle {
            self.init_charm_info();
        }
        true
    }

    pub fn remove_charmed_by(
        &mut self,
        expected_charmer: Option<ObjectGuid>,
        is_guardian: bool,
    ) -> bool {
        let Some(charmer) = self.charmer_guid else {
            return false;
        };
        if expected_charmer.is_some_and(|expected| expected != charmer) {
            return false;
        }
        if self.charm_type != Some(CharmType::Vehicle) {
            self.last_charmer_guid = Some(charmer);
        }
        self.charmer_guid = None;
        self.controlled_by_player = false;
        self.charm_type = None;
        self.old_faction_id = None;
        if !is_guardian {
            self.delete_charm_info();
        }
        true
    }

    pub fn add_controlled(&mut self, guid: ObjectGuid) -> bool {
        self.controlled_guids.insert(guid)
    }

    pub fn remove_controlled(&mut self, guid: ObjectGuid) -> bool {
        if self.charmed_guid == Some(guid) {
            self.charmed_guid = None;
        }
        self.controlled_guids.remove(&guid)
    }

    pub fn clear_controlled(&mut self) {
        self.controlled_guids.clear();
        self.charmed_guid = None;
    }

    pub fn is_charmed(&self) -> bool {
        self.charmer_guid.is_some()
    }

    pub fn is_possessed(&self) -> bool {
        self.charm_type == Some(CharmType::Possess)
    }

    pub fn is_possessed_by_player(&self) -> bool {
        self.is_possessed() && self.controlled_by_player
    }

    pub fn is_possessing(&self) -> bool {
        self.charmed_guid.is_some()
    }

    pub fn is_possessing_guid(&self, guid: ObjectGuid) -> bool {
        self.charmed_guid == Some(guid)
    }

    pub fn charmer_or_owner_guid(&self) -> Option<ObjectGuid> {
        self.charmer_guid.or(self.owner_guid)
    }

    pub fn charmer_or_owner_or_self_guid(&self, own_guid: ObjectGuid) -> ObjectGuid {
        self.charmer_or_owner_guid().unwrap_or(own_guid)
    }

    pub fn init_charm_info(&mut self) -> &mut CharmInfoState {
        self.charm_info.get_or_insert_with(CharmInfoState::default)
    }

    pub fn delete_charm_info(&mut self) {
        self.charm_info = None;
    }

    pub fn has_charm_info(&self) -> bool {
        self.charm_info.is_some()
    }

    pub fn remove_all_controlled(&mut self) -> Vec<ObjectGuid> {
        let removed = self.controlled_guids.drain().collect();
        self.charmed_guid = None;
        removed
    }

    pub fn set_moved_unit(&mut self, target: Option<ObjectGuid>) {
        self.unit_moved_by_me = target;
    }

    pub fn set_player_moving_me(&mut self, player: Option<ObjectGuid>) {
        self.player_moving_me = player;
    }

    pub fn add_shared_vision(&mut self, guid: ObjectGuid) -> bool {
        self.shared_vision_guids.insert(guid)
    }

    pub fn remove_shared_vision(&mut self, guid: ObjectGuid) -> bool {
        self.shared_vision_guids.remove(&guid)
    }

    pub fn has_shared_vision(&self) -> bool {
        !self.shared_vision_guids.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VehicleKitState {
    pub kit_id: u32,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VehicleSubsystem {
    pub vehicle_guid: Option<ObjectGuid>,
    pub base_vehicle_guid: Option<ObjectGuid>,
    pub seat_id: Option<i8>,
    pub kit: Option<VehicleKitState>,
}

impl VehicleSubsystem {
    pub fn enter_vehicle(&mut self, vehicle_guid: ObjectGuid, seat_id: Option<i8>) {
        self.vehicle_guid = Some(vehicle_guid);
        self.seat_id = seat_id;
    }

    pub fn exit_vehicle(&mut self) {
        self.vehicle_guid = None;
        self.seat_id = None;
    }

    pub fn set_vehicle_kit(&mut self, kit_id: u32, active: bool) {
        self.kit = Some(VehicleKitState { kit_id, active });
    }

    pub fn clear_vehicle_kit(&mut self) {
        self.kit = None;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AiSubsystem {
    pub active_ai: Option<String>,
    pub ai_stack: Vec<String>,
    pub locked: bool,
}

impl AiSubsystem {
    pub fn set_active(&mut self, ai: Option<impl Into<String>>) {
        self.active_ai = ai.map(Into::into);
    }

    pub fn push(&mut self, ai: impl Into<String>) {
        if let Some(active) = self.active_ai.take() {
            self.ai_stack.push(active);
        }
        self.active_ai = Some(ai.into());
    }

    pub fn pop(&mut self) -> Option<String> {
        let popped = self.active_ai.take();
        self.active_ai = self.ai_stack.pop();
        popped
    }

    pub fn set_locked(&mut self, locked: bool) {
        self.locked = locked;
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct UnitSubsystems {
    pub auras: AuraSubsystem,
    pub spells: SpellSubsystem,
    pub combat: CombatSubsystem,
    pub motion: MotionSubsystem,
    pub control: ControlSubsystem,
    pub vehicle: VehicleSubsystem,
    pub ai: AiSubsystem,
}

impl UnitSubsystems {
    pub fn clear_runtime_state(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod unit_subsystems_tests {
    use super::*;

    fn guid(low: i64) -> ObjectGuid {
        ObjectGuid::new(0, low)
    }

    #[test]
    fn aura_spell_history_and_current_spell_helpers_roundtrip() {
        let mut subsystems = UnitSubsystems::default();
        let caster = guid(1);
        let owned = OwnedAuraRef::new(100, caster, None);
        let applied = AppliedAuraRef::new(100, caster, 2, 0x5);

        subsystems.auras.add_owned(owned);
        subsystems.auras.add_applied(applied);
        subsystems.auras.set_visible(2, AuraRef::new(100, caster));
        subsystems.auras.mark_removed(AuraRef::new(100, caster));
        subsystems.auras.interrupt_flags = 0x10;
        subsystems.auras.interrupt_flags2 = 0x20;

        assert!(subsystems.auras.has_owned(owned));
        assert!(subsystems.auras.has_applied(applied));
        assert_eq!(
            subsystems.auras.visible_auras.get(&2).copied(),
            Some(AuraRef::new(100, caster))
        );
        assert_eq!(subsystems.auras.removed_count(), 1);
        assert!(subsystems.auras.remove_owned(owned));
        assert!(subsystems.auras.remove_applied(applied));
        assert_eq!(
            subsystems.auras.clear_visible(2),
            Some(AuraRef::new(100, caster))
        );
        subsystems.auras.clear_removed();
        assert_eq!(subsystems.auras.removed_count(), 0);

        let spell = CurrentSpellRef::new(200, Some(caster), Some(guid(3)));
        subsystems
            .spells
            .set_current_spell(CurrentSpellSlot::Generic, spell);
        assert_eq!(
            subsystems.spells.current_spell(CurrentSpellSlot::Generic),
            Some(spell)
        );
        assert_eq!(
            subsystems
                .spells
                .clear_current_spell(CurrentSpellSlot::Generic),
            Some(spell)
        );

        subsystems.spells.history.set_cooldown(200, 1_000, 30_000);
        subsystems.spells.history.set_charges(200, 2, 1_000, 10_000);
        assert_eq!(
            subsystems.spells.history.cooldown(200),
            Some(SpellCooldown {
                spell_id: 200,
                item_id: 0,
                cooldown_end_ms: 31_000,
                category_id: 0,
                category_end_ms: 1_000,
                on_hold: false,
            })
        );
        assert_eq!(
            subsystems.spells.history.charges(200).map(VecDeque::len),
            Some(2)
        );
        assert!(subsystems.spells.history.clear_cooldown(200));
        subsystems.spells.history.reset();
        assert!(subsystems.spells.history.cooldowns.is_empty());
        assert!(subsystems.spells.history.charges.is_empty());
    }

    #[test]
    fn spell_history_cooldowns_track_spell_category_hold_and_update_like_cpp() {
        let mut history = SpellHistory::default();

        assert!(history.start_cooldown(1_000, 100, 7, 3_000, 9, 1_500, false));
        assert!(history.has_cooldown(100, 9, 2_000));
        assert_eq!(history.remaining_cooldown_ms(100, 9, 2_000), 2_000);
        assert_eq!(history.remaining_category_cooldown_ms(9, 2_000), 500);

        assert!(!history.add_cooldown(100, 7, 2_000, 9, 1_500, false));
        assert_eq!(
            history
                .cooldown(100)
                .map(|cooldown| cooldown.cooldown_end_ms),
            Some(4_000)
        );

        assert!(history.start_cooldown(2_000, 101, 0, 1, 11, 1, true));
        let held = history.cooldown(101).expect("on-hold cooldown");
        assert!(held.on_hold);
        assert_eq!(held.cooldown_end_ms, 2_000 + INFINITY_COOLDOWN_DELAY_MS);
        assert_eq!(held.category_end_ms, 2_000 + INFINITY_COOLDOWN_DELAY_MS);

        assert!(history.modify_cooldown(100, -2_000, false, 2_500));
        assert_eq!(history.cooldown(100), None);
        assert!(!history.has_cooldown(100, 9, 2_500));

        history.update(2_501);
        assert!(!history.has_cooldown(100, 9, 2_501));
        assert!(history.has_cooldown(101, 11, 2_501));
    }

    #[test]
    fn spell_history_charges_school_locks_gcd_and_duel_snapshot_match_cpp_shape() {
        let mut history = SpellHistory::default();

        assert!(history.consume_charge(44, 1_000, 5_000, 2));
        assert!(history.consume_charge(44, 1_500, 5_000, 2));
        assert!(!history.has_charge(44, 2));
        assert_eq!(history.consumed_charges(44), 2);
        assert_eq!(
            history
                .charges(44)
                .and_then(|charges| charges.front())
                .map(|charge| charge.recharge_end_ms),
            Some(6_000)
        );

        assert!(history.modify_charge_recovery_time(44, -1_000, 1_500));
        assert_eq!(
            history
                .charges(44)
                .and_then(|charges| charges.front())
                .map(|charge| charge.recharge_end_ms),
            Some(5_000)
        );
        assert!(history.restore_charge(44));
        assert_eq!(history.consumed_charges(44), 1);
        history.update(5_000);
        assert_eq!(history.consumed_charges(44), 0);

        history.lock_spell_school(0b0010_1000, 10_000, 3_000);
        assert!(history.is_school_locked(0b0000_1000, 12_000));
        assert!(history.is_school_locked(0b0010_0000, 12_000));
        assert!(!history.is_school_locked(0b0000_1000, 13_001));

        history.add_global_cooldown(12, 20_000, 1_500);
        assert!(history.has_global_cooldown(12, 21_000));
        assert_eq!(history.remaining_global_cooldown_ms(12, 21_000), 500);
        history.cancel_global_cooldown(12);
        assert!(!history.has_global_cooldown(12, 21_000));

        history.start_cooldown(30_000, 777, 0, 10_000, 55, 5_000, false);
        history.save_cooldown_state_before_duel();
        history.start_cooldown(31_000, 888, 0, 10_000, 66, 5_000, false);
        history.restore_cooldown_state_after_duel();
        assert!(history.has_cooldown(777, 55, 31_000));
        assert!(!history.has_cooldown(888, 66, 31_000));
        assert_eq!(history.category_cooldowns.get(&55), Some(&777));
    }

    #[test]
    fn current_spell_slots_match_trinity_values_and_roundtrip() {
        assert_eq!(CurrentSpellSlot::Melee as u8, 0);
        assert_eq!(CurrentSpellSlot::Generic as u8, 1);
        assert_eq!(CurrentSpellSlot::Channeled as u8, 2);
        assert_eq!(CurrentSpellSlot::Autorepeat as u8, 3);
        assert_eq!(CURRENT_FIRST_NON_MELEE_SPELL, 1);
        assert_eq!(CURRENT_MAX_SPELL, 4);

        let caster = guid(4);
        let mut spells = SpellSubsystem::default();
        let slots = [
            CurrentSpellSlot::Melee,
            CurrentSpellSlot::Generic,
            CurrentSpellSlot::Channeled,
            CurrentSpellSlot::Autorepeat,
        ];

        for (index, slot) in slots.into_iter().enumerate() {
            let spell = CurrentSpellRef::new(300 + index as u32, Some(caster), None);
            spells.set_current_spell(slot, spell);
            assert_eq!(spells.current_spell(slot), Some(spell));
            assert_eq!(spells.clear_current_spell(slot), Some(spell));
            assert_eq!(spells.current_spell(slot), None);
        }
    }

    #[test]
    fn threat_combat_helpers_roundtrip() {
        let mut combat = CombatSubsystem::default();
        let attacker = guid(10);

        assert!(!combat.combat_disallowed);
        assert_eq!(combat.add_threat(attacker, 5.0), 5.0);
        assert_eq!(combat.add_threat(attacker, 2.5), 7.5);
        assert!(combat.is_threatened_by(attacker));
        assert_eq!(combat.threat_value(attacker), Some(7.5));
        combat.set_threat(attacker, 1.0);
        assert_eq!(combat.remove_threat(attacker), Some(1.0));

        assert!(combat.add_attacker(attacker));
        combat.set_attacking(Some(attacker));
        combat.combat_disallowed = true;
        assert!(combat.attackers.contains(&attacker));
        assert_eq!(combat.attacking_guid, Some(attacker));
        assert!(combat.combat_disallowed);
        assert!(combat.remove_attacker(attacker));
        combat.clear_attackers();
        assert!(combat.attackers.is_empty());
        assert_eq!(combat.attacking_guid, None);
    }

    #[test]
    fn motion_generator_ids_slots_and_priorities_match_cpp_motion_master_shape() {
        assert_eq!(MovementGeneratorKind::Idle.trinity_id(), 0);
        assert_eq!(MovementGeneratorKind::Random.trinity_id(), 1);
        assert_eq!(MovementGeneratorKind::Waypoint.trinity_id(), 2);
        assert_eq!(MovementGeneratorKind::from_trinity_id(3), None);
        assert_eq!(
            MovementGeneratorKind::from_trinity_id(14),
            Some(MovementGeneratorKind::Follow)
        );
        assert_eq!(
            MovementGeneratorKind::from_trinity_id(18),
            Some(MovementGeneratorKind::Formation)
        );
        assert_eq!(MovementSlot::Default as u8, 0);
        assert_eq!(MovementSlot::Active as u8, 1);

        let mut motion = MotionSubsystem::default();
        motion.add_to_world();
        assert_eq!(motion.size(), 1);
        assert_eq!(motion.current_slot(), MovementSlot::Default);
        assert_eq!(
            motion.current_movement_generator().kind,
            MovementGeneratorKind::Idle
        );

        motion.add_generator(
            MovementGeneratorRef::new(MovementGeneratorKind::Follow, MovementSlot::Active)
                .with_priority(MovementGeneratorPriority::Normal)
                .with_target_guid(guid(30)),
        );
        assert_eq!(motion.current_slot(), MovementSlot::Active);
        assert_eq!(
            motion.current_movement_generator().kind,
            MovementGeneratorKind::Follow
        );

        motion.move_charge(42);
        let current = motion.current_movement_generator();
        assert_eq!(current.kind, MovementGeneratorKind::Point);
        assert_eq!(current.priority, MovementGeneratorPriority::Highest);
        assert_eq!(current.base_unit_state, UnitState::CHARGING.bits());
        assert_eq!(
            motion.base_unit_states.get(&UnitState::CHARGING.bits()),
            Some(&1)
        );
        assert!(
            motion
                .active_generators
                .iter()
                .any(|generator| generator.kind == MovementGeneratorKind::Follow
                    && generator.has_flag(MOVEMENTGENERATOR_FLAG_DEACTIVATED))
        );

        let removed = motion.clear_by_priority(MovementGeneratorPriority::Highest);
        assert_eq!(removed.len(), 1);
        assert_eq!(
            motion.base_unit_states.get(&UnitState::CHARGING.bits()),
            None
        );
        assert_eq!(
            motion.current_movement_generator().kind,
            MovementGeneratorKind::Follow
        );
    }

    #[test]
    fn motion_stop_on_death_preserves_persistent_generators_like_cpp() {
        let mut motion = MotionSubsystem::default();
        motion.add_generator(
            MovementGeneratorRef::new(MovementGeneratorKind::Effect, MovementSlot::Active)
                .with_priority(MovementGeneratorPriority::Highest)
                .with_flags(MOVEMENTGENERATOR_FLAG_PERSIST_ON_DEATH),
        );

        assert!(!motion.stop_on_death());
        assert_eq!(
            motion.current_movement_generator().kind,
            MovementGeneratorKind::Effect
        );

        motion.clear_active();
        motion.move_point(9);
        motion.start_spline(7, 1_000);
        assert!(motion.stop_on_death());
        assert_eq!(motion.current_slot(), MovementSlot::Default);
        assert_eq!(
            motion.current_movement_generator().kind,
            MovementGeneratorKind::Idle
        );
        assert!(motion.stopped);
        assert!(!motion.spline.enabled);
    }

    #[test]
    fn control_summon_slots_match_cpp_shared_defines() {
        assert_eq!(SUMMON_SLOT_PET, 0);
        assert_eq!(SUMMON_SLOT_TOTEM, 1);
        assert_eq!(SUMMON_SLOT_TOTEM_2, 2);
        assert_eq!(SUMMON_SLOT_TOTEM_3, 3);
        assert_eq!(SUMMON_SLOT_TOTEM_4, 4);
        assert_eq!(SUMMON_SLOT_MINIPET, 5);
        assert_eq!(SUMMON_SLOT_QUEST, 6);
        assert_eq!(MAX_SUMMON_SLOT, 7);
        assert_eq!(MAX_TOTEM_SLOT, 5);

        let mut control = ControlSubsystem::default();
        let pet = guid(40);
        let totem = guid(41);

        assert_eq!(control.pet_guid(), ObjectGuid::EMPTY);
        control.set_pet_guid(pet);
        assert_eq!(control.pet_guid(), pet);
        assert!(control.set_summon_slot(SUMMON_SLOT_TOTEM_3, totem));
        assert_eq!(control.summon_slots[SUMMON_SLOT_TOTEM_3], totem);
        assert!(!control.set_summon_slot(MAX_SUMMON_SLOT, guid(42)));
        assert_eq!(control.clear_summon_slot(SUMMON_SLOT_TOTEM_3), Some(totem));
        assert_eq!(control.summon_slots[SUMMON_SLOT_TOTEM_3], ObjectGuid::EMPTY);
    }

    #[test]
    fn control_charm_controller_and_target_state_follow_cpp_set_charm() {
        let mut controller = ControlSubsystem::default();
        let mut target = ControlSubsystem::default();
        let controller_guid = guid(50);
        let target_guid = guid(51);
        let other_guid = guid(52);

        controller.apply_charm_as_controller(target_guid, true);
        assert_eq!(controller.charmed_guid, Some(target_guid));
        assert!(controller.controlled_guids.contains(&target_guid));
        assert!(!controller.has_charm_info());

        assert!(target.apply_charmed_by(
            controller_guid,
            CharmType::Possess,
            true,
            Some(123),
            true,
        ));
        assert_eq!(target.charmer_guid, Some(controller_guid));
        assert_eq!(target.charm_type, Some(CharmType::Possess));
        assert_eq!(target.old_faction_id, Some(123));
        assert!(target.walking_before_charm);
        assert!(target.is_charmed());
        assert!(target.is_possessed_by_player());
        assert!(target.has_charm_info());
        assert!(!target.apply_charmed_by(other_guid, CharmType::Charm, false, None, false,));

        assert!(!target.remove_charmed_by(Some(other_guid), false));
        assert!(target.remove_charmed_by(Some(controller_guid), false));
        assert_eq!(target.charmer_guid, None);
        assert_eq!(target.last_charmer_guid, Some(controller_guid));
        assert_eq!(target.charm_type, None);
        assert_eq!(target.old_faction_id, None);
        assert!(!target.has_charm_info());

        controller.remove_charm_as_controller(target_guid, false, true, false);
        assert_eq!(controller.charmed_guid, None);
        assert!(!controller.controlled_guids.contains(&target_guid));
    }

    #[test]
    fn control_remove_charm_preserves_owned_minions_like_cpp() {
        let mut controller = ControlSubsystem::default();
        let minion = guid(60);

        controller.apply_charm_as_controller(minion, false);
        controller.remove_charm_as_controller(minion, true, true, false);
        assert!(controller.controlled_guids.contains(&minion));

        controller.remove_charm_as_controller(minion, true, false, false);
        assert!(!controller.controlled_guids.contains(&minion));
    }

    #[test]
    fn charm_info_direct_control_and_shared_vision_helpers_roundtrip() {
        let mut control = ControlSubsystem::default();
        let controller = guid(70);
        let controlled = guid(71);
        let observer = guid(72);

        control.set_owner_guid(Some(controller));
        assert_eq!(control.charmer_or_owner_guid(), Some(controller));
        assert_eq!(
            control.charmer_or_owner_or_self_guid(controlled),
            controller
        );

        let charm_info = control.init_charm_info();
        charm_info.pet_number = 9;
        charm_info.command_state = 2;
        charm_info.action_bar[0] = 100;
        charm_info.charm_spells[0] = 200;
        charm_info.is_command_follow = true;
        charm_info.stay_position = Some((1.0, 2.0, 3.0));
        assert!(control.has_charm_info());
        assert_eq!(
            control.charm_info.as_ref().map(|info| info.pet_number),
            Some(9)
        );

        control.add_controlled(controlled);
        control.set_charmed(controlled);
        control.set_moved_unit(Some(controlled));
        control.set_player_moving_me(Some(controller));
        assert!(control.is_possessing_guid(controlled));
        assert_eq!(control.unit_moved_by_me, Some(controlled));
        assert_eq!(control.player_moving_me, Some(controller));

        assert!(control.add_shared_vision(observer));
        assert!(control.has_shared_vision());
        assert!(control.remove_shared_vision(observer));
        assert!(!control.has_shared_vision());

        let removed = control.remove_all_controlled();
        assert_eq!(removed, vec![controlled]);
        assert_eq!(control.charmed_guid, None);
        control.delete_charm_info();
        assert!(!control.has_charm_info());
    }

    #[test]
    fn motion_charm_vehicle_and_ai_helpers_roundtrip() {
        let mut subsystems = UnitSubsystems::default();
        let controller = guid(20);
        let controlled = guid(21);
        let vehicle = guid(30);

        subsystems
            .motion
            .set_current_generator(MovementGeneratorKind::Chase);
        subsystems.motion.start_spline(7, 1_000);
        subsystems.motion.set_spline_progress(1_500);
        assert_eq!(
            subsystems.motion.current_generator,
            MovementGeneratorKind::Chase
        );
        assert_eq!(subsystems.motion.spline.progress_ms, 1_000);
        subsystems.motion.pause_movement();
        assert!(subsystems.motion.paused);
        subsystems.motion.resume_movement();
        subsystems.motion.stop_moving();
        assert!(!subsystems.motion.paused);
        assert!(subsystems.motion.stopped);
        assert!(!subsystems.motion.spline.enabled);

        subsystems.control.set_charmer(controller, true);
        subsystems.control.set_charmed(controlled);
        subsystems.control.unit_moved_by_me = Some(controlled);
        subsystems.control.player_moving_me = Some(controller);
        assert!(subsystems.control.is_charmed());
        assert!(subsystems.control.controlled_by_player);
        assert!(subsystems.control.controlled_guids.contains(&controlled));
        assert!(subsystems.control.add_shared_vision(controlled));
        subsystems.control.remove_charmed();
        subsystems.control.remove_charmer();
        assert!(!subsystems.control.is_charmed());
        assert_eq!(subsystems.control.last_charmer_guid, Some(controller));

        subsystems.vehicle.enter_vehicle(vehicle, Some(1));
        subsystems.vehicle.base_vehicle_guid = Some(vehicle);
        subsystems.vehicle.set_vehicle_kit(42, true);
        assert_eq!(subsystems.vehicle.vehicle_guid, Some(vehicle));
        assert_eq!(subsystems.vehicle.seat_id, Some(1));
        assert_eq!(subsystems.vehicle.kit.map(|kit| kit.kit_id), Some(42));
        subsystems.vehicle.exit_vehicle();
        subsystems.vehicle.clear_vehicle_kit();
        assert_eq!(subsystems.vehicle.vehicle_guid, None);
        assert_eq!(subsystems.vehicle.kit, None);

        subsystems.ai.set_active(Some("NullAI"));
        subsystems.ai.push("CombatAI");
        assert_eq!(subsystems.ai.active_ai.as_deref(), Some("CombatAI"));
        assert_eq!(subsystems.ai.ai_stack, vec![String::from("NullAI")]);
        assert_eq!(subsystems.ai.pop().as_deref(), Some("CombatAI"));
        assert_eq!(subsystems.ai.active_ai.as_deref(), Some("NullAI"));
        subsystems.ai.set_locked(true);
        assert!(subsystems.ai.locked);
    }
}
