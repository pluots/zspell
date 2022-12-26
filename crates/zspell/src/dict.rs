pub(crate) mod parser;
pub(crate) mod types;

use std::rc::Rc;

use hashbrown::hash_set::Iter as HashSetIter;
use hashbrown::{HashMap, HashSet};
use unicode_segmentation::UnicodeSegmentation;

use self::parser::{parse_dict, parse_personal_dict, DictEntry, PersonalEntry, PersonalMeta};
use self::types::{Extra, ExtraBorrowed, Source, SourceBorrowed};
use crate::error::{BuildError, Error};
use crate::Config;

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
    /// Meta information about how the wordlists were built and what the source was
    sources: HashSet<Rc<Source>>,
    /// A list of all stem words
    stems: HashSet<Rc<String>>,
    /// Affix configuration file. This will also hold references where our `meta`
    /// object points
    // FIXME: we don't need to store the whole `Config` here. It would be better
    // to replace with information that is relevant
    config: Box<Config>,
}

impl Dictionary {
    /// Create a new empty dictionary with default config
    fn new() -> Self {
        Self {
            wordlist: WordList::new(),
            wordlist_nosuggest: WordList::new(),
            wordlist_forbidden: WordList::new(),
            sources: HashSet::new(),
            stems: HashSet::new(),
            config: Box::default(),
        }
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

    pub(crate) fn wordlist(&self) -> &WordList {
        &self.wordlist
    }

    pub(crate) fn wordlist_forbidden(&self) -> &WordList {
        &self.wordlist_forbidden
    }

    pub(crate) fn wordlist_nosuggest(&self) -> &WordList {
        &self.wordlist_nosuggest
    }

    /// Update the internal wordlist and forbidden wordlist from a dictionary
    /// file string
    fn parse_update_wordlist(&mut self, source: &str) -> Result<(), Error> {
        let entries = parse_dict(source)?;
        self.update_wordlist(&entries)
    }

    /// Update internal wordlists from dictionary entries
    fn update_wordlist(&mut self, entries: &[DictEntry]) -> Result<(), Error> {
        self.config.validate_entry_flags(entries)?;
        // use baseline 3 words per line entry
        self.wordlist.0.reserve(entries.len() * 3);

        for entry in entries {
            let DictEntry { stem, flags, morph } = entry;
            let afx_words = self.config.create_affixed_words(stem, flags);
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
                let flags = dict.iter().find(|d| &d.stem == friend).map(|d| &d.flags);
                todo!()
            } else {
                // Select the correct word to work with
                let map = if entry.forbid {
                    &mut self.wordlist_forbidden.0
                } else {
                    &mut self.wordlist.0
                };

                let stem_rc = self
                    .stems
                    .get_or_insert_with(&entry.stem, |stem| Rc::new(stem.to_string()))
                    .clone();
                let source_comp = SourceBorrowed::new_personal(entry.friend.as_ref(), &entry.morph);
                let source_rc = self
                    .sources
                    .get_or_insert_with(&source_comp, |scomp| Rc::new(scomp.to_owned()))
                    .clone();
                let extra = Extra::new(stem_rc, source_rc);
                // Add our word, update its meta
                let extra_vec = map.entry_ref(&entry.stem).or_insert_with(Vec::new);
                extra_vec.push(extra);
            }
        }
        Ok(())
    }
}

/// The internal representation of a wordlist
#[derive(Clone, Debug, PartialEq)]
pub struct WordList(HashMap<String, Vec<Extra>>);

impl WordList {
    fn new() -> Self {
        Self(HashMap::new())
    }
}

///
#[derive(Clone, Debug, PartialEq)]
pub struct DictBuilder<'a> {
    cfg: Option<Config>,
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
    pub fn config_str(mut self, s: &'a str) -> Self {
        self.cfg_src = Some(s);
        self
    }

    /// Use instead of `config_str` if you have a preexisting `Config` type
    ///
    /// Don't use with `config_src`
    #[inline]
    pub fn config(mut self, cfg: Config) -> Self {
        self.cfg = Some(cfg);
        self
    }

    #[inline]
    pub fn dict_str(mut self, s: &'a str) -> Self {
        self.dict_src = Some(s);
        self
    }

    #[inline]
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
    #[inline]
    pub fn build(self) -> Result<Dictionary, Error> {
        if self.cfg.is_some() && self.cfg_src.is_some() {
            return Err(Error::Build(BuildError::CfgSpecTwice));
        }

        let cfg = if let Some(c) = self.cfg {
            c
        } else if let Some(cs) = self.cfg_src {
            Config::load_from_str(cs)?
        } else {
            return Err(Error::Build(BuildError::CfgUnspecified));
        };

        let mut dict = Dictionary::new();
        *dict.config = cfg;

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
