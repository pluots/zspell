use crate::affix::types::{AffixRule, MorphInfo};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MetaInfo<'a> {
    /// this meta came from an affix and has a full affix rule
    Affix(&'a AffixRule),
    /// this meta came from a .dic file, only contains morphinfo
    Dict(Option<&'a MorphInfo>),
    /// this meta came from the personal dictionary
    Personal,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Meta<'a> {
    stem: String,
    source: MetaInfo<'a>,
}

impl Meta<'_> {
    pub(crate) fn new_dict(stem: &str, morph: Option<&MorphInfo>) -> Self {
        Self {
            stem: stem.to_owned(),
            source: MetaInfo::Dict(morph),
        }
    }

    pub(crate) fn new_afx(stem: &str, rule: &AffixRule) -> Self {
        Self {
            stem: stem.to_owned(),
            source: MetaInfo::Affix(rule),
        }
    }
}
