#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub use enum_update_derive::{ EnumUpdate, EnumUpdateSetters };

/// Implemented on structs that have their updates represented by
/// some enum `U`. Implement this trait using the derive
/// macro [`EnumUpdate`].
pub trait EnumUpdate<U> {
    /// Apply the given update and mutate the state.
    fn apply(&mut self, update: U);
}