//! A module for applying various closeness algorithms

mod basic;
mod damerau;
mod jaccard;
mod levenshtein;

pub use self::basic::hamming;
pub use self::damerau::damerau_levenshtein;
pub use self::jaccard::{jaccard, jaccard_iter};
pub use self::levenshtein::{levenshtein, levenshtein_limit};
