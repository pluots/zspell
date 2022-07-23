//! # Zspell library
//!
//! The main operations of this module include [`AffixConfig`], which represents
//! configuration options, and [`Dictionary`], which contains the affix
//! configuration and the implementation to perform spell checking.
//!
//! Please note that the spellchecker is currently in alpha, and really not
//! ready for any mainstream use. Contributions are more than welcome at,
//! <https://github.com/pluots/zspell>.
#![warn(clippy::pedantic, clippy::cargo)]
// Pedantic config
#![allow(clippy::match_same_arms)]

pub mod affix;
pub mod dictionary;
pub mod errors;
pub mod system;

mod helpers;

pub use affix::AffixConfig;
pub use dictionary::Dictionary;
