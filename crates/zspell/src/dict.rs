//! Main datastructure module with entrypoints for checking

mod flags;
mod helpers;
mod parser;
mod rule;
mod types;

use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::rc::Rc;

use hashbrown::{HashMap, HashSet};
use unicode_segmentation::UnicodeSegmentation;

pub use self::flags::FlagValue;
use self::helpers::create_affixed_word_map;
pub use self::parser::DictEntry;
use self::parser::{parse_dict, parse_personal_dict, PersonalEntry};
pub use self::rule::AfxRule;
use self::types::{Meta, PersonalMeta, Source, SourceBorrowed};
use crate::affix::FlagType;
use crate::error::{BuildError, Error};
use crate::helpers::StrWrapper;
use crate::morph::MorphInfo;
use crate::ParsedCfg;

/// Main dictionary object used for spellchecking and suggestions
///
/// Internally, this is represented as the following:
///
/// - A main wordlist
/// - A list of words to accept byt never suggest
/// - A list of words that are usually allowed but are forbidden by a personal
///   dictionary
/// - A list of stem words and source information
/// - Configuration information
///
/// The easiest way to construct a dictionary is using a [`DictBuilder`].
#[derive(Clone, Debug, PartialEq)]
pub struct Dictionary {
    /// General word list of words that are accepted and suggested. Note that it
    /// may make sense in the future to include non-suggest words here too.
    wordlist: WordList,
    /// Words to accept but never suggest
    wordlist_nosuggest: WordList,
    /// Words forbidden by the personal dictionary, i.e. do not accept as correct
    wordlist_forbidden: WordList,

    /* the following few types are used to store  meta information */
    /// A list of all stem words
    stems: HashSet<Rc<String>>,
    /// Affix flags and rules
    flags: BTreeMap<u32, FlagValue>,
    /// Possible morphs
    morphs: HashSet<Rc<MorphInfo>>,
    /// Type of flags to expect in our file
    flag_type: FlagType,
    /// Affix configuration file. This will also hold references where our `meta`
    /// object points
    // FIXME: we don't need to store the whole `Config` here. It would be better
    // to replace with information that is relevant
    parsed_config: Box<ParsedCfg>,
}

/// Check API
impl Dictionary {
    /// Create a new empty dictionary with default config
    fn new(cfg: ParsedCfg) -> Result<Self, Error> {
        Ok(Self {
            wordlist: WordList::new(),
            wordlist_nosuggest: WordList::new(),
            wordlist_forbidden: WordList::new(),
            stems: HashSet::new(),
            morphs: HashSet::new(),
            flags: cfg.compile_flags()?,
            flag_type: cfg.flag_type(),
            parsed_config: Box::new(cfg),
        })
    }

    /// Check that a string contains only words that are spelled correctly.
    /// Returns true if so
    ///
    /// # Example
    ///
    /// ```
    /// use std::fs;
    ///
    /// use zspell::DictBuilder;
    ///
    /// let aff_content = fs::read_to_string("tests/files/w1_eng_short.aff").unwrap();
    /// let dic_content = fs::read_to_string("tests/files/w1_eng_short.dic").unwrap();
    ///
    /// let dict = DictBuilder::new()
    ///     .config_str(&aff_content)
    ///     .dict_str(&dic_content)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(dict.check("reptiles pillow bananas"), true);
    /// assert_eq!(dict.check("pine missssspelled"), false);
    /// ```
    #[inline]
    pub fn check(&self, s: &str) -> bool {
        s.unicode_words().all(|w| self.check_word(w))
    }

    /// Check words in a string, returning a list of the start and end indices
    /// of any incorrect words
    ///
    /// This can be used ot create
    #[inline]
    pub fn check_indices(&self, s: &str) -> Vec<(usize, usize)> {
        s.split_word_bound_indices()
            .filter(|(idx, w)| !self.check_word(w))
            .map(|(idx, w)| (idx, idx + w.len()))
            .collect()
    }

    /// Check that a single word is spelled correctly: returns `true` if so
    #[inline]
    pub fn check_word(&self, s: &str) -> bool {
        // FIXME: we should make sure there are no overlaps among our wordlists
        let lower = s.to_lowercase();
        (!self.wordlist_forbidden.0.contains_key(s))
            && (self.wordlist.0.contains_key(s)
                || self.wordlist.0.contains_key(&lower)
                || self.wordlist_nosuggest.0.contains_key(s)
                || self.wordlist_nosuggest.0.contains_key(&lower))
    }
}

