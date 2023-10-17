use std::fmt::Display;
use std::sync::Arc;

use super::rule::AfxRule;

/// A representation of what a flag represents
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
