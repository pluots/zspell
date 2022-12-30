//! Various functions that are helpful throughout the crate

use core::fmt::Display;
use std::borrow::Borrow;
use std::hash::Hash;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use hashbrown::Equivalent;
use regex::Regex;

use crate::affix::RuleType;

/// Wrap `Regex` objects so they can be hashed
#[derive(Clone, Debug)]
pub struct ReWrapper(Regex);

impl ReWrapper {
    pub fn new(re: &str) -> Result<Self, regex::Error> {
        Ok(Self(Regex::new(re)?))
    }
}

impl Eq for ReWrapper {}

impl PartialEq for ReWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_str() == other.0.as_str()
    }
}

impl Hash for ReWrapper {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_str().hash(state);
    }
}

impl Deref for ReWrapper {
    type Target = Regex;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Convert any integer to a u32, panic if it does not fit
#[inline]
pub fn convertu32<T: TryInto<u32> + Display + Copy>(value: T) -> u32 {
    value
        .try_into()
        .unwrap_or_else(|_| panic!("value {value} overflows u32 max of {}", u32::MAX))
}

/// Compile a regex pattern in the context of an affix. Returns None if
/// the universal pattern "." is provided
pub fn compile_re_pattern(
    condition: &str,
    kind: RuleType,
) -> Result<Option<ReWrapper>, regex::Error> {
    if condition == "." {
        return Ok(None);
    }
    let re_pattern = match kind {
        RuleType::Prefix => format!("^{condition}.*$"),
        RuleType::Suffix => format!("^.*{condition}$"),
    };
    ReWrapper::new(re_pattern.as_str()).map(Some)
}

/// Implement a type that derefs to compare to a string
#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub struct StrWrapper<'a>(pub &'a str);

impl<'a> StrWrapper<'a> {
    pub fn new(s: &'a str) -> Self {
        Self(s)
    }
}

impl Display for StrWrapper<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Equivalent<Rc<String>> for StrWrapper<'_> {
    fn equivalent(&self, key: &Rc<String>) -> bool {
        self.0 == key.as_ref()
    }
}

impl Equivalent<Arc<String>> for StrWrapper<'_> {
    fn equivalent(&self, key: &Arc<String>) -> bool {
        self.0 == key.as_ref()
    }
}
