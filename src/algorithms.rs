//! This module contains functions for applying various closeness algorithms.
//!
//! See the indiviual functions for usage and examples.

mod basic;
mod damerau;
mod jaccard;
mod levenshtein;

pub use self::basic::hamming;
pub use self::damerau::damerau_levenshtein;
pub use self::jaccard::{jaccard, jaccard_iter};
pub use self::levenshtein::{
    levenshtein, levenshtein_limit, levenshtein_limit_weight, levenshtein_weight,
};
