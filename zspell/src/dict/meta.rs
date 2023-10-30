use std::borrow::Borrow;
use std::sync::Arc;

use super::rule::AfxRule;
use crate::morph::MorphInfo;

/// Additional information attached to an entry in a dictionary
///
/// Cheaply cloneable
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Meta {
    stem: Arc<str>,
    source: Source,
}

impl Meta {
    pub(crate) fn new(stem_rc: Arc<str>, source: Source) -> Self {
        Self {
            stem: stem_rc,
            source,
        }
    }

    /// Return the stem of a word. Prefers the stem from the morph info if it is available
    pub fn stem(&self) -> &str {
        // If we have a dictionary source, check if we have a stem-type `MorphInfo`
        // and return it
        if let Source::Dict(morphvec) = &self.source {
            if let Some(stem) = morphvec.iter().find_map(|morph| {
                if let MorphInfo::Stem(st) = morph.borrow() {
                    Some(st)
                } else {
                    None
                }
            }) {
                return stem.as_ref();
            }
        }

        &self.stem
    }

    pub fn source(&self) -> &Source {
        &self.source
    }
}

/// Source information
#[allow(clippy::box_collection)]
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Source {
    /// This meta came from an affix and has a full affix rule
    Affix {
        /// The full rule that created this
        rule: Arc<AfxRule>,
        /// Index of the relevant pattern within the rule. This could potentially be a reference
        /// but that might require a RefCell, and I don't want to risk reference
        pat_idx: usize,
    },
    /// This meta came from a .dic file, only contains morphinfo
    Dict(Arc<[Arc<MorphInfo>]>),
    /// This meta came from the personal dictionary
    Personal(Arc<PersonalMeta>),
    /// The source is a raw text file with no additional metadata
    Raw,
}

impl Source {
    /// Iterate through all morph info available
    pub fn morphs(&self) -> impl Iterator<Item = &MorphInfo> {
        match self {
            Source::Affix { rule, pat_idx } => rule.patterns()[*pat_idx].morph_info(),
            Source::Dict(v) => v.as_ref(),
            Source::Personal(v) => v.morph.as_ref(),
            Source::Raw => &[],
        }
        .iter()
        .map(AsRef::as_ref)
    }

    /// Helper to create an `Affix` source when the `Arc` already exists
    pub(crate) fn new_affix(rule: &Arc<AfxRule>, pat_idx: usize) -> Self {
        Self::Affix {
            rule: Arc::clone(rule),
            pat_idx,
        }
    }
}

/// Representation of meta info for a personal dictionary
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct PersonalMeta {
    friend: Option<Arc<str>>,
    morph: Vec<Arc<MorphInfo>>,
}

impl PersonalMeta {
    pub fn new(friend: Option<Arc<str>>, morph: Vec<Arc<MorphInfo>>) -> Self {
        Self { friend, morph }
    }
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    use super::*;

    fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }
}
