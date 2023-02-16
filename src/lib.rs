#![warn(missing_docs)]

/*!
*/

/// [`DualHashSet`] and adapters
pub mod hash;
pub use hash::DualHashSet;

/// A value that contains its own key
pub trait Dual {
    /// The key type
    type Key: Clone + Eq;
    /// Get a reference to the key
    fn key(&self) -> &Self::Key;
}
