//! Main datastructure module with entrypoints for checking

mod flags;
mod helpers;
mod parser;
mod rule;
mod types;

use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::sync::Arc;

use hashbrown::{HashMap, HashSet};
use stringmetrics::try_levenshtein;
use unicode_segmentation::UnicodeSegmentation;

pub use self::flags::FlagValue;
use self::helpers::{create_affixed_word_map, word_splitter};
pub use self::parser::DictEntry;
use self::parser::{parse_dict, parse_personal_dict, PersonalEntry};
pub use self::rule::AfxRule;
use self::types::{Meta, PersonalMeta, Source};
use crate::affix::FlagType;
use crate::error::{BuildError, Error, WordNotFoundError};
use crate::helpers::StrWrapper;
use crate::morph::MorphInfo;
use crate::{suggestions, ParsedCfg};

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
/// The easiest way to construct a dictionary is using a [`DictBuilder`]. You
/// can use this `Dictionary` object to perform various checks, likely via
/// [`check`][Dictionary::check] (for simple true/false checking of strings) or
/// [`check_indices`][Dictionary::check_indices] (to validate a string and
/// return the location of errors).
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
    stems: HashSet<Arc<String>>,
    /// Affix flags and rules
    flags: BTreeMap<u32, FlagValue>,
    /// Possible morphs
    morphs: HashSet<Arc<MorphInfo>>,
    /// Type of flags to expect in our file
    flag_type: FlagType,
    /// Affix configuration file. This will also hold references where our `meta`
    /// object points
    // FIXME: we don't need to store the whole `Config` here. It would be better
    // to replace with information that is relevant
    parsed_config: Box<ParsedCfg>,
}

// Check API
impl Dictionary {
    /// Create a new empty dictionary with default config
    #[inline]
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

    /// Check that an entire string contains only words that are spelled
    /// correctly, returns `true` if so.
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
    pub fn check(&self, input: &str) -> bool {
        input.unicode_words().all(|w| self.check_word(w))
    }

    /// Check that a single word is spelled correctly, returns `true` if so
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
    /// assert_eq!(dict.check_word("reptiles"), true);
    /// assert_eq!(dict.check_word("reptiles pillow"), false);
    /// ```
    #[inline]
    pub fn check_word(&self, word: &str) -> bool {
        // FIXME: we should make sure there are no overlaps among our wordlists
        let lower = word.to_lowercase();
        (!self.wordlist_forbidden.0.contains_key(word))
            && (self.wordlist.0.contains_key(word)
                || self.wordlist.0.contains_key(&lower)
                || self.wordlist_nosuggest.0.contains_key(word)
                || self.wordlist_nosuggest.0.contains_key(&lower))
    }

