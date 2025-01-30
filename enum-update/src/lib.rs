#![warn(missing_docs)]
//! This crate provides several derive macros that help with representing
//! changes to structs using enums.
//!
//! For api documentation, see the [`EnumUpdate`](macro@EnumUpdate) documentation. Otherwise,
//! here is an example of two threads using the generated enum type to keep
//! their state in sync.
//!
//! ```rust
//! # use enum_update::{ EnumUpdate, EnumUpdateSetters };
//! #[derive(Debug, EnumUpdate, Clone, EnumUpdateSetters)]
//! #[enum_update(derive(Debug, Clone, PartialEq))]
//! pub struct SharedState {
//!     value: String,
//! }
//! let mut thread_a_state = SharedState { value: "Hello".to_string() };
//! let mut thread_b_state = thread_a_state.clone();
//!
//! let (sender, receiver) = std::sync::mpsc::sync_channel(1);
//! let thread_a = std::thread::Builder::new()
//!     .spawn(move || {
//!         let change = thread_a_state.modify_value("World".to_string());
//!         sender.send(change).unwrap();
//!     })
//!     .unwrap();
//! let thread_b = std::thread::Builder::new()
//!     .spawn(move || {
//!         assert_eq!(thread_b_state.value, "H
//! ello".to_string());
//!         // now, we receive the change
//!         let change = receiver.recv().unwrap();
//!         assert_eq!(change, SharedStateUpdate::Value("World".to_string()));
//!         // applying the change
//!         thread_b_state.apply(change);
//!         // it becomes true
//!         assert_eq!(thread_b_state.value, "World".to_string());
//!     })
//!     .unwrap();
//! ```

pub use enum_update_derive::{EnumUpdate, EnumUpdateSetters};

/// Implemented on structs that have their updates represented by
/// some enum `U`. Implement this trait using the derive
/// macro [`EnumUpdate`].
pub trait EnumUpdate<U> {
    /// Apply the given update and mutate the state.
    fn apply(&mut self, update: U);
}
