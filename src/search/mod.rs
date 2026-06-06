mod v1;
mod v2;

pub use v1::matches as fuzzy_matches;
pub use v2::{best_matches, best_matches_by, matches_any};
