mod parser;
pub(crate) mod types;

use hashbrown::hash_set::Iter as HashSetIter;
use hashbrown::{HashMap, HashSet};
use unicode_segmentation::UnicodeSegmentation;

use self::types::Meta;
use crate::error::{BuildError, Error};
use crate::Config;

type WordList<'a> = HashMap<String, Vec<&'a Meta>>;

/// Main dictionary object used for spellchecking and suggestions
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dictionary<'a> {
    /// General word list of words that are accepted and suggested. Note that it
    /// may make sense in the future to include non-suggest words here too.
    wordlist: WordList<'a>,
    /// Words to accept but never suggest
    wordlist_nosuggest: WordList<'a>,
    /// Words forbidden by the personal dictionary, i.e. do not accept as correct
    wordlist_forbidden: WordList<'a>,
    /// Information about how the wordlists were built
    meta: HashSet<Meta>,

    config: Box<Config>,
}

pub struct DictBuilder<'a> {
    cfg: Option<Config>,
    cfg_src: Option<&'a str>,
    dict_src: Option<&'a str>,
    personal_src: Option<&'a str>,
}

impl<'a> DictBuilder<'a> {
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
    pub fn config_from_str(&mut self, s: &'a str) -> &mut Self {
        self.cfg_src = Some(s);
        self
    }

    /// Use instead of `config_from_str` if you have a preexisting `Config` type
    ///
    /// Don't use with `config_src`
    pub fn config(&mut self, cfg: Config) -> &mut Self {
        self.cfg = Some(cfg);
        self
    }

    pub fn compile(self) -> Result<Dictionary<'a>, Error> {
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

        let mut dict = Dictionary {
            wordlist: WordList::new(),
            wordlist_nosuggest: WordList::new(),
            wordlist_forbidden: WordList::new(),
            meta: HashSet::new(),
            config: Box::new(cfg),
        };
        todo!()
    }
}
