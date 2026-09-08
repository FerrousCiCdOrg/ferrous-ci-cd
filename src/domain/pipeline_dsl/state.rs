//! Type-level markers that record whether a builder holds its required contents.
//!
//! These are never constructed. They exist only so that a builder which is
//! still missing something cannot be handed to the next stage of the DSL.

mod sealed {
    /// Prevents outside crates from adding builder states.
    pub trait Sealed {}
}

/// A builder state marker.
pub trait BuilderState: sealed::Sealed {}

/// The builder is still missing something it needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Incomplete;

/// The builder holds everything required to produce a valid value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ready;

impl sealed::Sealed for Incomplete {}
impl sealed::Sealed for Ready {}

impl BuilderState for Incomplete {}
impl BuilderState for Ready {}
