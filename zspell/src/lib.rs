//! ZSpell is a spellchecking tool written entirely in Rust, aimed to be
//! compatible with the widely-used [Hunspell] dictionary format. This is the
//! documentation for the system library, please see the [CLI docs] if that is
//! what you expected.
//!
//! # Usage
//!
//! A Hunspell dictionary format has three main components:
//!
//! - An "affix" or "config" file, usually with extension `.aff`
//! - A dictionary word list file, ususally `.dic` or `.dict`
//! - An optional personal dictionary
//!
//! You will need to know the location of dictionary files on your system, or
//! obtain them yourself. A repository exists that has dictionaries for many
//! different languages, if you don't have any available:
//! <https://github.com/wooorm/dictionaries>.
//!
//! This library requires specifying the input from these files, then building a
//! [`Dictionary`] object that can be used to perform all other operations.
//! Usage will typically look like the following:
//!
//! ```
//! # #![cfg(not(miri))]
//! use std::fs;
//!
//! use zspell::{DictBuilder, Dictionary};
//!
//! // This example just uses some shortened files. Load them to a string
//! let aff_content =
//!     fs::read_to_string("tests/files/w1_eng_short.aff").expect("failed to load config file");
//! let dic_content =
//!     fs::read_to_string("tests/files/w1_eng_short.dic").expect("failed to load wordlist file");
//!
//! // Use the builder pattern to create our `Dictionary` object
//! let dict: Dictionary = DictBuilder::new()
//!     .config_str(&aff_content)
//!     .dict_str(&dic_content)
//!     .build()
//!     .expect("failed to build dictionary!");
//!
//! // The `.check(&str)` method is useful for quickly verifying entire strings
//! assert_eq!(dict.check("reptiles pillow: bananas"), true);
//! assert_eq!(dict.check("well, I misspelled soemthing this tiem"), false);
//!
//! // Or use `.check_word(&str)` to validate the input as a single word
//! assert_eq!(dict.check_word("okay"), true);
//! assert_eq!(dict.check_word("okay okay"), false);
//!
//! // `.check_indices(&str)` provides more useful information for anything other than trivial
//! // checks. It returns an iterator over `(usize, &str)`, which gives the byte offset and
//! // string reference of any spelling errors.
//! let input = "okay, I misspelled soemthing this tiem";
//! let errors: Vec<(usize, &str)> = dict.check_indices(input).collect();
//! let expected = vec![(19, "soemthing"), (34, "tiem")];
//! assert_eq!(errors, expected);
//! ```
//!
//! There is also a powerful entry-based API that allows for stemming and analysis, as well as
//! suggestions (which are currently unstable).
//!
//! ```
//! # #![cfg(not(miri))]
//!
//! # use std::fs;
//! # use zspell::{DictBuilder, Dictionary};
//! # let aff_content =
//! #     fs::read_to_string("tests/files/w1_eng_short.aff").expect("failed to load config file");
//! # let dic_content =
//! #     fs::read_to_string("tests/files/w1_eng_short.dic").expect("failed to load wordlist file");
//! # let dict: Dictionary = DictBuilder::new()
//! #     .config_str(&aff_content)
//! #     .dict_str(&dic_content)
//! #     .build()
//! #     .expect("failed to build dictionary!");
//! let input = "bananas rusting";
//! let mut entries = dict.entries(input);
//!
//! // We can use the entry API to do the standard checks (word position and correctness),
//! // but also to find word roots.
//! let banana_entry = entries.next().unwrap();
//! let banana_stems: Vec<&str> = banana_entry.stems().unwrap().collect();
//! assert_eq!(banana_entry.word(), "bananas");
//! assert_eq!(banana_entry.index(), 0);
//! assert_eq!(banana_entry.correct(), true);
//! assert_eq!(banana_stems, ["bananas", "banana"]);
//!
//! let rust_entry = entries.next().unwrap();
//! let rust_stems: Vec<&str> = rust_entry.stems().unwrap().collect();
//! assert_eq!(rust_stems, ["rusting", "rust"]);
//! ```
//!
//! See [`Dictionary`] and [`DictBuilder`] to get started.
//!
//! # Stability & Feature Flags
//!
//! At the moment, the only public functions available are `check`,
//! `check_word`, and `check_indices`. These three functions are more or less
//! guaranteed to have stable interfaces, though the internals may change.
//!
//! There are also some unstable components to this library:
//!
//! - `unstable-suggestions`: Needed for providing suggestions, this is
//!   currently disabled because it is slow.
//! - `unstable-system`: Needed for system interfaces like locating existing
//!   dictionaries
//! - `zspell-unstable`: Enable all of these options
//!
//! These flags can be enabled in your `Cargo.toml` if you would like to
//! experiment with these featuers. Any APIs protected behind these feature
//! flags are subject to change, but the need for these flags will be removed as
//! they are stabalized.
//!
//! [Hunspell]: http://hunspell.github.io/
//! [CLI docs]: https://pluots.github.io/zspell/
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
// #![warn(clippy::cargo)]
#![warn(clippy::nursery)]
#![warn(clippy::str_to_string)]
#![warn(clippy::missing_inline_in_public_items)]
#![warn(clippy::disallowed_types)]
#![allow(clippy::use_self)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]
// #![allow(clippy::redundant_pub_crate)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::derive_partial_eq_without_eq)]

mod affix;
mod dict;
pub mod error;
mod helpers;
mod meta;
mod morph;
mod suggestions;

#[cfg(feature = "unstable-system")]
pub mod system;

pub(crate) use affix::ParsedCfg;
pub use affix::PartOfSpeech;
#[doc(inline)]
pub use dict::{DictBuilder, Dictionary, WordEntry, WordList};
#[doc(inline)]
pub use error::Error;
pub use morph::MorphInfo;

// Make some things public when benchmarking
#[cfg(feature = "unstable-bench")]
pub mod bench {
    pub use super::affix::{affix_from_str, FlagType};
    pub use super::dict::DictEntry;
}