/// Internal config API
impl Dictionary {
    /// Create a vector of words from a single root word by applying rules in
    /// this affix. Does not check if the flag is valid.
    ///
    /// May contain duplicates, does not contain the original word
    ///
    /// Return type is vector of `(new_word, rule, second_rule)` where
    /// `second_rule` is available if both a prefix and a suffix were applied
    // PERF: benchmark taking a vec reference instead of returning
    fn create_affixed_words<'a>(&'a mut self, stem: &str, flags: &[u32]) {
        let mut pfx_rules = Vec::new();
        let mut sfx_rules = Vec::new();

        let stem_rc: &Rc<String> = self
            .stems
            .get_or_insert_with(&StrWrapper::new(stem), |sw: &StrWrapper| {
                Rc::new(sw.to_string())
            });

        for flag in flags {
            let mut forbid = false;
            let mut nosuggest = false;

            match self.flags.get(flag).unwrap().borrow() {
                FlagValue::ForbiddenWord => forbid = true,
                FlagValue::NoSuggest => nosuggest = true,
                FlagValue::Rule(rule) => {
                    if rule.is_pfx() {
                        pfx_rules.push(rule);
                    } else {
                        sfx_rules.push(rule);
                    }
                }
                _ => unimplemented!(),
            }

            // Forbid trumps nosuggest
            let map = if forbid {
                &mut self.wordlist_forbidden
            } else if nosuggest {
                &mut self.wordlist_nosuggest
            } else {
                &mut self.wordlist
            };

            create_affixed_word_map(&pfx_rules, &sfx_rules, stem, stem_rc, map);
            pfx_rules.clear();
            sfx_rules.clear();
        }
    }

    /// Update the internal wordlist and forbidden wordlist from a dictionary
    /// file string
    fn parse_update_wordlist(&mut self, source: &str) -> Result<(), Error> {
        let entries = parse_dict(source, self.flag_type)?;
        self.update_wordlist(&entries)
    }

    /// Update internal wordlists from dictionary entries
    fn update_wordlist(&mut self, entries: &[DictEntry]) -> Result<(), Error> {
        // use baseline 3 words per line entry
        self.wordlist.0.reserve(entries.len() * 3);

        // PERF: try moving flags outside of loop
        for entry in entries {
            let DictEntry { stem, flags, morph } = entry;
            let afx_words = self.create_affixed_words(stem, flags);

            // // Select the correct word to work with
            // let map = if entry.forbid {
            //     &mut self.wordlist_forbidden.0
            // } else {
            //     &mut self.wordlist.0
            // };

            // let stem_rc = self
            //     .stems
            //     .get_or_insert_with(&entry.stem, |stem| Rc::new(stem.to_string()))
            //     .clone();
            // let source_comp = SourceBorrowed::new_personal(entry.friend.as_ref(), &entry.morph);
            // let source_rc = self
            //     .sources
            //     .get_or_insert_with(&source_comp, |scomp| Rc::new(scomp.to_owned()))
            //     .clone();
            // let extra = Extra::new(stem_rc, source_rc);
            // // Add our word, update its meta
            // let extra_vec = map.entry_ref(&entry.stem).or_insert_with(Vec::new);
            // extra_vec.push(extra);
        }

        self.wordlist.0.shrink_to_fit();
        // todo!()
        Ok(())
    }

    fn parse_update_personal(&mut self, source: &str, dict: &[DictEntry]) -> Result<(), Error> {
        let entries = parse_personal_dict(source)?;
        self.update_personal(&entries, dict)
    }

    /// Must happen after `update_wordlist`
    #[allow(clippy::unnecessary_wraps)]
    fn update_personal(
        &mut self,
        entries: &[PersonalEntry],
        dict: &[DictEntry],
    ) -> Result<(), Error> {
        // FIXME: don't take `dict` as an argument, use our existing hashmaps
        self.wordlist.0.reserve(entries.len() * 2);
        for entry in entries {
            if let Some(friend) = &entry.friend {
                // Find the friend in our dictionary, find its source affixes
                // let flags = dict.iter().find(|d| &d.stem() == friend).map(|d| &d.flags);
                todo!()
            } else {
                let stem_rc: Rc<String> = self
                    .stems
                    .get_or_insert_with(&entry.stem, |stem| Rc::new(stem.to_string()))
                    .clone();

                let source = Source::Personal(Box::new(PersonalMeta::new(
                    None,
                    self.get_or_insert_morphs(&entry.morph),
                )));
                let meta = Meta::new(stem_rc, source);

                // Select the correct word to work with
                let hmap = if entry.forbid {
                    &mut self.wordlist_forbidden.0
                } else {
                    &mut self.wordlist.0
                };

                // Add our word, update its meta
                let extra_vec: &mut Vec<Meta> = hmap
                    .entry_ref(&entry.stem)
                    .or_insert_with(|| Vec::with_capacity(1));
                extra_vec.push(meta);
            }
        }
        Ok(())
    }

    /// For each morph in the slice: find it or insert it in our hashset, return
    /// a vector of references to the newly inserted (or found) items
    fn get_or_insert_morphs(&mut self, morphs: &[MorphInfo]) -> Vec<Rc<MorphInfo>> {
        let mut ret: Vec<Rc<MorphInfo>> = Vec::with_capacity(morphs.len());
        for morph in morphs {
            ret.push(
                self.morphs
                    .get_or_insert_with(morph, |m| Rc::new(m.clone()))
                    .clone(),
            );
        }
        ret
    }
}

