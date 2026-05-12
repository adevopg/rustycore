use crate::{
    MovementGenerator, MovementGeneratorFlags, MovementGeneratorMode, MovementGeneratorPriority,
    MovementGeneratorState, MovementGeneratorType, RotateDirection,
};

pub const UNIT_STATE_ROTATING_LIKE_CPP: u32 = 0x0020_0000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RotateFacingSpline {
    pub facing_angle: f32,
    pub disable_transport_path_transformations: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RotateMovementUpdate {
    pub keep_running: bool,
    pub facing_spline: Option<RotateFacingSpline>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RotateMovementInform {
    pub movement_type: MovementGeneratorType,
    pub movement_id: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RotateMovementGenerator {
    state: MovementGeneratorState,
    id: u32,
    duration_ms: u32,
    max_duration_ms: u32,
    direction: RotateDirection,
    pub stop_moving_calls: u32,
    pub last_facing_spline: Option<RotateFacingSpline>,
    pub movement_inform: Option<RotateMovementInform>,
}

impl RotateMovementGenerator {
    #[must_use]
    pub const fn new(id: u32, time_ms: u32, direction: RotateDirection) -> Self {
        Self {
            state: MovementGeneratorState {
                mode: MovementGeneratorMode::Default,
                priority: MovementGeneratorPriority::Normal,
                flags: MovementGeneratorFlags::INITIALIZATION_PENDING,
                base_unit_state: UNIT_STATE_ROTATING_LIKE_CPP,
            },
            id,
            duration_ms: time_ms,
            max_duration_ms: time_ms,
            direction,
            stop_moving_calls: 0,
            last_facing_spline: None,
            movement_inform: None,
        }
    }

    #[must_use]
    pub const fn id(&self) -> u32 {
        self.id
    }

    #[must_use]
    pub const fn duration_ms(&self) -> u32 {
        self.duration_ms
    }

    #[must_use]
    pub const fn max_duration_ms(&self) -> u32 {
        self.max_duration_ms
    }

    #[must_use]
    pub const fn direction(&self) -> RotateDirection {
        self.direction
    }

    pub fn update_with_owner_like_cpp(
        &mut self,
        owner_exists: bool,
        diff_ms: u32,
        current_orientation: f32,
        owner_on_transport: bool,
    ) -> RotateMovementUpdate {
        if !owner_exists {
            return RotateMovementUpdate {
                keep_running: false,
                facing_spline: None,
            };
        }

        let sign = match self.direction {
            RotateDirection::Left => 1.0,
            RotateDirection::Right => -1.0,
        };
        let facing_angle = if self.max_duration_ms == 0 {
            current_orientation
        } else {
            (current_orientation
                + (diff_ms as f32 * std::f32::consts::TAU / self.max_duration_ms as f32) * sign)
                .clamp(0.0, std::f32::consts::TAU)
        };
        let facing_spline = RotateFacingSpline {
            facing_angle,
            disable_transport_path_transformations: owner_on_transport,
        };
        self.last_facing_spline = Some(facing_spline);

        if self.duration_ms > diff_ms {
            self.duration_ms -= diff_ms;
            RotateMovementUpdate {
                keep_running: true,
                facing_spline: Some(facing_spline),
            }
        } else {
            self.add_flag(MovementGeneratorFlags::INFORM_ENABLED);
            RotateMovementUpdate {
                keep_running: false,
                facing_spline: Some(facing_spline),
            }
        }
    }
}

impl MovementGenerator for RotateMovementGenerator {
    fn state(&self) -> &MovementGeneratorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut MovementGeneratorState {
        &mut self.state
    }

    fn kind(&self) -> MovementGeneratorType {
        MovementGeneratorType::Rotate
    }

    fn initialize(&mut self) {
        self.remove_flag(
            MovementGeneratorFlags::INITIALIZATION_PENDING | MovementGeneratorFlags::DEACTIVATED,
        );
        self.add_flag(MovementGeneratorFlags::INITIALIZED);
        self.stop_moving_calls = self.stop_moving_calls.saturating_add(1);
    }

    fn reset(&mut self) {
        self.remove_flag(MovementGeneratorFlags::DEACTIVATED);
        self.initialize();
    }

    fn update(&mut self, diff_ms: u32) -> bool {
        let current_orientation = self
            .last_facing_spline
            .map_or(0.0, |facing_spline| facing_spline.facing_angle);
        self.update_with_owner_like_cpp(true, diff_ms, current_orientation, false)
            .keep_running
    }

    fn deactivate(&mut self) {
        self.add_flag(MovementGeneratorFlags::DEACTIVATED);
    }

    fn finalize(&mut self, _active: bool, movement_inform: bool) {
        self.add_flag(MovementGeneratorFlags::FINALIZED);
        self.movement_inform = movement_inform.then_some(RotateMovementInform {
            movement_type: MovementGeneratorType::Rotate,
            movement_id: self.id,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.000_01;

    fn assert_close(left: f32, right: f32) {
        assert!((left - right).abs() < EPSILON, "left={left}, right={right}");
    }

    #[test]
    fn rotate_movement_generator_matches_cpp_lifecycle_shape() {
        let mut rotate = RotateMovementGenerator::new(7, 1_000, RotateDirection::Left);
        assert_eq!(rotate.kind(), MovementGeneratorType::Rotate);
        assert_eq!(rotate.id(), 7);
        assert_eq!(rotate.duration_ms(), 1_000);
        assert_eq!(rotate.max_duration_ms(), 1_000);
        assert_eq!(rotate.direction(), RotateDirection::Left);
        assert_eq!(rotate.state().mode, MovementGeneratorMode::Default);
        assert_eq!(rotate.state().priority, MovementGeneratorPriority::Normal);
        assert_eq!(
            rotate.state().flags,
            MovementGeneratorFlags::INITIALIZATION_PENDING
        );
        assert_eq!(rotate.state().base_unit_state, UNIT_STATE_ROTATING_LIKE_CPP);

        rotate.initialize();
        assert_eq!(rotate.stop_moving_calls, 1);
        assert!(!rotate.has_flag(MovementGeneratorFlags::INITIALIZATION_PENDING));
        assert!(!rotate.has_flag(MovementGeneratorFlags::DEACTIVATED));
        assert!(rotate.has_flag(MovementGeneratorFlags::INITIALIZED));

        rotate.deactivate();
        assert!(rotate.has_flag(MovementGeneratorFlags::DEACTIVATED));
        rotate.reset();
        assert_eq!(rotate.stop_moving_calls, 2);
        assert!(!rotate.has_flag(MovementGeneratorFlags::DEACTIVATED));

        rotate.finalize(true, true);
        assert!(rotate.has_flag(MovementGeneratorFlags::FINALIZED));
        assert_eq!(
            rotate.movement_inform,
            Some(RotateMovementInform {
                movement_type: MovementGeneratorType::Rotate,
                movement_id: 7,
            })
        );
    }

    #[test]
    fn rotate_update_matches_cpp_orientation_duration_and_transport_rules() {
        let mut left = RotateMovementGenerator::new(7, 1_000, RotateDirection::Left);
        let update = left.update_with_owner_like_cpp(true, 250, 1.0, true);
        assert!(update.keep_running);
        let facing = update.facing_spline.expect("facing spline");
        assert_close(facing.facing_angle, 1.0 + std::f32::consts::TAU / 4.0);
        assert!(facing.disable_transport_path_transformations);
        assert_eq!(left.duration_ms(), 750);
        assert!(!left.has_flag(MovementGeneratorFlags::INFORM_ENABLED));

        let mut right = RotateMovementGenerator::new(7, 1_000, RotateDirection::Right);
        let update = right.update_with_owner_like_cpp(true, 250, 1.0, false);
        assert!(update.keep_running);
        let facing = update.facing_spline.expect("facing spline");
        assert_close(facing.facing_angle, 0.0);
        assert!(!facing.disable_transport_path_transformations);

        let update = right.update_with_owner_like_cpp(true, 1_000, 2.0, false);
        assert!(!update.keep_running);
        assert!(update.facing_spline.is_some());
        assert!(right.has_flag(MovementGeneratorFlags::INFORM_ENABLED));
    }

    #[test]
    fn rotate_update_without_owner_matches_cpp_false_without_spline() {
        let mut rotate = RotateMovementGenerator::new(7, 1_000, RotateDirection::Left);
        let update = rotate.update_with_owner_like_cpp(false, 250, 1.0, false);
        assert!(!update.keep_running);
        assert!(update.facing_spline.is_none());
        assert_eq!(rotate.duration_ms(), 1_000);
    }
}
