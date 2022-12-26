use std::rc::Rc;

use hashbrown::Equivalent;

use super::parser::PersonalMeta;
use crate::affix::types::{AffixRule, MorphInfo};

/// Extra meta information about where a word came from
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Source {
    /// this meta came from an affix and has a full affix rule
    Affix(Box<AffixRule>),
    /// this meta came from a .dic file, only contains morphinfo
    Dict(Option<Box<MorphInfo>>),
    /// this meta came from the personal dictionary
    Personal(Box<PersonalMeta>),
    /// The source is a raw text file with no additional metadata
    Raw,
}

// We will re-add this, but in the form of a RC to a stem and RC to an extra

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Extra {
    stem: Rc<String>,
    source: Rc<Source>,
}

impl Extra {
    pub(crate) fn new(stem_rc: Rc<String>, source_rc: Rc<Source>) -> Self {
        Self {
            stem: stem_rc,
            source: source_rc,
        }
    }
}

/// Clone of [`Source`] for quick construction and Eq comparison
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum SourceBorrowed<'a> {
    Affix(&'a AffixRule),
    Dict(Option<&'a MorphInfo>),
    Personal {
        friend: Option<&'a String>,
        morph: &'a Vec<MorphInfo>,
    },
}

/// Clone of [`Extra`] for quick construction and Eq comparison
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ExtraBorrowed<'a> {
    stem: &'a str,
    source: SourceBorrowed<'a>,
}

impl<'a> SourceBorrowed<'a> {
    pub(crate) fn new_personal(friend: Option<&'a String>, morph: &'a Vec<MorphInfo>) -> Self {
        Self::Personal { friend, morph }
    }
}

impl<'a> ExtraBorrowed<'a> {
    //     pub(crate) fn new_dict(stem: &str, morph: Option<&'a MorphInfo>) -> Self {
    //         Self {
    //             stem: stem.to_owned(),
    //             source: MetaInfo::Dict(morph),
    //         }
    // }

    //     pub(crate) fn new_afx(stem: &str, rule: &'a AffixRule) -> Self {
    //         Self {
    //             stem: stem.to_owned(),
    //             source: MetaInfo::Affix(rule),
    //         }
    //     }

    // pub(crate) fn new_personal(
    //     stem: &'a str,
    //     friend: Option<&'a String>,
    //     morph: &'a Vec<MorphInfo>,
    // ) -> Self {
    //     Self {
    //         stem: stem,
    //         source: SourceBorrowed::Personal { friend, morph },
    //     }
    // }

    // pub(crate) fn to_owned(&self) -> Extra {
    //     Extra {
    //         stem: self.stem.to_owned(),
    //         source: self.source.to_owned(),
    //     }
    // }
}

impl<'a> SourceBorrowed<'a> {
    pub(crate) fn to_owned(&self) -> Source {
        match self {
            SourceBorrowed::Affix(rule) => Source::Affix(Box::new((*rule).clone())),
            SourceBorrowed::Dict(morph_opt) => Source::Dict(morph_opt.map(|x| Box::new(x.clone()))),
            SourceBorrowed::Personal { friend, morph } => {
                Source::Personal(Box::new(PersonalMeta::new(*friend, (*morph).clone())))
            }
        }
    }
}

impl<'a> PartialEq<Source> for SourceBorrowed<'a> {
    fn eq(&self, other: &Source) -> bool {
        match (self, other) {
            (Self::Affix(l0), Source::Affix(r0)) => l0 == &r0.as_ref(),
            (Self::Dict(l0), Source::Dict(r0)) => l0 == &r0.as_ref().map(AsRef::as_ref),
            (Self::Personal { friend, morph }, Source::Personal(r0)) => {
                *friend == r0.friend.as_ref() && *morph == &r0.morph
            }
            _ => false,
        }
    }
}

impl<'a> PartialEq<SourceBorrowed<'a>> for Source {
    fn eq(&self, other: &SourceBorrowed) -> bool {
        other == self
    }
}

// impl<'a> PartialEq<Extra> for ExtraBorrowed<'a> {
//     fn eq(&self, other: &Extra) -> bool {
//         self.stem == other.stem && self.source == other.source
//     }
// }
// impl<'a> PartialEq<ExtraBorrowed<'a>> for Extra {
//     fn eq(&self, other: &ExtraBorrowed) -> bool {
//         self.stem == other.stem && self.source == other.source
//     }
// }

impl Equivalent<Rc<Source>> for SourceBorrowed<'_> {
    fn equivalent(&self, key: &Rc<Source>) -> bool {
        self == &**key
    }
}

#[cfg(test)]
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
