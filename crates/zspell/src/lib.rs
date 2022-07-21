//! # Zspell library
//!
//! The main operations of this module include [`AffixConfig`], which represents
//! configuration options, and [`Dictionary`], which contains the affix
//! configuration and the implementation to perform spell checking.
//!
//! Please note that the spellchecker is currently in alpha, and really not
//! ready for any mainstream use. Contributions are more than welcome at,
//! <https://github.com/pluots/zspell>.

pub mod affix;
pub mod dictionary;
pub mod errors;
mod helpers;

pub use affix::AffixConfig;
pub use dictionary::Dictionary;
