pub mod spline;

pub use spline::{
    AnimTierTransition, FacingInfo, MonsterMoveType, MoveSpline, MoveSplineFlag,
    MoveSplineInitArgs, MoveSplineValidationError, SpellEffectExtraData, SplineUpdateResult,
    compute_fall_elevation, compute_fall_time,
};
