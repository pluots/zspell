use crate::affix::types::{AffixRule, MorphInfo};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MetaInfo {
    /// this meta came from an affix and has a full affix rule
    Affix(AffixRule),
    /// this meta came from a .dic file, only contains morphinfo
    Dict(MorphInfo),
    /// this meta came from the personal dictionary
    Personal,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Meta {
    stem: String,
    source: MetaInfo,
}
