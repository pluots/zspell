use std::fmt::{self, Display};
use std::sync::Arc;

use super::rule::AfxRule;

/// A flag representation is either an ASCII char, unicode char, or number. We can fit
/// any of those in a u32.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Flag(pub u32);

impl Flag {
    pub fn new_ascii(ch: u8) -> Self {
        debug_assert!(ch.is_ascii());
        Self(ch.into())
    }

    pub fn new_utf8(ch: char) -> Self {
        Self(ch.into())
    }

    /// Must be a 2-character string
    pub fn new_long(s: &str) -> Self {
        debug_assert!(s.len() == 2, "invalid string length: {s}");
        debug_assert!(
            s.chars().all(|ch| ch.is_ascii()),
            "invalid string characters: {s}"
        );

        let num = u16::from_le_bytes(s[..=1].as_bytes().try_into().unwrap());

        Self(num.into())
    }

    pub fn new_number(num: u32) -> Self {
        Self(num)
    }
}

impl fmt::Debug for Flag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(single_flag) = u8::try_from(self.0) {
            write!(f, "{}", char::from(single_flag))
        } else if let Ok(long_flag) = u16::try_from(self.0) {
            let [a, b] = long_flag.to_le_bytes();
            write!(f, "{}{}", char::from(a), char::from(b))
        } else {
            write!(f, "{:#06x}", self.0)
        }
    }
}

/// A representation of a flag value
#[non_exhaustive]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum FlagValue {
    // LemmaPresent and PseudoRoot are missing as they are deprecated
    AfxCircumfix,
    AfxKeepCase,
    AfxNeeded,
    AfxPseudoRoot,
    AfxSubstandard,
    Compound,
    CompoundBegin,
    CompoundEnd,
    CompoundForbid,
    CompoundForceUp,
    CompoundMiddle,
    CompoundOnly,
    CompoundPermit,
    CompoundRoot,
    ForbiddenWord,
    NoSuggest,
    WarnRare,
    /// Special case
    Rule(Arc<AfxRule>),
}

impl Display for FlagValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlagValue::AfxCircumfix => write!(f, "AfxCircumfix"),
            FlagValue::AfxKeepCase => write!(f, "AfxKeepCase"),
            FlagValue::AfxNeeded => write!(f, "AfxNeeded"),
            FlagValue::AfxPseudoRoot => write!(f, "AfxPseudoRoot"),
            FlagValue::AfxSubstandard => write!(f, "AfxSubstandard"),
            FlagValue::Compound => write!(f, "Compound"),
            FlagValue::CompoundBegin => write!(f, "CompoundBegin"),
            FlagValue::CompoundEnd => write!(f, "CompoundEnd"),
            FlagValue::CompoundForbid => write!(f, "CompoundForbid"),
            FlagValue::CompoundForceUp => write!(f, "CompoundForceUp"),
            FlagValue::CompoundMiddle => write!(f, "CompoundMiddle"),
            FlagValue::CompoundOnly => write!(f, "CompoundOnly"),
            FlagValue::CompoundPermit => write!(f, "CompoundPermit"),
            FlagValue::CompoundRoot => write!(f, "CompoundRoot"),
            FlagValue::ForbiddenWord => write!(f, "ForbiddenWord"),
            FlagValue::NoSuggest => write!(f, "NoSuggest"),
            FlagValue::WarnRare => write!(f, "WarnRare"),
            FlagValue::Rule(_) => write!(f, "Rule"),
        }
    }
}
