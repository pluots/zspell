//! # Algorithms module
//!
//! This module includes [`modjaccard`] and [`modlevenshtein`], which contain
//! the various implementations for Levenshthein and Hamming distance, as well
//! as the Jaccard index. See these modules for in-depth explanation of
//!
//! Functions can be directly imported from this alrogithms module, no need to
//! access them via [`modjaccard`] or [`modlevenshtein`] (see the example
//! below).
//!
//! ## Example
//!
//! ```
//! use stringmetrics::algorithms::levenshtein;
//! let a = "this is a book";
//! let b = "i am a cook";
//! assert_eq!(levenshtein(a, b), 6);
//! ```

mod modbasic;
// mod damerau;
pub mod modjaccard;
pub mod modlevenshtein;

pub use self::modbasic::hamming;
// pub use self::damerau::damerau_levenshtein;
pub use self::modjaccard::{jaccard, jaccard_set};
pub use self::modlevenshtein::{
    levenshtein, levenshtein_limit, levenshtein_limit_weight, levenshtein_limit_weight_iter,
    levenshtein_weight,
};
