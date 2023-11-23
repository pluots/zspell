//! Take rules and qpply them to a word, trying to find a match in an
//! existing wordlist.
#![allow(unused)]

use crate::affix::CompoundConfig;
use crate::Dictionary;

/// Try to create a word
fn entrypoint(dict: &Dictionary, word: &str) -> bool {
    todo!()
}

fn try_strip_pfx() {
    todo!()
}

fn try_strip_sfx() {}

/// Try splitting the word at each position and testing the parts according to
/// compound rules
fn compound_thing(cfg: &CompoundConfig) {}
