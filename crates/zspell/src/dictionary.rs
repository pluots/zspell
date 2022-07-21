//! A dictionary contains methods and a list of Entries
//! Load hunspell dicts, as described at
//! <http://pwet.fr/man/linux/fichiers_speciaux/hunspell/>

use crate::{
    affix::AffixConfig,
    errors::{AffixError, CompileError},
};
use core::hash::Hash;
use std::collections::HashSet;
use stringmetrics::tokenizers::split_whitespace_remove_punc;

/// Main dictionary object used for spellchecking and autocorrect
///
/// A dictionary contains
pub struct Dictionary {
    /// This contains the dictionary's configuration
    pub config: AffixConfig,

    // General word list of words that are accepted and suggested. Note that it
    // may make sense in the future to include non-suggest words here too.
    wordlist: HashSet<String>,
    // Words to accept but never suggest
    wordlist_nosuggest: HashSet<String>,
    // Words forbidden by the personal dictionary, i.e. do not accept as correct
    wordlist_forbidden: HashSet<String>,

    // These hold the files as loaded
    // Will be emptied upon compile
    raw_wordlist: Vec<String>,
    raw_wordlist_personal: Vec<String>,
    // Indicator of whether or not this has been compiled
    compiled: bool,
}

impl Dictionary {
    pub fn new() -> Dictionary {
        Dictionary {
            config: AffixConfig::new(),
            wordlist: HashSet::new(),
            wordlist_nosuggest: HashSet::new(),
            wordlist_forbidden: HashSet::new(),
            raw_wordlist: Vec::new(),
            raw_wordlist_personal: Vec::new(),
            compiled: false,
        }
    }

    /// Can also be done with strings
    pub fn load_affix_from_str(&mut self, s: &str) -> Result<(), AffixError> {
        self.compiled = false;
        self.config.load_from_str(s)
    }

    pub fn load_dict_from_str(&mut self, s: &str) {
        self.compiled = false;

        let mut lines = s.lines();
        // First line is just a count of the number of items
        let _first = lines.next();
        self.raw_wordlist = lines.map(|l| l.to_string()).collect()
    }

    pub fn load_personal_dict_from_str(&mut self, s: &str) {
        self.compiled = false;

        self.raw_wordlist_personal = s.lines().map(|l| l.to_string()).collect()
    }

    /// Match affixes, personal dict, etc
    pub fn compile(&mut self) -> Result<(), CompileError> {
        // Work through the personal word list
        for word in self.raw_wordlist_personal.iter() {
            // Words will be in the format "*word/otherword" where "word" is the
            // main word to add, but it will get all rules of "otherword"
            let split: Vec<&str> = word.split('/').collect();
            let _forbidden = word.starts_with('*');

            match split.get(1) {
                Some(rootword) => {
                    // Find "otherword/" in main wordlist
                    let mut tmp = rootword.to_string();
                    tmp.push('/');
                    let filtval = tmp.trim_start_matches('*');

                    match self.raw_wordlist.iter().find(|s| s.starts_with(&filtval)) {
                        Some(_w) => (),
                        None => {
                            return Err(CompileError::MissingRootWord {
                                rootword: rootword.to_string(),
                            })
                        }
                    }
                }
                None => (),
            }
        }

        for word in self.raw_wordlist.iter() {
            let split: Vec<&str> = word.split('/').collect();
            let rootword = split[0];
            match split.get(1) {
                Some(rule_keys) => {
                    let wordlist = self.config.create_affixed_words(rootword, rule_keys);
                    match rule_keys.contains(&self.config.nosuggest_flag) {
                        true => iter_to_hashset(wordlist, &mut self.wordlist_nosuggest),
                        false => iter_to_hashset(wordlist, &mut self.wordlist),
                    }
                }
                None => {
                    self.wordlist.insert(rootword.to_string());
                }
            }
        }

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
    /// let aff_content = fs::read_to_string("tests/files/short.aff").unwrap();
    /// let dic_content = fs::read_to_string("tests/files/short.dic").unwrap();
    ///
    /// dic.config.load_from_str(aff_content.as_str()).unwrap();
    /// dic.load_dict_from_str(dic_content.as_str());
    /// dic.compile().unwrap();
    ///
    /// assert_eq!(dic.check("yyication"), true);
    /// ```
    #[inline]
    pub fn check<T: AsRef<str>>(&self, s: T) -> bool {
        // We actually just need to check
        self.break_if_not_compiled();

        let sref = s.as_ref();

        for word in split_whitespace_remove_punc(sref) {
            if !self.check_word_no_break(word) {
                return false;
            }
        }

        true
    }

    /// Perform spellcheck on a string, return a list of misspelled words.
    /// Returns an iterator.
    pub fn check_return_list<T: AsRef<str>>(&self, s: T) -> Vec<String> {
        // We actually just need to check
        self.break_if_not_compiled();

        split_whitespace_remove_punc(s.as_ref())
            .filter(|word| !self.check_word_no_break(word))
            .collect::<Vec<String>>()
    }

    /// Perform spellcheck on a single word
    #[inline]
    pub fn check_word<T: AsRef<str>>(&self, s: T) -> bool {
        // We actually just need to check
        self.break_if_not_compiled();

        self.check_word_no_break(s)
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
            && (self.wordlist.contains(sref) || self.wordlist.contains(lower))
    }

    /// Create a sorted vector of all items in the word list
    ///
    /// Note that this is relatively slow. Prefer [`Dictionary::check`] for
    /// validating a word exists.
    pub fn wordlist_items(&self) -> Vec<&str> {
        self.break_if_not_compiled();

        let mut items = self
            .wordlist
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>();
        items.sort_unstable();
        items
    }

    /// Helper function to error if we haven't compiled when we needed to
    #[inline]
    fn break_if_not_compiled(&self) {
        assert!(
            self.compiled,
            "This method requires compiling the dictionary with `dic.compile()` first."
        )
    }
}

impl Default for Dictionary {
    fn default() -> Self {
        Self::new()
    }
}

fn iter_to_hashset<I, T>(items: I, hs: &mut HashSet<T>)
where
    I: IntoIterator<Item = T>,
    T: Eq + Hash,
{
    for item in items {
        hs.insert(item);
    }
}
