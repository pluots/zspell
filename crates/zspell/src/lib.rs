//! # Zspell library
//!
//! The main operations of this module include [`Config`], which represents
//! affix configuration options, and [`Dictionary`], which contains the
//! configuration and the implementation to perform spell checking.
//!
//! Please note that the spellchecker is currently in alpha, and really not yet
//! ready for any mainstream use. Contributions are more than welcome at
//! <https://github.com/pluots/zspell>.
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![warn(clippy::nursery)]
#![warn(clippy::str_to_string)]
#![warn(clippy::missing_inline_in_public_items)]
#![warn(clippy::disallowed_types)]
#![allow(clippy::use_self)] // disabled because strum doesn't enforce it
#![allow(clippy::match_same_arms)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::redundant_pub_crate)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(unused)]

mod affix;
mod dict;
pub mod error;
pub(crate) mod helpers;
mod meta;
mod parser_affix;
mod suggestions;
mod system;

pub(crate) use affix::Config;
pub use dict::{DictBuilder, Dictionary};
#[doc(inline)]
pub use error::Error;
