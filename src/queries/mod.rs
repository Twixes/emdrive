use std::hash::Hash;

mod sql;
mod errors;
mod component_types;
mod statement_types;

/// Trait for calculating distances used in the BK tree structure.
pub trait Distancable<D>: Eq + Ord + Copy + Hash
where
    D: num::Num + Eq + Ord + Copy + Hash,
{
    fn distance(&self, other: &Self) -> D;
}