    /// Check words in a string, returning a list of the start and end indices
    /// of any incorrect words.
    ///
    /// This can be used ot create spellcheckers that provide feedback to a
    /// user.
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
    /// let errors: Vec<(usize, &str)> = dict.check_indices("pine missspelled").collect();
    /// assert_eq!(errors, vec![(5, "missspelled")]);
    /// ```
    ///
    /// The return signature is a bit clunky looking if you're not familiar with
    /// Rust, but I promise it's more simple than it looks
    /// 1. It returns an iterator so you can lazily iterate: `for (idx, wrongword)
    ///    in dict.check_indice(ssentence) {...}`
    /// 2. Lifetimes: the iterator itself can't outlive the `Dictionary` object
    ///    itself (both have lifetime `'d`) since it calls some internal
    ///    functions
    /// 3. Lifetimes 2: the strings in the returned iterator values can't
    ///    outlive the input string (both have lifetime `'a` since they're
    ///    just references to the input string)
    ///
    /// Still hitting lifetime errors? Just `.collect()` it into a vector like
    /// in the above example.
    #[inline]
    pub fn check_indices<'a: 'd, 'd>(
        &'d self,
        input: &'a str,
    ) -> impl Iterator<Item = (usize, &'a str)> + 'd {
        word_splitter(input).filter(|(idx, w)| !self.check_word(w))
    }

    /// **UNSTABLE** Suggest a word at given indices. Feature gated behind
    /// `unstable-suggestions`.
    #[inline]
    #[cfg(feature = "unstable-suggestions")]
    pub fn suggest_indices<'a>(
        &self,
        input: &'a str,
    ) -> impl Iterator<Item = (usize, &'a str, Vec<&str>)> {
        word_splitter(input).filter_map(|(idx, w)| {
            self.suggest_word(w)
                .map_or_else(|v| Some((idx, w, v)), |_| None)
        })
    }

    /// **UNSTABLE** Suggest a replacement for a single word. Feature gated
    /// behind `unstable-suggestions`.
    ///
    /// If the word exists, this will return `Ok(())`. If it does not, it will
    /// return a vector of suggestions `Err(Vec<&str>)`.
    ///
    /// This function is unstable because it has performance issues. We are
    /// going to try to speed up the algorithm significantly.
    // PERF: bench with par_iter
    #[inline]
    #[cfg(feature = "unstable-suggestions")]
    #[allow(clippy::missing_errors_doc)]
    pub fn suggest_word(&self, word: &str) -> Result<(), Vec<&str>> {
        if self.check_word(word) {
            return Ok(());
        }
        let mut suggestions: Vec<(u32, &String)> = self
            .wordlist
            .0
            .keys()
            .filter_map(|key| try_levenshtein(key, word, 1).map(|lim| (lim, key)))
            .collect();
        suggestions.sort_unstable_by_key(|(k, v)| *k);
        Err(suggestions
            .iter()
            .take(10)
            .map(|(k, v)| v.as_str())
            .collect())
    }

    /// **UNSTABLE** Generate the stems for a single word. Feature gated behind
    /// `unstable-stem`.
    ///
    /// If the word is found, this will return a vector of `&str` potential
    /// stems.
    ///
    /// # Errors
    ///
    /// Returns a dummy error if the word is not found
    #[inline]
    #[cfg(feature = "unstable-stem")]
    pub fn stem_word(&self, word: &str) -> Result<Vec<&str>, WordNotFoundError> {
        let Some(meta) = self.wordlist.0.get(word).or_else(|| self.wordlist_nosuggest.0.get(word)) else {
            return Err(WordNotFoundError);
        };

        let mut stems: Vec<&str> = Vec::with_capacity(meta.len());
        let mut morphs: Vec<&MorphInfo> = Vec::with_capacity(meta.len());
        for item in meta {
            item.source().push_morphs(&mut morphs);
            stems.push(item.stem());
        }

        for morph in morphs {
            if let MorphInfo::Stem(s) = morph {
                stems.push(s);
            }
        }

        Ok(stems)
    }

    /// **UNSTABLE** Generate the morphological analysis for a single word.
    /// Feature gated behind `unstable-analysis`.
    ///
    /// # Errors
    ///
    /// Returns a dummy error if the word is not found
    #[inline]
    #[cfg(feature = "unstable-analysis")]
    pub fn analyze_word(&self, word: &str) -> Result<Vec<MorphInfo>, WordNotFoundError> {
        todo!()
    }

    /// Return a reference to the internal wordlist
    #[inline]
    pub fn wordlist(&self) -> &WordList {
        &self.wordlist
    }

    /// Return a reference to the internal nosuggest wordlist
    #[inline]
    pub fn wordlist_nosuggest(&self) -> &WordList {
        &self.wordlist_nosuggest
    }

    /// Return a reference to the internal forbidden wordlist
    #[inline]
    pub fn wordlist_forbidden(&self) -> &WordList {
        &self.wordlist_forbidden
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
    // TODO: include morph data for generated words
    fn create_affixed_words(&mut self, stem: &str, flags: &[u32], _morph: &[MorphInfo]) {
        let mut prefix_rules = Vec::new();
        let mut suffix_rules = Vec::new();

        let stem_rc: &Arc<String> = self
            .stems
            .get_or_insert_with(&StrWrapper::new(stem), |sw: &StrWrapper| {
                Arc::new(sw.to_string())
            });

        let mut add_stem = true;
        let mut forbid = false;
        let mut nosuggest = false;

        for flag in flags {
            if self.flags.get(flag).is_none() {
                // FIXME: we get stuck on compound rules
                continue;
            }

            match self.flags.get(flag).unwrap().borrow() {
                FlagValue::ForbiddenWord => forbid = true,
                FlagValue::NoSuggest => nosuggest = true,
                FlagValue::Rule(rule) => {
                    if rule.is_pfx() {
                        prefix_rules.push(rule);
                    } else {
                        suffix_rules.push(rule);
                    }
                }
                FlagValue::AfxNeeded => add_stem = false,
                _ => {
                    // FIXME: should be unimplemented
                    // unimplemented!()
                    // eprintln!("unexpected flag {}", flag);
                }
            }
        }

        // Forbid trumps nosuggest
        let dest = if forbid {
            &mut self.wordlist_forbidden
        } else if nosuggest {
            &mut self.wordlist_nosuggest
        } else {
            &mut self.wordlist
        };

        if add_stem {
            // TODO: fix location for this, add morph
            let meta = Meta::new(stem_rc.clone(), Source::Dict(Box::default()));
            let meta_vec = dest.0.entry_ref(stem).or_insert_with(Vec::new);
            meta_vec.push(meta);
        }

        create_affixed_word_map(&prefix_rules, &suffix_rules, stem, stem_rc, dest);
        prefix_rules.clear();
        suffix_rules.clear();
    }

    /// Update the internal wordlist and forbidden wordlist from a dictionary
    /// file string
    fn parse_update_wordlist(&mut self, source: &str) -> Result<(), Error> {
        let entries = parse_dict(source, self.flag_type)?;
        self.update_wordlist(&entries)
    }

    /// Update internal wordlists from dictionary entries
    #[allow(clippy::unnecessary_wraps)]
    fn update_wordlist(&mut self, entries: &[DictEntry]) -> Result<(), Error> {
        // use baseline 3 words per line entry
        self.wordlist.0.reserve(entries.len() * 3);

        // PERF: try moving flags outside of loop
        for entry in entries {
            let DictEntry { stem, flags, morph } = entry;

            self.create_affixed_words(stem, flags, morph);
        }

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
                let stem_arc: Arc<String> = self
                    .stems
                    .get_or_insert_with(&entry.stem, |stem| Arc::new(stem.to_string()))
                    .clone();

                let source = Source::Personal(Box::new(PersonalMeta::new(
                    None,
                    self.get_or_insert_morphs(&entry.morph),
                )));
                let meta = Meta::new(stem_arc, source);

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
    fn get_or_insert_morphs(&mut self, morphs: &[MorphInfo]) -> Vec<Arc<MorphInfo>> {
        let mut ret: Vec<Arc<MorphInfo>> = Vec::with_capacity(morphs.len());
        for morph in morphs {
            ret.push(
                self.morphs
                    .get_or_insert_with(morph, |m| Arc::new(m.clone()))
                    .clone(),
            );
        }
        ret
    }

    /// Free as much memory as possible when we know we won't be using it anymore
    fn shrink_storage(&mut self) {
        self.wordlist.0.shrink_to_fit();
        self.wordlist_nosuggest.0.shrink_to_fit();
        self.wordlist_forbidden.0.shrink_to_fit();
        self.stems.shrink_to_fit();
        self.morphs.shrink_to_fit();
    }
}

/// The internal representation of a wordlist.
///
/// Currently contains a `HashMap<String, Vec<Meta>>`
#[derive(Clone, Debug, PartialEq)]
pub struct WordList(HashMap<String, Vec<Meta>>);

impl WordList {
    fn new() -> Self {
        Self(HashMap::new())
    }

    /// **UNSTABLE** Get a reference to the internal map. This is behind the
    /// `zspell-unstable` marker as the internal format may change
    #[cfg_attr(feature = "zspell-unstable", visibility::make(pub))]
    pub(crate) fn inner(&self) -> &HashMap<String, Vec<Meta>> {
        &self.0
    }
}

/// A builder stucture that is used to create a [`Dictionary`].
///
/// See module-level documentation for an example.
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

    /// Load the affix file from the given string.
    #[inline]
    #[must_use]
    pub fn config_str(mut self, config: &'a str) -> Self {
        self.cfg_src = Some(config);
        self
    }

    /// Use instead of `config_str` if you have a preexisting `Config` type
    ///
    /// Don't use with `config_src`
    #[inline]
    #[must_use]
    fn config(mut self, cfg: ParsedCfg) -> Self {
        self.cfg = Some(cfg);
        self
    }

    /// Load the dictionary file from a string
    #[inline]
    #[must_use]
    pub fn dict_str(mut self, dict: &'a str) -> Self {
        self.dict_src = Some(dict);
        self
    }

    /// Load a personal dictionary file from a string
    #[inline]
    #[must_use]
    pub fn personal_str(mut self, personal: &'a str) -> Self {
        self.personal_src = Some(personal);
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

        dict.shrink_storage();

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
