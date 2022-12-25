//! # Zspell library
//!
//! The main operations of this module include [`Config`], which represents
//! affix configuration options, and [`Dictionary`], which contains the
//! configuration and the implementation to perform spell checking.
//!
//! Please note that the spellchecker is currently in alpha, and really not
//! ready for any mainstream use. Contributions are more than welcome at
//! <https://github.com/pluots/zspell>.
#![warn(
    clippy::pedantic,
    clippy::cargo,
    clippy::nursery,
    clippy::str_to_string,
    clippy::missing_inline_in_public_items,
    // clippy::restriction,
    // clippy::exhaustive_enums,
    // clippy::pattern_type_mismatch,
)]
// Pedantic config
#![allow(
    clippy::match_same_arms,
    clippy::struct_excessive_bools,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::use_self, // disabled because strum doesn't enforce it
)]

pub mod affix;
pub mod dictionary;
pub mod errors;
pub mod system;

mod helpers;

pub use affix::Config;
pub use dictionary::Dictionary;
