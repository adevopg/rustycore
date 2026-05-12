use crate::{
    MovementGenerator, MovementGeneratorFlags, MovementGeneratorMode, MovementGeneratorPriority,
    MovementGeneratorState, MovementGeneratorType,
};

pub const UNIT_STATE_DISTRACTED_LIKE_CPP: u32 = 0x0000_1000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DistractFacingSpline {
    pub facing_angle: f32,
    pub disable_transport_path_transformations: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DistractInitializeAction {
    pub stand_up: bool,
    pub facing_spline: DistractFacingSpline,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DistractFinalizeAction {
    pub return_home_orientation: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssistanceDistractFinalizeAction {
    pub set_react_aggressive: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DistractMovementGenerator {
    state: MovementGeneratorState,
    timer_ms: u32,
    orientation: f32,
    pub last_initialize_action: Option<DistractInitializeAction>,
    pub finalize_action: Option<DistractFinalizeAction>,
}

impl DistractMovementGenerator {
    #[must_use]
    pub const fn new(timer_ms: u32, orientation: f32) -> Self {
        Self::new_with_priority(timer_ms, orientation, MovementGeneratorPriority::Highest)
    }

    const fn new_with_priority(
        timer_ms: u32,
        orientation: f32,
        priority: MovementGeneratorPriority,
    ) -> Self {
        Self {
            state: MovementGeneratorState {
                mode: MovementGeneratorMode::Default,
                priority,
                flags: MovementGeneratorFlags::INITIALIZATION_PENDING,
                base_unit_state: UNIT_STATE_DISTRACTED_LIKE_CPP,
            },
            timer_ms,
            orientation,
            last_initialize_action: None,
            finalize_action: None,
        }
    }

    #[must_use]
    pub const fn timer_ms(&self) -> u32 {
        self.timer_ms
    }

    #[must_use]
    pub const fn orientation(&self) -> f32 {
        self.orientation
    }

    pub fn initialize_with_owner_like_cpp(
        &mut self,
        owner_is_standing: bool,
        owner_on_transport: bool,
    ) -> DistractInitializeAction {
        self.remove_flag(
            MovementGeneratorFlags::INITIALIZATION_PENDING | MovementGeneratorFlags::DEACTIVATED,
        );
        self.add_flag(MovementGeneratorFlags::INITIALIZED);

        let action = DistractInitializeAction {
            stand_up: !owner_is_standing,
            facing_spline: DistractFacingSpline {
                facing_angle: self.orientation,
                disable_transport_path_transformations: owner_on_transport,
            },
        };
        self.last_initialize_action = Some(action);
        action
    }

    pub fn reset_with_owner_like_cpp(
        &mut self,
        owner_is_standing: bool,
        owner_on_transport: bool,
    ) -> DistractInitializeAction {
        self.remove_flag(MovementGeneratorFlags::DEACTIVATED);
        self.initialize_with_owner_like_cpp(owner_is_standing, owner_on_transport)
    }

    pub fn update_with_owner_like_cpp(&mut self, owner_exists: bool, diff_ms: u32) -> bool {
        if !owner_exists {
            return false;
        }

        if diff_ms > self.timer_ms {
            self.add_flag(MovementGeneratorFlags::INFORM_ENABLED);
            return false;
        }

        self.timer_ms -= diff_ms;
        true
    }

    pub fn finalize_with_owner_like_cpp(
        &mut self,
        movement_inform: bool,
        owner_is_creature: bool,
        home_orientation: f32,
    ) -> DistractFinalizeAction {
        self.add_flag(MovementGeneratorFlags::FINALIZED);
        let action = DistractFinalizeAction {
            return_home_orientation: (movement_inform
                && self.has_flag(MovementGeneratorFlags::INFORM_ENABLED)
                && owner_is_creature)
                .then_some(home_orientation),
        };
        self.finalize_action = Some(action);
        action
    }
}

impl MovementGenerator for DistractMovementGenerator {
    fn state(&self) -> &MovementGeneratorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut MovementGeneratorState {
        &mut self.state
    }

    fn kind(&self) -> MovementGeneratorType {
        MovementGeneratorType::Distract
    }

    fn initialize(&mut self) {
        self.initialize_with_owner_like_cpp(true, false);
    }

    fn reset(&mut self) {
        self.reset_with_owner_like_cpp(true, false);
    }

    fn update(&mut self, diff_ms: u32) -> bool {
        self.update_with_owner_like_cpp(true, diff_ms)
    }

    fn deactivate(&mut self) {
        self.add_flag(MovementGeneratorFlags::DEACTIVATED);
    }

    fn finalize(&mut self, _active: bool, movement_inform: bool) {
        self.finalize_with_owner_like_cpp(movement_inform, false, 0.0);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssistanceDistractMovementGenerator {
    inner: DistractMovementGenerator,
    pub finalize_action: Option<AssistanceDistractFinalizeAction>,
}

impl AssistanceDistractMovementGenerator {
    #[must_use]
    pub const fn new(timer_ms: u32, orientation: f32) -> Self {
        Self {
            inner: DistractMovementGenerator::new_with_priority(
                timer_ms,
                orientation,
                MovementGeneratorPriority::Normal,
            ),
            finalize_action: None,
        }
    }

    #[must_use]
    pub const fn timer_ms(&self) -> u32 {
        self.inner.timer_ms()
    }

    #[must_use]
    pub const fn orientation(&self) -> f32 {
        self.inner.orientation()
    }

    pub fn initialize_with_owner_like_cpp(
        &mut self,
        owner_is_standing: bool,
        owner_on_transport: bool,
    ) -> DistractInitializeAction {
        self.inner
            .initialize_with_owner_like_cpp(owner_is_standing, owner_on_transport)
    }

    pub fn reset_with_owner_like_cpp(
        &mut self,
        owner_is_standing: bool,
        owner_on_transport: bool,
    ) -> DistractInitializeAction {
        self.inner
            .reset_with_owner_like_cpp(owner_is_standing, owner_on_transport)
    }

    pub fn update_with_owner_like_cpp(&mut self, owner_exists: bool, diff_ms: u32) -> bool {
        self.inner.update_with_owner_like_cpp(owner_exists, diff_ms)
    }

    pub fn finalize_with_owner_like_cpp(
        &mut self,
        movement_inform: bool,
        owner_is_creature: bool,
    ) -> AssistanceDistractFinalizeAction {
        self.add_flag(MovementGeneratorFlags::FINALIZED);
        let action = AssistanceDistractFinalizeAction {
            set_react_aggressive: movement_inform
                && self.has_flag(MovementGeneratorFlags::INFORM_ENABLED)
                && owner_is_creature,
        };
        self.finalize_action = Some(action);
        action
    }
}

impl MovementGenerator for AssistanceDistractMovementGenerator {
    fn state(&self) -> &MovementGeneratorState {
        self.inner.state()
    }

    fn state_mut(&mut self) -> &mut MovementGeneratorState {
        self.inner.state_mut()
    }

    fn kind(&self) -> MovementGeneratorType {
        MovementGeneratorType::AssistanceDistract
    }

    fn initialize(&mut self) {
        self.inner.initialize();
    }

    fn reset(&mut self) {
        self.inner.reset();
    }

    fn update(&mut self, diff_ms: u32) -> bool {
        self.inner.update(diff_ms)
    }

    fn deactivate(&mut self) {
        self.inner.deactivate();
    }

    fn finalize(&mut self, _active: bool, movement_inform: bool) {
        self.finalize_with_owner_like_cpp(movement_inform, false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distract_movement_generator_matches_cpp_lifecycle_shape() {
        let mut distract = DistractMovementGenerator::new(1_000, 1.25);
        assert_eq!(distract.kind(), MovementGeneratorType::Distract);
        assert_eq!(distract.timer_ms(), 1_000);
        assert_eq!(distract.orientation(), 1.25);
        assert_eq!(distract.state().mode, MovementGeneratorMode::Default);
        assert_eq!(
            distract.state().priority,
            MovementGeneratorPriority::Highest
        );
        assert_eq!(
            distract.state().flags,
            MovementGeneratorFlags::INITIALIZATION_PENDING
        );
        assert_eq!(
            distract.state().base_unit_state,
            UNIT_STATE_DISTRACTED_LIKE_CPP
        );

        let action = distract.initialize_with_owner_like_cpp(false, true);
        assert!(action.stand_up);
        assert_eq!(action.facing_spline.facing_angle, 1.25);
        assert!(action.facing_spline.disable_transport_path_transformations);
        assert!(!distract.has_flag(MovementGeneratorFlags::INITIALIZATION_PENDING));
        assert!(distract.has_flag(MovementGeneratorFlags::INITIALIZED));

        distract.deactivate();
        assert!(distract.has_flag(MovementGeneratorFlags::DEACTIVATED));
        let action = distract.reset_with_owner_like_cpp(true, false);
        assert!(!action.stand_up);
        assert!(!action.facing_spline.disable_transport_path_transformations);
        assert!(!distract.has_flag(MovementGeneratorFlags::DEACTIVATED));
    }

    #[test]
    fn distract_update_and_finalize_match_cpp_timer_rules() {
        let mut distract = DistractMovementGenerator::new(100, 2.0);
        assert!(!distract.update_with_owner_like_cpp(false, 1));
        assert_eq!(distract.timer_ms(), 100);

        assert!(distract.update_with_owner_like_cpp(true, 100));
        assert_eq!(distract.timer_ms(), 0);
        assert!(!distract.has_flag(MovementGeneratorFlags::INFORM_ENABLED));

        assert!(!distract.update_with_owner_like_cpp(true, 1));
        assert!(distract.has_flag(MovementGeneratorFlags::INFORM_ENABLED));

        let action = distract.finalize_with_owner_like_cpp(true, true, 4.5);
        assert!(distract.has_flag(MovementGeneratorFlags::FINALIZED));
        assert_eq!(action.return_home_orientation, Some(4.5));

        let mut no_inform = DistractMovementGenerator::new(100, 2.0);
        let action = no_inform.finalize_with_owner_like_cpp(true, true, 4.5);
        assert_eq!(action.return_home_orientation, None);
    }

    #[test]
    fn assistance_distract_overrides_priority_kind_and_finalize_like_cpp() {
        let mut assistance = AssistanceDistractMovementGenerator::new(100, 2.0);
        assert_eq!(assistance.kind(), MovementGeneratorType::AssistanceDistract);
        assert_eq!(
            assistance.state().priority,
            MovementGeneratorPriority::Normal
        );
        assert_eq!(
            assistance.state().base_unit_state,
            UNIT_STATE_DISTRACTED_LIKE_CPP
        );

        assert!(!assistance.update_with_owner_like_cpp(true, 101));
        assert!(assistance.has_flag(MovementGeneratorFlags::INFORM_ENABLED));
        let action = assistance.finalize_with_owner_like_cpp(true, true);
        assert!(assistance.has_flag(MovementGeneratorFlags::FINALIZED));
        assert!(action.set_react_aggressive);

        let mut no_creature = AssistanceDistractMovementGenerator::new(100, 2.0);
        assert!(!no_creature.update_with_owner_like_cpp(true, 101));
        let action = no_creature.finalize_with_owner_like_cpp(true, false);
        assert!(!action.set_react_aggressive);
    }
}
