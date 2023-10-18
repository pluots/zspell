use std::borrow::Borrow;
use std::sync::Arc;

use itertools::Either;

use super::rule::AfxRule;
use crate::morph::MorphInfo;

/// Additional information attached to an entry in a dictionary
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
        // If we have a dictionary source, check if we have a stem `MorphInfo`
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
    /// this meta came from an affix and has a full affix rule
    Affix(Arc<AfxRule>),
    /// this meta came from a .dic file, only contains morphinfo
    Dict(Box<[Arc<MorphInfo>]>),
    /// this meta came from the personal dictionary
    /// String is the "friend" word
    Personal(Box<PersonalMeta>),
    /// The source is a raw text file with no additional metadata
    Raw,
}

impl Source {
    /// Iterate through all morph info available
    // https://github.com/rust-lang/rust-clippy/issues/11680
    #[allow(clippy::iter_on_empty_collections)]
    pub fn morphs(&self) -> impl Iterator<Item = &MorphInfo> {
        match self {
            Source::Affix(rule) => {
                let iter = rule
                    .patterns()
                    .iter()
                    .flat_map(|pat| pat.morph_info().iter());
                Either::Left(iter)
            }
            Source::Dict(v) => Either::Right(v.as_ref().iter()),
            Source::Personal(v) => Either::Right(v.morph.iter()),
            Source::Raw => Either::Right([].iter()),
        }
        .map(AsRef::as_ref)
    }
}

/// Representation of meta info for a personal dictionary
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

    // FIXME
    // #[test]
    // fn check_hashes() {
    //     // validate that our owned & borrowed types have the same hash
    //     let owned = Extra {
    //         stem: "abcd".to_string(),
    //         source: Source::Personal(Box::new(PersonalMeta {
    //             friend: Some("efgh".to_owned()),
    //             morph: vec![MorphInfo::DerivPfx("xyz".to_owned())],
    //         })),
    //     };

    //     let meta = match owned.source {
    //         Source::Personal(ref m) => m,
    //         _ => panic!(),
    //     };

    //     let borrowed = ExtraBorrowed {
    //         stem: &owned.stem,
    //         source: SourceBorrowed::Personal {
    //             friend: meta.friend.as_ref(),
    //             morph: &meta.morph,
    //         },
    //     };

    //     let h1 = calculate_hash(&owned);
    //     let h2 = calculate_hash(&borrowed);

    //     assert_eq!(h1, h2);
    //     assert_eq!(&owned, &borrowed);
    //     assert_eq!(owned, borrowed.to_owned());
    // }
}
