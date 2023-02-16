pub mod hash;

pub use hash::DualHashSet;

/// A value that contains its own key
pub trait Dual {
    type Key: Clone + Eq;
    fn key(&self) -> &Self::Key;
}
