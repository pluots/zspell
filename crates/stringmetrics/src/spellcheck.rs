//! # Spellcheck module
//!
//! The main operations of this module include [`Affix`], which represents
//! configuration options, and [`Dictionary`], which contains the affix
//! configuration and the implementation to perform spell checking.
//!
//! Please note that the spellchecker is currently in alpha, and really not
//! ready for any mainstream use. Contributions are more than welcome,
//! https://github.com/pluots/stringmetrics-rust.

/// Create a vector of unicode graphemes
/// Each &str within this array is a single unicode character, which
/// is composed of one to four 8-bit integers ("chars")
#[macro_export]
macro_rules! graph_vec {
    ($ex:expr) => {
        $ex.graphemes(true)
            .collect::<Vec<&str>>()
            .iter()
            .map(|s| s.to_string())
            .collect()
    };
}

mod affix_serde;
mod affix_types;

pub mod affix;
pub mod dictionary;

pub use affix::Affix;
pub use dictionary::Dictionary;
