use std::f32::consts::{FRAC_PI_4, FRAC_PI_8, TAU};

use wow_core::{ObjectGuid, Position};

use crate::{
    MovementGenerator, MovementGeneratorFlags, MovementGeneratorMode, MovementGeneratorPriority,
    MovementGeneratorState, MovementGeneratorType, normalize_orientation_like_cpp,
};

pub const MIN_QUIET_DISTANCE_LIKE_CPP: f32 = 28.0;
pub const MAX_QUIET_DISTANCE_LIKE_CPP: f32 = 43.0;
pub const FLEEING_PATH_LENGTH_LIMIT_LIKE_CPP: f32 = 30.0;
pub const FLEEING_LOS_RETRY_MS_LIKE_CPP: i32 = 200;
pub const FLEEING_PATH_RETRY_MS_LIKE_CPP: i32 = 100;
pub const FLEEING_RANDOM_DELAY_MIN_MS_LIKE_CPP: i32 = 800;
pub const FLEEING_RANDOM_DELAY_MAX_MS_LIKE_CPP: i32 = 1500;
pub const UNIT_STATE_FLEEING_LIKE_CPP: u32 = 0x0000_0080;
pub const UNIT_STATE_FLEEING_MOVE_LIKE_CPP: u32 = 0x0200_0000;
pub const UNIT_FLAG_FLEEING_LIKE_CPP: u32 = 0x0080_0000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FleeingUnitSnapshot {
    pub owner_position: Position,
    pub owner_alive: bool,
    pub can_move: bool,
    pub movement_prevented_by_casting: bool,
    pub move_spline_finalized: bool,
    pub has_los_to_destination: bool,
    pub path_result: FleeingPathResult,
    pub travel_time_ms: i32,
    pub random_delay_ms: i32,
    pub flee_target_position: Option<Position>,
    pub random: FleeingRandomInputs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FleeingPathResult {
    Success,
    Failed,
    NoPath,
    Shortcut,
    FarFromPoly,
}

impl FleeingPathResult {
    #[must_use]
    pub const fn is_usable_like_cpp(self) -> bool {
        matches!(self, Self::Success)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FleeingRandomInputs {
    pub fallback_angle: f32,
    pub distance_factor: f32,
    pub angle_offset: f32,
    pub quiet_angle: f32,
}

impl Default for FleeingRandomInputs {
    fn default() -> Self {
        Self {
            fallback_angle: 0.0,
            distance_factor: 1.0,
            angle_offset: 0.0,
            quiet_angle: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FleeingDestinationPlan {
    pub caster_distance: f32,
    pub caster_angle: f32,
    pub distance: f32,
    pub angle: f32,
    pub destination: Position,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FleeingLaunchPlan {
    pub destination: Position,
    pub path_length_limit: f32,
    pub walk: bool,
    pub timer_ms: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FleeingMovementAction {
    Continue,
    Finished,
    StopMoving,
    RetryAfterLosFailure {
        timer_ms: i32,
    },
    RetryAfterPathFailure {
        timer_ms: i32,
        result: FleeingPathResult,
    },
    Launch(FleeingLaunchPlan),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FleeingFinalizeAction {
    pub remove_fleeing_flag: bool,
    pub clear_fleeing_move: bool,
    pub stop_moving: bool,
    pub set_target_to_victim: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimedFleeingFinalizeAction {
    pub remove_fleeing_flag: bool,
    pub stop_moving: bool,
    pub attack_stop: bool,
    pub attack_start_victim: bool,
    pub inform: Option<TimedFleeingInform>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimedFleeingInform {
    pub movement_type: MovementGeneratorType,
    pub point_id: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FleeingMovementGenerator {
    state: MovementGeneratorState,
    flee_target_guid: ObjectGuid,
    timer_ms: i32,
    path_allocated: bool,
    last_destination_plan: Option<FleeingDestinationPlan>,
    pub stop_moving_calls: u32,
    pub set_fleeing_flag_calls: u32,
    pub remove_fleeing_flag_calls: u32,
    pub finalize_action: Option<FleeingFinalizeAction>,
}

impl FleeingMovementGenerator {
    #[must_use]
    pub const fn new(flee_target_guid: ObjectGuid) -> Self {
        Self {
            state: MovementGeneratorState {
                mode: MovementGeneratorMode::Default,
                priority: MovementGeneratorPriority::Highest,
                flags: MovementGeneratorFlags::INITIALIZATION_PENDING,
                base_unit_state: UNIT_STATE_FLEEING_LIKE_CPP,
            },
            flee_target_guid,
            timer_ms: 0,
            path_allocated: false,
            last_destination_plan: None,
            stop_moving_calls: 0,
            set_fleeing_flag_calls: 0,
            remove_fleeing_flag_calls: 0,
            finalize_action: None,
        }
    }

    #[must_use]
    pub const fn flee_target_guid(&self) -> ObjectGuid {
        self.flee_target_guid
    }

    #[must_use]
    pub const fn timer_ms(&self) -> i32 {
        self.timer_ms
    }

    #[must_use]
    pub const fn path_allocated(&self) -> bool {
        self.path_allocated
    }

    #[must_use]
    pub const fn last_destination_plan(&self) -> Option<FleeingDestinationPlan> {
        self.last_destination_plan
    }

    pub fn initialize_like_cpp(
        &mut self,
        owner_exists: bool,
        snapshot: FleeingUnitSnapshot,
    ) -> FleeingMovementAction {
        self.remove_flag(
            MovementGeneratorFlags::INITIALIZATION_PENDING
                | MovementGeneratorFlags::TRANSITORY
                | MovementGeneratorFlags::DEACTIVATED,
        );
        self.add_flag(MovementGeneratorFlags::INITIALIZED);

        if !owner_exists || !snapshot.owner_alive {
            return FleeingMovementAction::Continue;
        }

        self.set_fleeing_flag_calls = self.set_fleeing_flag_calls.saturating_add(1);
        self.path_allocated = false;
        self.set_target_location_like_cpp(owner_exists, snapshot)
    }

    pub fn reset_like_cpp(
        &mut self,
        owner_exists: bool,
        snapshot: FleeingUnitSnapshot,
    ) -> FleeingMovementAction {
        self.remove_flag(MovementGeneratorFlags::TRANSITORY | MovementGeneratorFlags::DEACTIVATED);
        self.initialize_like_cpp(owner_exists, snapshot)
    }

    pub fn update_like_cpp(
        &mut self,
        owner_exists: bool,
        diff_ms: u32,
        snapshot: FleeingUnitSnapshot,
    ) -> FleeingMovementAction {
        if !owner_exists || !snapshot.owner_alive {
            return FleeingMovementAction::Finished;
        }

        if !snapshot.can_move || snapshot.movement_prevented_by_casting {
            self.add_flag(MovementGeneratorFlags::INTERRUPTED);
            self.stop_moving_calls = self.stop_moving_calls.saturating_add(1);
            self.path_allocated = false;
            return FleeingMovementAction::StopMoving;
        }

        self.remove_flag(MovementGeneratorFlags::INTERRUPTED);
        self.timer_ms = self.timer_ms.saturating_sub(diff_ms as i32);
        let speed_update_pending = self.has_flag(MovementGeneratorFlags::SPEED_UPDATE_PENDING);
        if (speed_update_pending && !snapshot.move_spline_finalized)
            || (self.timer_ms <= 0 && snapshot.move_spline_finalized)
        {
            self.remove_flag(MovementGeneratorFlags::TRANSITORY);
            return self.set_target_location_like_cpp(owner_exists, snapshot);
        }

        FleeingMovementAction::Continue
    }

    pub fn set_target_location_like_cpp(
        &mut self,
        owner_exists: bool,
        snapshot: FleeingUnitSnapshot,
    ) -> FleeingMovementAction {
        if !owner_exists || !snapshot.owner_alive {
            return FleeingMovementAction::Continue;
        }

        if !snapshot.can_move || snapshot.movement_prevented_by_casting {
            self.add_flag(MovementGeneratorFlags::INTERRUPTED);
            self.stop_moving_calls = self.stop_moving_calls.saturating_add(1);
            self.path_allocated = false;
            return FleeingMovementAction::StopMoving;
        }

        let destination_plan = compute_flee_destination_like_cpp(
            snapshot.owner_position,
            snapshot.flee_target_position,
            snapshot.random,
        );
        self.last_destination_plan = Some(destination_plan);

        if !snapshot.has_los_to_destination {
            self.timer_ms = FLEEING_LOS_RETRY_MS_LIKE_CPP;
            return FleeingMovementAction::RetryAfterLosFailure {
                timer_ms: self.timer_ms,
            };
        }

        self.path_allocated = true;
        if !snapshot.path_result.is_usable_like_cpp() {
            self.timer_ms = FLEEING_PATH_RETRY_MS_LIKE_CPP;
            return FleeingMovementAction::RetryAfterPathFailure {
                timer_ms: self.timer_ms,
                result: snapshot.path_result,
            };
        }

        self.timer_ms = snapshot.travel_time_ms + snapshot.random_delay_ms;
        FleeingMovementAction::Launch(FleeingLaunchPlan {
            destination: destination_plan.destination,
            path_length_limit: FLEEING_PATH_LENGTH_LIMIT_LIKE_CPP,
            walk: false,
            timer_ms: self.timer_ms,
        })
    }

    pub fn deactivate_like_cpp(&mut self) -> FleeingFinalizeAction {
        self.add_flag(MovementGeneratorFlags::DEACTIVATED);
        FleeingFinalizeAction {
            remove_fleeing_flag: false,
            clear_fleeing_move: true,
            stop_moving: false,
            set_target_to_victim: false,
        }
    }

    pub fn finalize_player_like_cpp(&mut self, active: bool) -> FleeingFinalizeAction {
        self.add_flag(MovementGeneratorFlags::FINALIZED);
        let action = FleeingFinalizeAction {
            remove_fleeing_flag: active,
            clear_fleeing_move: active,
            stop_moving: active,
            set_target_to_victim: false,
        };
        if active {
            self.remove_fleeing_flag_calls = self.remove_fleeing_flag_calls.saturating_add(1);
            self.stop_moving_calls = self.stop_moving_calls.saturating_add(1);
        }
        self.finalize_action = Some(action);
        action
    }

    pub fn finalize_creature_like_cpp(
        &mut self,
        active: bool,
        has_victim: bool,
    ) -> FleeingFinalizeAction {
        self.add_flag(MovementGeneratorFlags::FINALIZED);
        let action = FleeingFinalizeAction {
            remove_fleeing_flag: active,
            clear_fleeing_move: active,
            stop_moving: false,
            set_target_to_victim: active && has_victim,
        };
        if active {
            self.remove_fleeing_flag_calls = self.remove_fleeing_flag_calls.saturating_add(1);
        }
        self.finalize_action = Some(action);
        action
    }
}

impl MovementGenerator for FleeingMovementGenerator {
    fn state(&self) -> &MovementGeneratorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut MovementGeneratorState {
        &mut self.state
    }

    fn kind(&self) -> MovementGeneratorType {
        MovementGeneratorType::Fleeing
    }

    fn initialize(&mut self) {
        self.remove_flag(
            MovementGeneratorFlags::INITIALIZATION_PENDING
                | MovementGeneratorFlags::TRANSITORY
                | MovementGeneratorFlags::DEACTIVATED,
        );
        self.add_flag(MovementGeneratorFlags::INITIALIZED);
    }

    fn reset(&mut self) {
        self.remove_flag(MovementGeneratorFlags::TRANSITORY | MovementGeneratorFlags::DEACTIVATED);
        self.initialize();
    }

    fn update(&mut self, _diff_ms: u32) -> bool {
        true
    }

    fn deactivate(&mut self) {
        self.deactivate_like_cpp();
    }

    fn finalize(&mut self, active: bool, _movement_inform: bool) {
        self.finalize_player_like_cpp(active);
    }

    fn unit_speed_changed(&mut self) {
        self.add_flag(MovementGeneratorFlags::SPEED_UPDATE_PENDING);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimedFleeingMovementGenerator {
    fleeing: FleeingMovementGenerator,
    total_flee_time_ms: i32,
    pub finalize_action: Option<TimedFleeingFinalizeAction>,
}

impl TimedFleeingMovementGenerator {
    #[must_use]
    pub const fn new(flee_target_guid: ObjectGuid, total_flee_time_ms: i32) -> Self {
        Self {
            fleeing: FleeingMovementGenerator::new(flee_target_guid),
            total_flee_time_ms,
            finalize_action: None,
        }
    }

    #[must_use]
    pub const fn fleeing(&self) -> &FleeingMovementGenerator {
        &self.fleeing
    }

    #[must_use]
    pub const fn fleeing_mut(&mut self) -> &mut FleeingMovementGenerator {
        &mut self.fleeing
    }

    #[must_use]
    pub const fn total_flee_time_ms(&self) -> i32 {
        self.total_flee_time_ms
    }

    pub fn initialize_like_cpp(
        &mut self,
        owner_exists: bool,
        snapshot: FleeingUnitSnapshot,
    ) -> FleeingMovementAction {
        self.fleeing.initialize_like_cpp(owner_exists, snapshot)
    }

    pub fn update_like_cpp(
        &mut self,
        owner_exists: bool,
        diff_ms: u32,
        snapshot: FleeingUnitSnapshot,
    ) -> FleeingMovementAction {
        if !owner_exists || !snapshot.owner_alive {
            return FleeingMovementAction::Finished;
        }

        self.total_flee_time_ms = self.total_flee_time_ms.saturating_sub(diff_ms as i32);
        if self.total_flee_time_ms <= 0 {
            return FleeingMovementAction::Finished;
        }

        self.fleeing
            .update_like_cpp(owner_exists, diff_ms, snapshot)
    }

    pub fn finalize_like_cpp(
        &mut self,
        active: bool,
        owner_alive: bool,
        has_victim: bool,
        movement_inform: bool,
    ) -> TimedFleeingFinalizeAction {
        self.fleeing.add_flag(MovementGeneratorFlags::FINALIZED);
        let action = TimedFleeingFinalizeAction {
            remove_fleeing_flag: active,
            stop_moving: active,
            attack_stop: active && owner_alive && has_victim,
            attack_start_victim: active && owner_alive && has_victim,
            inform: (active && movement_inform).then_some(TimedFleeingInform {
                movement_type: MovementGeneratorType::TimedFleeing,
                point_id: 0,
            }),
        };
        if active {
            self.fleeing.remove_fleeing_flag_calls =
                self.fleeing.remove_fleeing_flag_calls.saturating_add(1);
            self.fleeing.stop_moving_calls = self.fleeing.stop_moving_calls.saturating_add(1);
        }
        self.finalize_action = Some(action);
        action
    }
}

impl MovementGenerator for TimedFleeingMovementGenerator {
    fn state(&self) -> &MovementGeneratorState {
        self.fleeing.state()
    }

    fn state_mut(&mut self) -> &mut MovementGeneratorState {
        self.fleeing.state_mut()
    }

    fn kind(&self) -> MovementGeneratorType {
        MovementGeneratorType::TimedFleeing
    }

    fn initialize(&mut self) {
        self.fleeing.initialize();
    }

    fn reset(&mut self) {
        self.fleeing.reset();
    }

    fn update(&mut self, diff_ms: u32) -> bool {
        self.total_flee_time_ms = self.total_flee_time_ms.saturating_sub(diff_ms as i32);
        self.total_flee_time_ms > 0
    }

    fn deactivate(&mut self) {
        self.fleeing.deactivate();
    }

    fn finalize(&mut self, active: bool, movement_inform: bool) {
        self.finalize_like_cpp(active, true, false, movement_inform);
    }

    fn unit_speed_changed(&mut self) {
        self.fleeing.unit_speed_changed();
    }
}

#[must_use]
pub fn compute_flee_destination_like_cpp(
    owner_position: Position,
    flee_target_position: Option<Position>,
    random: FleeingRandomInputs,
) -> FleeingDestinationPlan {
    let (caster_distance, caster_angle) = if let Some(flee_target_position) = flee_target_position {
        let distance = distance_like_cpp(flee_target_position, owner_position);
        let angle = if distance > 0.2 {
            absolute_angle_like_cpp(flee_target_position, owner_position)
        } else {
            normalize_orientation_like_cpp(random.fallback_angle)
        };
        (distance, angle)
    } else {
        (0.0, normalize_orientation_like_cpp(random.fallback_angle))
    };

    let (distance, angle) = if caster_distance < MIN_QUIET_DISTANCE_LIKE_CPP {
        (
            random.distance_factor * (MIN_QUIET_DISTANCE_LIKE_CPP - caster_distance),
            caster_angle + clamp(random.angle_offset, -FRAC_PI_8, FRAC_PI_8),
        )
    } else if caster_distance > MAX_QUIET_DISTANCE_LIKE_CPP {
        (
            random.distance_factor * (MAX_QUIET_DISTANCE_LIKE_CPP - MIN_QUIET_DISTANCE_LIKE_CPP),
            -caster_angle + clamp(random.angle_offset, -FRAC_PI_4, FRAC_PI_4),
        )
    } else {
        (
            random.distance_factor * (MAX_QUIET_DISTANCE_LIKE_CPP - MIN_QUIET_DISTANCE_LIKE_CPP),
            normalize_orientation_like_cpp(random.quiet_angle),
        )
    };

    let angle = normalize_orientation_like_cpp(angle);
    let destination = Position::new(
        owner_position.x + distance * angle.cos(),
        owner_position.y + distance * angle.sin(),
        owner_position.z,
        owner_position.orientation,
    );

    FleeingDestinationPlan {
        caster_distance,
        caster_angle,
        distance,
        angle,
        destination,
    }
}

fn distance_like_cpp(left: Position, right: Position) -> f32 {
    let dx = left.x - right.x;
    let dy = left.y - right.y;
    let dz = left.z - right.z;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn absolute_angle_like_cpp(from: Position, to: Position) -> f32 {
    normalize_orientation_like_cpp((to.y - from.y).atan2(to.x - from.x))
}

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guid(counter: i64) -> ObjectGuid {
        ObjectGuid::create_uniq(counter)
    }

    fn snapshot(owner: Position, target: Option<Position>) -> FleeingUnitSnapshot {
        FleeingUnitSnapshot {
            owner_position: owner,
            owner_alive: true,
            can_move: true,
            movement_prevented_by_casting: false,
            move_spline_finalized: true,
            has_los_to_destination: true,
            path_result: FleeingPathResult::Success,
            travel_time_ms: 300,
            random_delay_ms: FLEEING_RANDOM_DELAY_MIN_MS_LIKE_CPP,
            flee_target_position: target,
            random: FleeingRandomInputs {
                fallback_angle: 1.0,
                distance_factor: 1.0,
                angle_offset: 0.0,
                quiet_angle: 2.0,
            },
        }
    }

    #[test]
    fn fleeing_constructor_and_initialize_match_cpp_shape() {
        let owner = Position::new(10.0, 0.0, 0.0, 0.0);
        let target = Position::new(0.0, 0.0, 0.0, 0.0);
        let mut fleeing = FleeingMovementGenerator::new(guid(7));

        assert_eq!(fleeing.kind(), MovementGeneratorType::Fleeing);
        assert_eq!(fleeing.flee_target_guid(), guid(7));
        assert_eq!(fleeing.state().mode, MovementGeneratorMode::Default);
        assert_eq!(fleeing.state().priority, MovementGeneratorPriority::Highest);
        assert_eq!(
            fleeing.state().flags,
            MovementGeneratorFlags::INITIALIZATION_PENDING
        );
        assert_eq!(fleeing.state().base_unit_state, UNIT_STATE_FLEEING_LIKE_CPP);
        assert_eq!(UNIT_STATE_FLEEING_MOVE_LIKE_CPP, 0x0200_0000);
        assert_eq!(UNIT_FLAG_FLEEING_LIKE_CPP, 0x0080_0000);

        let action = fleeing.initialize_like_cpp(true, snapshot(owner, Some(target)));
        assert!(fleeing.has_flag(MovementGeneratorFlags::INITIALIZED));
        assert!(!fleeing.has_flag(MovementGeneratorFlags::INITIALIZATION_PENDING));
        assert_eq!(fleeing.set_fleeing_flag_calls, 1);
        assert!(fleeing.path_allocated());
        assert!(matches!(action, FleeingMovementAction::Launch(_)));
    }

    #[test]
    fn fleeing_destination_uses_cpp_quiet_distance_branches() {
        let target = Position::new(0.0, 0.0, 0.0, 0.0);

        let too_close = compute_flee_destination_like_cpp(
            Position::new(10.0, 0.0, 0.0, 0.0),
            Some(target),
            FleeingRandomInputs::default(),
        );
        assert_eq!(too_close.caster_distance, 10.0);
        assert_eq!(too_close.distance, MIN_QUIET_DISTANCE_LIKE_CPP - 10.0);
        assert!(too_close.destination.x > 10.0);

        let quiet = compute_flee_destination_like_cpp(
            Position::new(35.0, 0.0, 0.0, 0.0),
            Some(target),
            FleeingRandomInputs {
                quiet_angle: 0.5,
                ..FleeingRandomInputs::default()
            },
        );
        assert_eq!(
            quiet.distance,
            MAX_QUIET_DISTANCE_LIKE_CPP - MIN_QUIET_DISTANCE_LIKE_CPP
        );
        assert_eq!(quiet.angle, 0.5);

        let too_far = compute_flee_destination_like_cpp(
            Position::new(50.0, 0.0, 0.0, 0.0),
            Some(target),
            FleeingRandomInputs::default(),
        );
        assert_eq!(
            too_far.distance,
            MAX_QUIET_DISTANCE_LIKE_CPP - MIN_QUIET_DISTANCE_LIKE_CPP
        );
        assert_eq!(too_far.angle, 0.0);
    }

    #[test]
    fn fleeing_set_target_retries_los_and_path_failures_like_cpp() {
        let owner = Position::new(10.0, 0.0, 0.0, 0.0);
        let target = Position::new(0.0, 0.0, 0.0, 0.0);
        let mut fleeing = FleeingMovementGenerator::new(guid(7));

        let mut no_los = snapshot(owner, Some(target));
        no_los.has_los_to_destination = false;
        assert_eq!(
            fleeing.set_target_location_like_cpp(true, no_los),
            FleeingMovementAction::RetryAfterLosFailure {
                timer_ms: FLEEING_LOS_RETRY_MS_LIKE_CPP,
            }
        );
        assert_eq!(fleeing.timer_ms(), FLEEING_LOS_RETRY_MS_LIKE_CPP);
        assert!(!fleeing.path_allocated());

        let mut no_path = snapshot(owner, Some(target));
        no_path.path_result = FleeingPathResult::NoPath;
        assert_eq!(
            fleeing.set_target_location_like_cpp(true, no_path),
            FleeingMovementAction::RetryAfterPathFailure {
                timer_ms: FLEEING_PATH_RETRY_MS_LIKE_CPP,
                result: FleeingPathResult::NoPath,
            }
        );
        assert_eq!(fleeing.timer_ms(), FLEEING_PATH_RETRY_MS_LIKE_CPP);
        assert!(fleeing.path_allocated());
    }

    #[test]
    fn fleeing_update_blocks_and_relaunches_like_cpp() {
        let owner = Position::new(10.0, 0.0, 0.0, 0.0);
        let target = Position::new(0.0, 0.0, 0.0, 0.0);
        let mut fleeing = FleeingMovementGenerator::new(guid(7));

        let mut blocked = snapshot(owner, Some(target));
        blocked.can_move = false;
        assert_eq!(
            fleeing.update_like_cpp(true, 50, blocked),
            FleeingMovementAction::StopMoving
        );
        assert!(fleeing.has_flag(MovementGeneratorFlags::INTERRUPTED));
        assert_eq!(fleeing.stop_moving_calls, 1);
        assert!(!fleeing.path_allocated());

        let launched = fleeing.update_like_cpp(true, 50, snapshot(owner, Some(target)));
        assert!(matches!(launched, FleeingMovementAction::Launch(_)));
        assert!(!fleeing.has_flag(MovementGeneratorFlags::INTERRUPTED));

        fleeing.unit_speed_changed();
        let mut moving = snapshot(owner, Some(target));
        moving.move_spline_finalized = false;
        assert!(matches!(
            fleeing.update_like_cpp(true, 1, moving),
            FleeingMovementAction::Launch(_)
        ));
        assert!(!fleeing.has_flag(MovementGeneratorFlags::SPEED_UPDATE_PENDING));
    }

    #[test]
    fn fleeing_finalize_specializations_match_player_and_creature_cpp() {
        let mut fleeing = FleeingMovementGenerator::new(guid(7));

        assert_eq!(
            fleeing.finalize_player_like_cpp(true),
            FleeingFinalizeAction {
                remove_fleeing_flag: true,
                clear_fleeing_move: true,
                stop_moving: true,
                set_target_to_victim: false,
            }
        );

        let mut creature = FleeingMovementGenerator::new(guid(7));
        assert_eq!(
            creature.finalize_creature_like_cpp(true, true),
            FleeingFinalizeAction {
                remove_fleeing_flag: true,
                clear_fleeing_move: true,
                stop_moving: false,
                set_target_to_victim: true,
            }
        );

        assert_eq!(
            creature.deactivate_like_cpp(),
            FleeingFinalizeAction {
                remove_fleeing_flag: false,
                clear_fleeing_move: true,
                stop_moving: false,
                set_target_to_victim: false,
            }
        );
    }

    #[test]
    fn timed_fleeing_update_and_finalize_match_cpp() {
        let owner = Position::new(10.0, 0.0, 0.0, 0.0);
        let target = Position::new(0.0, 0.0, 0.0, 0.0);
        let mut timed = TimedFleeingMovementGenerator::new(guid(7), 100);

        assert_eq!(timed.kind(), MovementGeneratorType::TimedFleeing);
        assert!(matches!(
            timed.update_like_cpp(true, 20, snapshot(owner, Some(target))),
            FleeingMovementAction::Launch(_)
        ));
        assert_eq!(timed.total_flee_time_ms(), 80);
        assert_eq!(
            timed.update_like_cpp(true, 80, snapshot(owner, Some(target))),
            FleeingMovementAction::Finished
        );

        assert_eq!(
            timed.finalize_like_cpp(true, true, true, true),
            TimedFleeingFinalizeAction {
                remove_fleeing_flag: true,
                stop_moving: true,
                attack_stop: true,
                attack_start_victim: true,
                inform: Some(TimedFleeingInform {
                    movement_type: MovementGeneratorType::TimedFleeing,
                    point_id: 0,
                }),
            }
        );
    }

    #[test]
    fn random_delay_bounds_are_cpp_literals() {
        assert_eq!(FLEEING_RANDOM_DELAY_MIN_MS_LIKE_CPP, 800);
        assert_eq!(FLEEING_RANDOM_DELAY_MAX_MS_LIKE_CPP, 1500);
        assert!((TAU - std::f32::consts::PI * 2.0).abs() < f32::EPSILON);
    }
}
