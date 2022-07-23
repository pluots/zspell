//! # Zspell library
//!
//! The main operations of this module include [`AffixConfig`], which represents
//! configuration options, and [`Dictionary`], which contains the affix
//! configuration and the implementation to perform spell checking.
//!
//! Please note that the spellchecker is currently in alpha, and really not
//! ready for any mainstream use. Contributions are more than welcome at,
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
    // Below items are from "restriction"
    clippy::missing_docs_in_private_items,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::implicit_return,
    clippy::wildcard_enum_match_arm,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::exhaustive_structs,
    clippy::self_named_module_files
)]

pub mod affix;
pub mod dictionary;
pub mod errors;
pub mod system;

mod helpers;

pub use affix::Config;
pub use dictionary::Dictionary;
