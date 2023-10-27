//! Main datastructure module with entrypoints for checking

mod flags;
mod meta;
mod parse;
mod rule;
mod util;

use std::collections::BTreeMap;
use std::sync::Arc;

use hashbrown::{HashMap, HashSet};
use stringmetrics::try_levenshtein;
use unicode_segmentation::UnicodeSegmentation;

pub use self::flags::FlagValue;
use self::meta::{Meta, PersonalMeta, Source};
pub use self::parse::DictEntry;
use self::parse::PersonalEntry;
pub use self::rule::AfxRule;
use self::util::{create_affixed_word_map, word_splitter};
use crate::affix::FlagType;
use crate::error::{BuildError, Error};
use crate::helpers::StrWrapper;
use crate::morph::MorphInfo;
use crate::ParsedCfg;

/// Main dictionary object used for spellchecking, suggestions, and analysis.
///
/// Internally, this is represented as the following:
///
/// - A main wordlist
/// - A list of words to accept but never suggest, from the `NOSUGGEST` flag
/// - A list of words that are usually allowed but are forbidden by a personal
///   dictionary or the `FORBIDDENWORD` flag
/// - A list of stem words and source information
/// - Configuration information
///
/// The easiest way to construct a dictionary is using a [`DictBuilder`]. You
/// can use this `Dictionary` object to perform various checks, likely via
/// [`check`][Self::check] (for simple true/false checking of strings) or
/// [`check_indices`][Self::check_indices] (to validate a string and
/// return the location of errors).
///
/// More powerful use for things such as stemming, morphological analysis, or (unstable)
/// suggestions will want to use the entry API via [`entry`](Self::entry) or
/// [`entries`](Self::entries).
#[must_use]
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
    stems: HashSet<Arc<str>>,
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
    /// # #![cfg(not(miri))]
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
    /// # #![cfg(not(miri))]
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
                || self.wordlist.0.contains_key(lower.as_str())
                || self.wordlist_nosuggest.0.contains_key(word)
                || self.wordlist_nosuggest.0.contains_key(lower.as_str()))
    }

    /// Check words in a string, returning a list of the start and end indices
    /// of any incorrect words.
    ///
    /// This can be used ot create spellcheckers that provide feedback to a
    /// user.
    ///
    /// ```
    /// # #![cfg(not(miri))]
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
    /// Rust, but I promise it's more simple than it looks:
    /// 1. It returns an iterator so you can lazily iterate: `for (idx, wrongword)
    ///    in dict.check_indice(sentence) {...}`
    /// 2. Lifetimes: the iterator itself can't outlive the `Dictionary` object
    ///    itself (both have lifetime `'d`) since it calls some internal
    ///    functions
    /// 3. Lifetimes 2: the strings in the returned iterator values can't
    ///    outlive the input string (both have lifetime `'a` since the resturn values
    ///    are just references to the input string)
    ///
    /// Still hitting lifetime errors? Just `.collect()` it into a vector like
    /// in the above example.
    #[inline]
    pub fn check_indices<'a: 'd, 'd>(
        &'d self,
        input: &'a str,
    ) -> impl Iterator<Item = (usize, &'a str)> + 'd {
        word_splitter(input).filter(|(_idx, w)| !self.check_word(w))
    }

    /// Helper for `locate_word` that allows setting the index
    fn locate_word_inner<'d, 's>(&'d self, word: &'s str, index: usize) -> WordEntry<'d, 's> {
        let lower = word.to_lowercase();

        let ctx = if self.wordlist_forbidden.0.contains_key(word)
            || self.wordlist_forbidden.0.contains_key(lower.as_str())
        {
            WordCtx::Incorrect { forbidden: true }
        } else if let Some((matched, meta)) = self.wordlist.0.get_key_value(word) {
            WordCtx::Correct {
                matched,
                meta_list: meta,
            }
        } else if let Some((matched, meta)) = self.wordlist.0.get_key_value(lower.as_str()) {
            WordCtx::Correct {
                matched,
                meta_list: meta,
            }
        } else if let Some((matched, meta)) = self.wordlist_nosuggest.0.get_key_value(word) {
            WordCtx::Correct {
                matched,
                meta_list: meta,
            }
        } else if let Some((matched, meta)) =
            self.wordlist_nosuggest.0.get_key_value(lower.as_str())
        {
            WordCtx::Correct {
                matched,
                meta_list: meta,
            }
        } else {
            WordCtx::Incorrect { forbidden: false }
        };

        WordEntry {
            word,
            index,
            dict: self,
            context: ctx,
        }
    }

    /// Return an iterator over entries for each word in a sentence.
    ///
    /// This can be used to stem or analyze words that are spelled correctly, or provide
    /// suggestions for incorrect words. See [`WordEntry`] for more information.
    #[inline]
    pub fn entries<'d, 's>(&'d self, input: &'s str) -> impl Iterator<Item = WordEntry<'d, 's>> {
        word_splitter(input).map(|(idx, word)| self.locate_word_inner(word, idx))
    }

    /// Return an entry for a single word.
    ///
    /// This can be used to stem or analyze words that are spelled correctly, or provide
    /// suggestions for incorrect words. See [`WordEntry`] for more information.
    #[inline]
    pub fn entry<'d, 's>(&'d self, word: &'s str) -> WordEntry<'d, 's> {
        self.locate_word_inner(word, 0)
    }

    /// Return a reference to the internal wordlist
    #[inline]
    #[doc(hidden)]
    pub fn wordlist(&self) -> &WordList {
        &self.wordlist
    }

    /// Return a reference to the internal nosuggest wordlist
    #[inline]
    #[doc(hidden)]
    pub fn wordlist_nosuggest(&self) -> &WordList {
        &self.wordlist_nosuggest
    }

    /// Return a reference to the internal forbidden wordlist
    #[inline]
    #[doc(hidden)]
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
    fn create_affixed_words(&mut self, stem: &str, flags: &[u32], morph: &[Arc<MorphInfo>]) {
        let mut prefix_rules = Vec::new();
        let mut suffix_rules = Vec::new();

        let stem: &Arc<str> = self
            .stems
            .get_or_insert_with(&StrWrapper::new(stem), |sw: &StrWrapper| Arc::from(sw.0));

        let mut add_stem = true;
        let mut forbid = false;
        let mut nosuggest = false;

        for flag in flags {
            if self.flags.get(flag).is_none() {
                // FIXME: we get stuck on compound rules
                continue;
            }

            match self.flags.get(flag).unwrap() {
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
            let meta = Meta::new(stem.clone(), Source::Dict(morph.into()));
            let meta_vec = dest.0.entry_ref(stem.as_ref()).or_insert_with(Vec::new);
            meta_vec.push(meta);
        }

        create_affixed_word_map(&prefix_rules, &suffix_rules, stem, dest);
        prefix_rules.clear();
        suffix_rules.clear();
    }

    /// Update the internal wordlist and forbidden wordlist from a dictionary
    /// file string
    fn parse_update_wordlist(&mut self, source: &str) -> Result<(), Error> {
        let entries = DictEntry::parse_all(source, self.flag_type)?;
        self.update_wordlist(&entries)
    }

    /// Update internal wordlists from dictionary entries
    #[allow(clippy::unnecessary_wraps)]
    fn update_wordlist(&mut self, entries: &[DictEntry]) -> Result<(), Error> {
        // the en dictionary has about 3 words per entry, German has 8ish
        self.wordlist.0.reserve(entries.len() * 5);

        // PERF: try moving flags outside of loop
        for entry in entries {
            let DictEntry { stem, flags, morph } = entry;

            self.create_affixed_words(stem, flags, morph);
        }

        Ok(())
    }

    fn parse_update_personal(&mut self, source: &str, dict: &[DictEntry]) -> Result<(), Error> {
        let entries = PersonalEntry::parse_all(source);
        self.update_personal(entries, dict)
    }

    /// Must happen after `update_wordlist`
    #[allow(clippy::unnecessary_wraps)]
    fn update_personal(
        &mut self,
        entries: Vec<PersonalEntry>,
        _dict: &[DictEntry],
    ) -> Result<(), Error> {
        // FIXME: don't take `dict` as an argument, use our existing hashmaps
        self.wordlist.0.reserve(entries.len() * 2);
        for entry in entries {
            if let Some(_friend) = &entry.friend {
                // Find the friend in our dictionary, find its source affixes
                // let flags = dict.iter().find(|d| &d.stem() == friend).map(|d| &d.flags);
                todo!()
            } else {
                let stem_arc: Arc<str> = self.stems.get_or_insert(entry.stem).clone();

                let source = Source::Personal(Box::new(PersonalMeta::new(
                    None,
                    self.get_or_insert_morphs(&entry.morph),
                )));
                let meta = Meta::new(Arc::clone(&stem_arc), source);

                // Select the correct word to work with
                let hmap = if entry.forbid {
                    &mut self.wordlist_forbidden.0
                } else {
                    &mut self.wordlist.0
                };

                // Add our word, update its meta
                let extra_vec: &mut Vec<Meta> = hmap
                    .entry_ref(stem_arc.as_ref())
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

/// The result of checking whether a word exists or not, with methods to perform
/// advanced operations.
///
/// This type is created by [`Dictionary::entry`] and [`Dictionary::entries`].
pub struct WordEntry<'dict, 'word> {
    /// Word provided to be matched
    word: &'word str,
    index: usize,
    dict: &'dict Dictionary,
    context: WordCtx<'dict>,
}

/// Context held by a `WordEntry` that differs based on whether
/// the word is correct or not.
enum WordCtx<'dict> {
    Correct {
        /// The value that was matched in the dictionary
        matched: &'dict str,
        /// Meta located in the dictionary
        meta_list: &'dict [Meta],
    },
    Incorrect {
        /// True if the word was located in a forbidden dictionary
        forbidden: bool,
    },
}

impl<'dict, 'word> WordEntry<'dict, 'word> {
    /// Return true if the word is spelled correctly.
    ///
    /// If you only need correctness checking, it can be easier to go through
    /// [`Dictionary::check`] or related functions.
    #[inline]
    pub fn correct(&self) -> bool {
        matches!(self.context, WordCtx::Correct { .. })
    }

    /// The input word that was checked.
    ///
    /// This is mostly useful when creating this `WordEntry` from [`Dictionary::entries`],
    /// where a string is split into any number of words.
    #[inline]
    pub fn word(&self) -> &str {
        self.word
    }

    /// The index of this word, if located within a larger string.
    ///
    /// This is most helpful when extracting individual words via [`Dictionary::entries`].
    #[inline]
    pub fn index(&self) -> usize {
        self.index
    }

    /// The dictionary this match was produced by.
    #[inline]
    pub fn dict(&self) -> &Dictionary {
        self.dict
    }

    /// If this was found in the dictionary, return the exact entry (this will often be
    /// the same as `input` but not always).
    #[inline]
    pub fn matched_entry(&self) -> Option<&str> {
        match self.context {
            WordCtx::Correct { matched, .. } => Some(matched),
            WordCtx::Incorrect { .. } => None,
        }
    }

    /// True if this entry was found in a forbidden list
    #[inline]
    pub fn forbidden(&self) -> bool {
        matches!(self.context, WordCtx::Incorrect { forbidden: true })
    }

    /// Returns stemming if the word was found, `None` otherwise.
    ///
    /// Stems are a list of potential root words, including the word itself. This list may
    /// contain duplicates (collect to a [`HashSet`](std::collections::HashSet) or use
    /// [`Vec::dedup`](std::vec::Vec::dedup) if this is needed).
    ///
    /// Note that for this to be most useful, you need a dictionary that contains stemming
    /// information, but these are less common. Some of the [SCOWL] dictionaries provide this
    /// for English; I do not know of sources for other languages (please let me know if you
    /// do!).
    ///
    /// [SCOWL]: http://wordlist.aspell.net/
    ///
    /// ```
    /// use zspell::DictBuilder;
    ///
    /// let affix_str = "
    /// SFX X Y 1
    /// SFX X 0 able . ds:able
    /// ";
    /// let dict_str = "
    /// drink/X po:verb
    /// ";
    ///
    /// let dict = DictBuilder::new()
    ///     .config_str(affix_str)
    ///     .dict_str(dict_str)
    ///     .build()
    ///     .unwrap();
    ///
    /// let entry = dict.entry("drink");
    /// let stems: Vec<_> = entry.stems().unwrap().collect();
    /// assert_eq!(stems, ["drink"]);
    ///
    /// let entry = dict.entry("drinkable");
    /// let stems: Vec<_> = entry.stems().unwrap().collect();
    /// assert_eq!(stems, ["drinkable", "drink"]);
    ///
    /// // Should be the same with capital letters
    /// let entry = dict.entry("Drinkable");
    /// let stems: Vec<_> = entry.stems().unwrap().collect();
    /// assert_eq!(stems, ["drinkable", "drink"]);
    /// ```
    #[inline]
    pub fn stems(&self) -> Option<impl Iterator<Item = &str>> {
        let WordCtx::Correct { matched, meta_list } = self.context else {
            return None;
        };

        let ret = meta_list.iter().flat_map(|meta| {
            // Combine the main stem with every stem provided by morphs
            let stem = std::iter::once(meta.stem());
            let morph_stems = meta.source().morphs().filter_map(|morph| match morph {
                MorphInfo::Stem(v) => Some(v.as_ref()),
                _ => None,
            });

            stem.chain(morph_stems)
        });
        // remove self because we will include that at the beginning
        let ret = ret.filter(move |value| value != &matched);
        let ret = std::iter::once(matched).chain(ret);
        Some(ret)
    }

    /// Return morphological analysis information about a word if found, `None` otherwise
    ///
    /// Like with [`stems`](Self::stems), this is most useful with nonstandard dictionaries that
    /// include morphological information.
    ///
    /// ```
    /// use zspell::{DictBuilder, MorphInfo, PartOfSpeech};
    ///
    /// let affix_str = "
    /// SFX X Y 1
    /// SFX X 0 able . ds:able
    /// ";
    /// let dict_str = "
    /// drink/X po:verb
    /// ";
    ///
    /// let dict = DictBuilder::new()
    ///     .config_str(affix_str)
    ///     .dict_str(dict_str)
    ///     .build()
    ///     .unwrap();
    ///
    /// # // FIXME:dict-parser our dictionary parser doesn't extract prefixes properly
    /// # // our file contains information that we have a verb part of speech
    /// # // and a derivational suffix
    /// # let verb_pos = MorphInfo::Part(PartOfSpeech::Verb);
    /// let deriv_sfx = MorphInfo::DerivSfx("able".into());
    ///
    /// # // let entry = dict.entry("drink");
    /// # // let stems: Vec<_> = entry.analyze().unwrap().collect();
    /// # //assert_eq!(stems, [&verb_pos]);
    ///
    /// let entry = dict.entry("drinkable");
    /// let stems: Vec<_> = entry.analyze().unwrap().collect();
    /// assert_eq!(stems, [&deriv_sfx])
    /// # // assert_eq!(stems, [&verb_pos, &deriv_sfx]);
    /// # // ^ yeah, this isn't a verb, but this is just an example...
    /// ```
    #[inline]
    pub fn analyze(&self) -> Option<impl Iterator<Item = &MorphInfo>> {
        let WordCtx::Correct { meta_list, .. } = self.context else {
            return None;
        };
        let ret = meta_list.iter().flat_map(|meta| meta.source().morphs());
        Some(ret)
    }

    /// Suggest replacements for a word. Feature gated behind `unstable-suggestions`.
    ///
    /// If the word is correct, this will return `None`. Otherwise, it will return an
    /// iterator over suggested words.
    ///
    /// This function is unstable because it has performance issues. We are
    /// going to try to speed up the algorithm significantly.
    // PERF: bench with par_iter
    #[inline]
    #[cfg(feature = "unstable-suggestions")]
    pub fn suggest(&self) -> Option<Vec<&str>> {
        if self.correct() {
            return None;
        };

        let mut suggestions: Vec<(u32, &str)> = self
            .dict
            .wordlist
            .0
            .keys()
            .filter_map(|key| try_levenshtein(key, self.word, 1).map(|lim| (lim, key.as_ref())))
            .collect();
        suggestions.sort_unstable_by_key(|(k, _v)| *k);
        Some(suggestions.iter().take(10).map(|(_k, v)| *v).collect())
    }
}

/// The internal representation of a wordlist.
///
/// Currently contains a `HashMap<String, Vec<Meta>>`
#[doc(hidden)]
#[derive(Clone, Debug, PartialEq)]
pub struct WordList(HashMap<Box<str>, Vec<Meta>>);

impl WordList {
    fn new() -> Self {
        Self(HashMap::new())
    }

    /// **UNSTABLE** Get a reference to the internal map. This is behind the
    /// `zspell-unstable` marker as the internal format may change
    #[inline]
    #[cfg_attr(feature = "zspell-unstable", visibility::make(pub))]
    pub(crate) fn inner(&self) -> &HashMap<Box<str>, Vec<Meta>> {
        &self.0
    }
}

/// A builder stucture that is used to create a [`Dictionary`].
///
/// See module-level documentation for an example.
#[must_use]
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
    pub fn config_str(mut self, config: &'a str) -> Self {
        self.cfg_src = Some(config);
        self
    }

    /// Use instead of `config_str` if you have a preexisting `Config` type
    ///
    /// Don't use with `config_src`
    #[inline]
    #[cfg_attr(feature = "zspell-unstable", visibility::make(pub))]
    fn config(mut self, cfg: ParsedCfg) -> Self {
        self.cfg = Some(cfg);
        self
    }

    /// Load the dictionary file from a string
    #[inline]
    pub fn dict_str(mut self, dict: &'a str) -> Self {
        self.dict_src = Some(dict);
        self
    }

    /// Load a personal dictionary file from a string
    #[inline]
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
