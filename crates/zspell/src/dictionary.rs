//! Word checking and correction suggestion framework
//!
//! This module is generally not imported, since [`Dictionary`] can be directly
//! imported from [`crate`].

use hashbrown::hash_set::Iter as HashSetIter;
use hashbrown::HashSet;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    affix::Config,
    errors::{AffixError, CompileError, DictError},
};

/// Main dictionary object used for spellchecking and autocorrect
///
/// A dictionary contains
///
/// Load hunspell dicts, as described at
/// <http://pwet.fr/man/linux/fichiers_speciaux/hunspell/>
#[derive(Debug, PartialEq, Eq)]
pub struct Dictionary {
    /// This contains the dictionary's configuration
    pub config: Config,

    /// General word list of words that are accepted and suggested. Note that it
    /// may make sense in the future to include non-suggest words here too.
    wordlist: HashSet<String>,
    /// Words to accept but never suggest
    wordlist_nosuggest: HashSet<String>,
    /// Words forbidden by the personal dictionary, i.e. do not accept as correct
    wordlist_forbidden: HashSet<String>,

    /// These hold the files as loaded
    /// Will be emptied upon compile
    raw_wordlist: Vec<String>,
    raw_wordlist_personal: Vec<String>,
    /// Indicator of whether or not this has been compiled
    compiled: bool,
}

impl Dictionary {
    /// Create a new, completely empty dictionary
    #[inline]
    pub fn new() -> Self {
        Self {
            config: Config::new(),
            wordlist: HashSet::new(),
            wordlist_nosuggest: HashSet::new(),
            wordlist_forbidden: HashSet::new(),
            raw_wordlist: Vec::new(),
            raw_wordlist_personal: Vec::new(),
            compiled: false,
        }
    }

    /// Load this dictionary's affix configuration from
    ///
    /// # Errors
    ///
    /// Returns an error if loading is unsuccessful
    #[inline]
    pub fn load_affix_from_str(&mut self, s: &str) -> Result<(), AffixError> {
        self.compiled = false;
        self.config.load_from_str(s)
    }

    /// Load this dictionary's word list from a string
    ///
    /// # Errors
    ///
    /// Returns an error if the load was unsuccessful
    #[inline]
    pub fn load_dict_from_str(&mut self, s: &str) -> Result<(), DictError> {
        self.compiled = false;

        let mut lines = s.lines();

        // First line "should" be a rough length of the list, need to extract it
        let firstline = if let Some(v) = lines.next() {
            v
        } else {
            self.raw_wordlist = Vec::new();
            return Ok(());
        };

        // Don't sweat it if we can't extract a number, just add it
        // to the list
        if let Ok(v) = firstline.parse::<usize>() {
            self.raw_wordlist = Vec::with_capacity(v);
        } else {
            self.raw_wordlist = Vec::with_capacity(10000);
            self.raw_wordlist.push(firstline.to_owned());
        };

        // Add all our items then shrink capacity as needed
        self.raw_wordlist.extend(lines.map(ToOwned::to_owned));
        self.raw_wordlist.shrink_to_fit();

        Ok(())
    }

    /// Load this dictionary's personal word list from a string
    ///
    /// # Errors
    ///
    /// Returns an error if the personal dictionary could not be loaded
    #[inline]
    pub fn load_personal_dict_from_str(&mut self, s: &str) -> Result<(), DictError> {
        self.compiled = false;

        self.raw_wordlist_personal = s.lines().map(ToOwned::to_owned).collect();
        Ok(())
    }

    /// Match affixes, personal dict, etc
    ///
    /// # Errors
    ///
    /// Raises an error if the compilation was unsuccessful
    #[inline]
    pub fn compile(&mut self) -> Result<(), CompileError> {
        // Work through the personal word list
        for word in &self.raw_wordlist_personal {
            // Words will be in the format "*word/otherword" where "word" is the
            // main word to add, but it will get all rules of "otherword"
            let split: Vec<&str> = word.split('/').collect();
            let _forbidden = word.starts_with('*');

            if let Some(rootword) = split.get(1) {
                // Find "otherword/" in main wordlist
                let mut tmp = (*rootword).to_owned();
                tmp.push('/');
                let filtval = tmp.trim_start_matches('*');

                match self.raw_wordlist.iter().find(|s| s.starts_with(filtval)) {
                    Some(_w) => (),
                    None => {
                        return Err(CompileError::MissingRootWord {
                            rootword: (*rootword).to_owned(),
                        })
                    }
                }
            }
        }

        // The english dictionay has about 3 words per line, use that as a baseline
        self.wordlist.reserve(self.raw_wordlist.len() * 3);

        for word in &self.raw_wordlist {
            let split: Vec<&str> = word.split('/').collect();
            let rootword = *split.first().unwrap();
            match split.get(1) {
                Some(rule_keys) => {
                    // Create all relevant words from this root word and affix
                    let tmp_wordlist = self.config.create_affixed_words(rootword, rule_keys);

                    if !&self.config.nosuggest_flag.is_empty()
                        && rule_keys.contains(&self.config.nosuggest_flag)
                    {
                        self.wordlist_nosuggest.extend(tmp_wordlist);
                    } else {
                        self.wordlist.extend(tmp_wordlist);
                    }
                }
                None => {
                    self.wordlist.insert(rootword.to_owned());
                }
            }
        }

        self.wordlist.shrink_to_fit();

        self.compiled = true;

        Ok(())
    }

