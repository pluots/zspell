use crate::affix::types::{AffixRule, MorphInfo};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MetaInfo {
    Affix(AffixRule),
    Dict(MorphInfo),
    Personal,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Meta {
    stem: String,
    source: MetaInfo,
}