/// The internal representation of a wordlist
///
/// Currently contains a `HashMap<String, Vec<Meta>>`
#[derive(Clone, Debug, PartialEq)]
pub struct WordList(HashMap<String, Vec<Meta>>);

impl WordList {
    fn new() -> Self {
        Self(HashMap::new())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DictBuilder<'a> {
    cfg: Option<ParsedCfg>,
    cfg_src: Option<&'a str>,
    dict_src: Option<&'a str>,
    personal_src: Option<&'a str>,
}

impl<'a> DictBuilder<'a> {
    /// Start a new `DictBuilder`
    #[inline]
    pub fn new() -> Self {
        Self {
            cfg: None,
            cfg_src: None,
            dict_src: None,
            personal_src: None,
        }
    }

    /// Load the affix file from the given string
    ///
    /// Don't use with `config`
    #[inline]
    #[must_use]
    pub fn config_str(mut self, s: &'a str) -> Self {
        self.cfg_src = Some(s);
        self
    }

    /// Use instead of `config_str` if you have a preexisting `Config` type
    ///
    /// Don't use with `config_src`
    #[inline]
    #[must_use]
    pub fn config(mut self, cfg: ParsedCfg) -> Self {
        self.cfg = Some(cfg);
        self
    }

    #[inline]
    #[must_use]
    pub fn dict_str(mut self, s: &'a str) -> Self {
        self.dict_src = Some(s);
        self
    }

    #[inline]
    #[must_use]
    pub fn personal_str(mut self, s: &'a str) -> Self {
        self.personal_src = Some(s);
        self
    }

    /// Consume this builder and return a `Dictionary`
    ///
    /// # Errors
    ///
    /// Returns an error if anything went wrong with parsing, or if the builder
    /// was in some way misconfigured.
    // PERF: parallize parsing of affix & dict files
    #[inline]
    pub fn build(self) -> Result<Dictionary, Error> {
        if self.cfg.is_some() && self.cfg_src.is_some() {
            return Err(Error::Build(BuildError::BuilderCfgSpecTwice));
        }

        let cfg = if let Some(c) = self.cfg {
            c
        } else if let Some(cs) = self.cfg_src {
            ParsedCfg::load_from_str(cs)?
        } else {
            return Err(Error::Build(BuildError::BuilderCfgUnspecified));
        };

        let mut dict = Dictionary::new(cfg)?;

        if let Some(wl) = self.dict_src {
            dict.parse_update_wordlist(wl)?;
        }

        if let Some(wl) = self.personal_src {
            dict.parse_update_personal(wl, &[])?;
        }

        Ok(dict)
    }
}

impl<'a> Default for DictBuilder<'a> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