    /// Check that a single word is spelled correctly. Returns true if so
    ///
    /// This is the main spellchecking feature. It checks a single word for
    /// validity according to the loaded dictionary. This accepts any
    /// string-like type including `&str`, `String` and `&String`
    ///
    /// # Panics
    ///
    /// This will panic if the dictionary has not yet been compiled.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs;
    /// use zspell::Dictionary;
    ///
    /// let mut dic = Dictionary::new();
    ///
    /// let aff_content = fs::read_to_string("tests/files/w1_eng_short.aff").unwrap();
    /// let dic_content = fs::read_to_string("tests/files/w1_eng_short.dic").unwrap();
    ///
    /// dic.config.load_from_str(aff_content.as_str()).unwrap();
    /// dic.load_dict_from_str(dic_content.as_str());
    /// dic.compile().unwrap();
    ///
    /// assert_eq!(dic.check("reptiles"), Ok(true));
    /// assert_eq!(dic.check("pillow"), Ok(true));
    /// assert_eq!(dic.check("missssspelled"), Ok(false));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the dictionary has not yet been compiled
    #[inline]
    pub fn check<T: AsRef<str>>(&self, s: T) -> Result<bool, DictError> {
        self.break_if_not_compiled()?;

        // Just check if any words are spelled incorrectly, then return the opposite
        Ok(!s
            .as_ref()
            .unicode_words()
            .any(|w| !self.check_word_no_break(w)))
    }

    /// Perform spellcheck on a string, return a list of misspelled words.
    /// Returns an iterator.
    ///
    /// # Errors
    ///
    /// Returns [`DictError::NotCompiled`] if the dictionary has not yet been
    /// compiled
    #[inline]
    pub fn check_returning_list<T: AsRef<str>>(&self, s: T) -> Result<Vec<String>, DictError> {
        // We actually just need to check
        self.break_if_not_compiled()?;

        Ok(s.as_ref()
            .unicode_words()
            .filter(|word| !self.check_word_no_break(word))
            .map(ToOwned::to_owned)
            .collect::<Vec<String>>())
    }

    /// Perform spellcheck on a single word
    ///
    /// # Errors
    ///
    /// Returns [`DictError::NotCompiled`] if the dictionary has not yet been
    /// compiled
    #[inline]
    pub fn check_word<T: AsRef<str>>(&self, s: T) -> Result<bool, DictError> {
        // We actually just need to check
        self.break_if_not_compiled()?;

        Ok(self.check_word_no_break(s))
    }

    // Private function that checks a single word. Same as check() but doesn't
    // validate this dictionary is compiled
    #[inline]
    fn check_word_no_break<T: AsRef<str>>(&self, s: T) -> bool {
        // Convert to a usable string reference
        let sref = s.as_ref();
        let lower = &sref.to_lowercase();

        // Must not be in a forbidden word list
        // Note that in the future this implementation might change
        // And one of the "exists" wordlists contains the word
        (!self.wordlist_forbidden.contains(sref))
            && (self.wordlist.contains(sref)
                || self.wordlist.contains(lower)
                || self.wordlist_nosuggest.contains(sref)
                || self.wordlist_nosuggest.contains(lower))
    }

    /// Create a iterator over all items in the dictionary's wordlist. That is,
    /// words that will always be accepted and succested.
    ///
    /// Note that this is relatively slow. Prefer [`Dictionary::check`] for
    /// validating a word exists.
    ///
    /// # Errors
    ///
    /// Returns [`DictError::NotCompiled`] if the dictionary has not yet been
    /// compiled
    #[inline]
    pub fn iter_wordlist_items(&self) -> Result<HashSetIter<String>, DictError> {
        self.break_if_not_compiled()?;

        Ok(self.wordlist.iter())
    }

    /// Create a iterator over all items in the dictionary's nonsuggesting
    /// wordlist. These are words that will always be accepted but never be
    /// suggested.
    ///
    /// Note that this is relatively slow. Prefer [`Dictionary::check`] for
    /// validating a word exists.
    ///
    /// # Errors
    ///
    /// Returns [`DictError::NotCompiled`] if the dictionary has not yet been
    /// compiled
    #[inline]
    pub fn iter_wordlist_nosuggest_items(&self) -> Result<HashSetIter<String>, DictError> {
        self.break_if_not_compiled()?;

        Ok(self.wordlist_nosuggest.iter())
    }

    /// Create a iterator over all items in the dictionary's forbidden wordlist.
    /// These are words that are never accepted as correct.
    ///
    /// Note that this is relatively slow. Prefer [`Dictionary::check`] for
    /// validating a word exists.
    ///
    /// # Errors
    ///
    /// Returns [`DictError::NotCompiled`] if the dictionary has not yet been
    /// compiled
    #[inline]
    pub fn iter_wordlist_forbidden_items(&self) -> Result<HashSetIter<String>, DictError> {
        self.break_if_not_compiled()?;

        Ok(self.wordlist_forbidden.iter())
    }

    /// Helper function to error if we haven't compiled when we needed to
    #[inline]
    const fn break_if_not_compiled(&self) -> Result<(), DictError> {
        if self.compiled {
            Ok(())
        } else {
            Err(DictError::NotCompiled)
        }
    }
}

impl Default for Dictionary {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
