use std::collections::VecDeque;

use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct MotionMasterFlags: u8 {
        const NONE = 0x0;
        const UPDATE = 0x1;
        const STATIC_INITIALIZATION_PENDING = 0x2;
        const INITIALIZATION_PENDING = 0x4;
        const INITIALIZING = 0x8;
        const DELAYED = Self::UPDATE.bits() | Self::INITIALIZATION_PENDING.bits();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MotionMasterDelayedActionType {
    Clear = 0,
    ClearSlot = 1,
    ClearMode = 2,
    ClearPriority = 3,
    Add = 4,
    Remove = 5,
    RemoveType = 6,
    Initialize = 7,
}

impl MotionMasterDelayedActionType {
    #[must_use]
    pub const fn trinity_id(self) -> u8 {
        self as u8
    }

    #[must_use]
    pub const fn from_trinity_id(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Clear),
            1 => Some(Self::ClearSlot),
            2 => Some(Self::ClearMode),
            3 => Some(Self::ClearPriority),
            4 => Some(Self::Add),
            5 => Some(Self::Remove),
            6 => Some(Self::RemoveType),
            7 => Some(Self::Initialize),
            _ => None,
        }
    }
}

pub struct DelayedAction<M> {
    pub action_type: MotionMasterDelayedActionType,
    action: Box<dyn FnOnce(&mut M) + Send>,
    validator: Box<dyn Fn() -> bool + Send>,
}

impl<M> DelayedAction<M> {
    #[must_use]
    pub fn new(
        action_type: MotionMasterDelayedActionType,
        action: impl FnOnce(&mut M) + Send + 'static,
    ) -> Self {
        Self {
            action_type,
            action: Box::new(action),
            validator: Box::new(|| true),
        }
    }

    #[must_use]
    pub fn with_validator(
        action_type: MotionMasterDelayedActionType,
        action: impl FnOnce(&mut M) + Send + 'static,
        validator: impl Fn() -> bool + Send + 'static,
    ) -> Self {
        Self {
            action_type,
            action: Box::new(action),
            validator: Box::new(validator),
        }
    }

    pub fn resolve(self, motion_master: &mut M) -> bool {
        if !(self.validator)() {
            return false;
        }
        (self.action)(motion_master);
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedDelayedAction {
    pub action_type: MotionMasterDelayedActionType,
    pub executed: bool,
}

#[derive(Default)]
pub struct DelayedActionQueue<M> {
    actions: VecDeque<DelayedAction<M>>,
}

impl<M> DelayedActionQueue<M> {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    pub fn push(&mut self, action: DelayedAction<M>) {
        self.actions.push_back(action);
    }

    pub fn resolve_all(&mut self, motion_master: &mut M) -> Vec<ResolvedDelayedAction> {
        let mut resolved = Vec::new();
        while let Some(action) = self.actions.pop_front() {
            let action_type = action.action_type;
            let executed = action.resolve(motion_master);
            resolved.push(ResolvedDelayedAction {
                action_type,
                executed,
            });
        }
        resolved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn motion_master_flags_and_delayed_action_values_match_cpp() {
        assert_eq!(MotionMasterFlags::NONE.bits(), 0x0);
        assert_eq!(MotionMasterFlags::UPDATE.bits(), 0x1);
        assert_eq!(MotionMasterFlags::STATIC_INITIALIZATION_PENDING.bits(), 0x2);
        assert_eq!(MotionMasterFlags::INITIALIZATION_PENDING.bits(), 0x4);
        assert_eq!(MotionMasterFlags::INITIALIZING.bits(), 0x8);
        assert_eq!(
            MotionMasterFlags::DELAYED,
            MotionMasterFlags::UPDATE | MotionMasterFlags::INITIALIZATION_PENDING
        );

        assert_eq!(MotionMasterDelayedActionType::Clear.trinity_id(), 0);
        assert_eq!(MotionMasterDelayedActionType::ClearSlot.trinity_id(), 1);
        assert_eq!(MotionMasterDelayedActionType::ClearMode.trinity_id(), 2);
        assert_eq!(MotionMasterDelayedActionType::ClearPriority.trinity_id(), 3);
        assert_eq!(MotionMasterDelayedActionType::Add.trinity_id(), 4);
        assert_eq!(MotionMasterDelayedActionType::Remove.trinity_id(), 5);
        assert_eq!(MotionMasterDelayedActionType::RemoveType.trinity_id(), 6);
        assert_eq!(MotionMasterDelayedActionType::Initialize.trinity_id(), 7);
        assert_eq!(MotionMasterDelayedActionType::from_trinity_id(8), None);
    }

    #[test]
    fn delayed_action_queue_resolves_fifo_and_honors_validator() {
        let mut queue = DelayedActionQueue::default();
        queue.push(DelayedAction::new(
            MotionMasterDelayedActionType::Add,
            |value: &mut Vec<u8>| {
                value.push(1);
            },
        ));
        queue.push(DelayedAction::with_validator(
            MotionMasterDelayedActionType::Remove,
            |value: &mut Vec<u8>| value.push(2),
            || false,
        ));
        queue.push(DelayedAction::new(
            MotionMasterDelayedActionType::Initialize,
            |value: &mut Vec<u8>| value.push(3),
        ));

        let mut value = Vec::new();
        let resolved = queue.resolve_all(&mut value);
        assert_eq!(value, vec![1, 3]);
        assert!(queue.is_empty());
        assert_eq!(
            resolved,
            vec![
                ResolvedDelayedAction {
                    action_type: MotionMasterDelayedActionType::Add,
                    executed: true,
                },
                ResolvedDelayedAction {
                    action_type: MotionMasterDelayedActionType::Remove,
                    executed: false,
                },
                ResolvedDelayedAction {
                    action_type: MotionMasterDelayedActionType::Initialize,
                    executed: true,
                },
            ]
        );
    }
}
