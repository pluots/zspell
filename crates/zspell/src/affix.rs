//! Representation of an affix file

pub(crate) mod types;
pub(crate) mod types_impl;

use self::types::{
    AffixRule, CompoundPattern, CompoundSyllable, Conversion, Encoding, FlagType, MorphInfo,
    Phonetic, RuleGroup, RuleType,
};
use crate::dict::parser::DictEntry;
use crate::error::{BuildError, Error};
use crate::parser_affix::parse_affix;
use crate::parser_affix::types::AffixNode;

/// A representation of an affix file
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    /*
        General Options
    */
    /// Charset to use, reference to an [`Encoding`] Currently this is
    /// unused; only UTF-8 is supported. However, the affix file must still have
    /// an accurate definition.
    encoding: Encoding,

    /// The type of flag in the `.dic` file
    flagtype: FlagType,

    /// Twofold prefix skipping for e.g. right-to-left languages
    complex_prefixes: bool,

    /// Language code, currently unused. Consider this unstable as it may change
    /// to be an object reference.
    lang: String,

    /// List of characters to ignore
    ignore_chars: Vec<char>,

    /// List of usable flag vectors. Defaults to all things after "/"" in a dict.
    affix_alias: Vec<String>,

    /// List of usable flag vectors
    morph_alias: Vec<String>,

    /*
        Suggestion options
    */
    /// List of e.g. "qwerty", "asdfg" that define neighbors
    neighbor_keys: Vec<String>,

    // Rules for suggestion replacements to try
    replacements: Vec<Conversion>,

    /// Suggest words that differe by 1 try character
    try_characters: String,

    /// Flag used to indicate words that should not be suggested
    nosuggest_flag: Option<char>,

    /// Note rare (i.e. commonly misspelled) words with this flag
    warn_rare_flag: Option<char>,

    /// Don't suggest anything with spaces
    no_split_suggestions: bool,

    /// If a dot comes with the spellcheck, return one with a suggestion word
    keep_term_dots: bool,

    /// Whether to never suggest words with the warn flag (above)
    forbid_warn_words: bool,

    /// Replace commonly misused letters, e.g. `u`/`ü`
    maps: Vec<(char, char)>,

    /// Phonetic replacements for similar words
    phonetics: Vec<Phonetic>,

    /*
        ngram configuration
    */
    /// Max number of ngram suggestions
    ngram_sug_max: u16,

    /// N-gram similarity limit
    ngram_diff_max: u8,

    /// Remove all suggestions except the diff max
    ngram_limit_to_diff_max: bool,

    /*
        Compounding-related items
    */
    compound_config: Box<CompoundConfig>,

    /*
        Affix Options
    */
    // Rules for setting prefixes and suffixes
    affix_rules: Vec<RuleGroup>,

    /*
        Other options
    */
    afx_circumflex_flag: Option<char>,
    forbidden_word_flag: Option<char>,
    afx_full_strip: bool,
    afx_keep_case_flag: char,
    input_conversions: Vec<Conversion>,
    output_conversions: Vec<Conversion>,
    afx_needed_flag: char,
    afx_substandard_flag: char,
    afx_word_chars: String,
    afx_check_sharps: bool,
    name: String,
    home_page: String,
    version: String,
}

/// Separated structure for compound rules
#[derive(Clone, Debug, PartialEq, Eq)]
struct CompoundConfig {
    /// Something like `-` to indicate whether both sides should be checked
    /// Prefer COMPOUNDRULE instead
    break_separators: Vec<String>,

    /// Maximum compound word suggestions
    sug_max: u16,

    /// Regex-like rules for compound words
    rules: Vec<String>,

    /// Minimum length of words used in a compound
    min_length: u16,

    /// Words with this flag may be in compounds
    flag: Option<char>,

    /// Words with this flag may start a compound
    begin_flag: Option<char>,

    /// Words with this flag may end a compound
    end_flag: Option<char>,

    /// Words with this flag may be in the middle of a compound
    middle_flag: Option<char>,

    /// Words with this flag can't be on their own, only in compounds
    only_flag: Option<char>,

    /// Allow these words inside compounds
    permit_flag: Option<char>,
    forbid_flag: Option<char>,
    more_suffixes: bool,
    root: Option<char>,
    word_max: u16,
    forbid_dup: bool,
    forbid_repeat: bool,
    check_case: bool,
    check_triple: bool,
    simplify_triple: bool,
    forbid_pats: Vec<CompoundPattern>,
    force_upper_flag: Option<char>,
    syllable: CompoundSyllable,
    syllable_num: String,
}

impl Default for Config {
    #[allow(clippy::default_trait_access)]
    #[inline]
    fn default() -> Self {
        Self {
            encoding: Default::default(),
            flagtype: Default::default(),
            complex_prefixes: Default::default(),
            lang: Default::default(),
            ignore_chars: Default::default(),
            affix_alias: Default::default(),
            morph_alias: Default::default(),
            neighbor_keys: vec![
                "qwertyuiop".to_owned(),
                "asdfghjkl".to_owned(),
                "zxcvbnm".to_owned(),
            ],
            replacements: Default::default(),
            try_characters: "esianrtolcdugmphbyfvkwzESIANRTOLCDUGMPHBYFVKWZ'".to_owned(),
            nosuggest_flag: Some('!'),
            warn_rare_flag: Default::default(),
            no_split_suggestions: Default::default(),
            keep_term_dots: Default::default(),
            forbid_warn_words: Default::default(),
            maps: Default::default(),
            phonetics: Default::default(),
            ngram_sug_max: 2,
            ngram_diff_max: 5,
            ngram_limit_to_diff_max: Default::default(),
            compound_config: Default::default(),
            affix_rules: Default::default(),
            afx_circumflex_flag: Default::default(),
            forbidden_word_flag: Default::default(),
            afx_full_strip: Default::default(),
            afx_keep_case_flag: Default::default(),
            input_conversions: Default::default(),
            output_conversions: Default::default(),
            afx_needed_flag: Default::default(),
            afx_substandard_flag: Default::default(),
            afx_word_chars: Default::default(),
            afx_check_sharps: Default::default(),
            name: Default::default(),
            home_page: Default::default(),
            version: Default::default(),
        }
    }
}

impl Default for CompoundConfig {
    #[allow(clippy::default_trait_access)]
    fn default() -> Self {
        Self {
            break_separators: Default::default(),
            sug_max: 3,
            rules: Default::default(),
            min_length: 3,
            flag: Default::default(),
            begin_flag: Default::default(),
            end_flag: Default::default(),
            middle_flag: Default::default(),
            only_flag: Default::default(),
            permit_flag: Default::default(),
            forbid_flag: Default::default(),
            more_suffixes: Default::default(),
            root: Default::default(),
            word_max: Default::default(),
            forbid_dup: Default::default(),
            forbid_repeat: Default::default(),
            check_case: Default::default(),
            check_triple: Default::default(),
            simplify_triple: Default::default(),
            forbid_pats: Default::default(),
            force_upper_flag: Default::default(),
            syllable: Default::default(),
            syllable_num: Default::default(),
        }
    }
}

impl Config {
    /// Create a `Config` object from a string version of an affix file
    ///
    /// # Errors
    ///
    /// Returns an error if there is a problem parsing, or if the file is
    /// invalid
    #[inline]
    pub fn load_from_str(s: &str) -> Result<Self, Error> {
        Self::from_parsed(parse_affix(s)?)
    }

    #[allow(clippy::unnecessary_wraps)]
    fn from_parsed(v: Vec<AffixNode>) -> Result<Self, Error> {
        let mut res = Self::default();
        let mut warnings: Vec<String> = Vec::new();

        for node in v {
            let name_str = node.name_str();
            match node {
                AffixNode::Encoding(v) => res.encoding = v,
                AffixNode::FlagType(v) => res.flagtype = v,
                AffixNode::ComplexPrefixes => res.complex_prefixes = true,
                AffixNode::Language(v) => res.lang = v,
                AffixNode::IgnoreChars(v) => res.ignore_chars = v,
                AffixNode::AffixAlias(v) => res.affix_alias = v,
                AffixNode::MorphAlias(v) => res.morph_alias = v,
                AffixNode::NeighborKeys(v) => res.neighbor_keys = v,
                AffixNode::TryCharacters(v) => res.try_characters = v,
                AffixNode::NoSuggestFlag(v) => res.nosuggest_flag = Some(v),
                AffixNode::CompoundSugMax(v) => res.compound_config.sug_max = v,
                AffixNode::NGramSugMax(v) => res.ngram_sug_max = v,
                AffixNode::NGramDiffMax(v) => res.ngram_diff_max = v,
                AffixNode::NGramLimitToDiffMax => res.ngram_limit_to_diff_max = true,
                AffixNode::NoSplitSuggestions => res.no_split_suggestions = true,
                AffixNode::KeepTermDots => res.keep_term_dots = true,
                AffixNode::Replacement(v) => res.replacements = v,
                AffixNode::Mapping(v) => res.maps = v,
                AffixNode::Phonetic(v) => res.phonetics = v,
                AffixNode::WarnRareFlag(v) => res.warn_rare_flag = Some(v),
                AffixNode::ForbidWarnWords => todo!(),
                AffixNode::BreakSeparator(v) => res.compound_config.break_separators = v,
                AffixNode::CompoundRule(v) => res.compound_config.rules = v,
                AffixNode::CompoundMinLen(v) => res.compound_config.min_length = v,
                AffixNode::CompoundFlag(v) => res.compound_config.flag = Some(v),
                AffixNode::CompoundBeginFlag(v) => res.compound_config.begin_flag = Some(v),
                AffixNode::CompoundEndFlag(v) => res.compound_config.end_flag = Some(v),
                AffixNode::CompoundMiddleFlag(v) => res.compound_config.middle_flag = Some(v),
                AffixNode::CompoundOnlyFlag(v) => res.compound_config.only_flag = Some(v),
                AffixNode::CompoundPermitFlag(v) => res.compound_config.permit_flag = Some(v),
                AffixNode::CompoundForbidFlag(v) => res.compound_config.forbid_flag = Some(v),
                AffixNode::CompoundMoreSuffixes => res.compound_config.more_suffixes = true,
                AffixNode::CompoundRoot(v) => res.compound_config.root = Some(v),
                AffixNode::CompoundWordMax(v) => res.compound_config.word_max = v,
                AffixNode::CompoundForbidDup => res.compound_config.forbid_dup = true,
                AffixNode::CompoundForbidRepeat => res.compound_config.forbid_repeat = true,
                AffixNode::CompoundCheckCase => res.compound_config.check_case = true,
                AffixNode::CompoundCheckTriple => res.compound_config.check_triple = true,
                AffixNode::CompoundSimplifyTriple => res.compound_config.simplify_triple = true,
                AffixNode::CompoundForbidPats(v) => res.compound_config.forbid_pats = v,
                AffixNode::CompoundForceUpper(v) => res.compound_config.force_upper_flag = Some(v),
                AffixNode::CompoundSyllable(v) => res.compound_config.syllable = v,
                AffixNode::SyllableNum(v) => res.compound_config.syllable_num = v,
                AffixNode::Prefix(v) => res.affix_rules.push(v),
                AffixNode::Suffix(v) => res.affix_rules.push(v),
                AffixNode::AfxCircumfixFlag(v) => res.afx_circumflex_flag = Some(v),
                AffixNode::ForbiddenWordFlag(v) => res.forbidden_word_flag = Some(v),
                AffixNode::AfxFullStrip => res.afx_full_strip = true,
                AffixNode::AfxKeepCaseFlag(v) => res.afx_keep_case_flag = v,
                AffixNode::AfxInputConversion(v) => res.input_conversions = v,
                AffixNode::AfxOutputConversion(v) => res.output_conversions = v,
                AffixNode::AfxLemmaPresentFlag(v) => {
                    warnings.push(format!("flag {name_str} is deprecated"));
                }
                AffixNode::AfxNeededFlag(v) => res.afx_needed_flag = v,
                AffixNode::AfxPseudoRootFlag(v) => {
                    warnings.push(format!("flag {name_str} is deprecated"));
                }
                AffixNode::AfxSubstandardFlag(v) => res.afx_substandard_flag = v,
                AffixNode::AfxWordChars(v) => res.afx_word_chars = v,
                AffixNode::AfxCheckSharps => res.afx_check_sharps = true,
                AffixNode::Comment => (),
                AffixNode::Name(v) => res.name = v,
                AffixNode::HomePage(v) => res.home_page = v,
                AffixNode::Version(v) => res.version = v,
            }
        }

        Ok(res)
    }

    /// Verify that all flags for a list of entries are valid
    pub(crate) fn validate_entry_flags(&self, entries: &[DictEntry]) -> Result<(), Error> {
        for entry in entries {
            self.validate_flags(&entry.flags)?;
        }
        Ok(())
    }

    /// Ensure that a list of flags is valid for this affix
    pub(crate) fn validate_flags<S: AsRef<str>>(&self, flags: &[S]) -> Result<(), Error> {
        for flag in flags {
            let flag_ref = flag.as_ref();
            if !self.affix_rules.iter().any(|group| group.flag == flag_ref) {
                return Err(BuildError::InvalidFlag(flag_ref.to_owned()).into());
            }
        }
        Ok(())
    }

    /// Create a vector of words from a single root word by applying rules in
    /// this affix. Does not check if the flag is valid.
    ///
    /// May contain duplicates, does not contain the original word
    ///
    /// Return type is vector of `(new_word, rule, second_rule)` where
    /// `second_rule` is available if both a prefix and a suffix were applied
    pub(crate) fn create_affixed_words<S: AsRef<str>>(
        &self,
        stem: &str,
        flags: &[S],
    ) -> Vec<(String, &AffixRule, Option<&AffixRule>)> {
        // BENCH: new vs. with capacity (with cap flags.len()?)
        let mut ret: Vec<(String, &AffixRule, Option<&AffixRule>)> = Vec::new();
        if flags.is_empty() {
            return ret;
        }

        // Store words with prefixes that can also have suffixes
        let mut prefixed_words: Vec<(String, &AffixRule)> = Vec::new();
        let mut suffix_rules: Vec<&AffixRule> = Vec::new();

        // Loop through rules where the flag matches and there are new words to
        // create.
        self.affix_rules
            .iter()
            // Use a fake `contains` because of `as_ref` (asm is about the same)
            .filter(|group| flags.iter().map(AsRef::as_ref).any(|s| s == group.flag))
            .for_each(|group| {
                if let Some((newword, rule)) = group.apply_pattern_meta(stem) {
                    if group.can_combine {
                        // For rules that can combine: if a prefix, store the
                        // word. If a suffix, store the rule. We'll go through
                        // and cross match these
                        if group.kind == RuleType::Prefix {
                            prefixed_words.push((newword.clone(), rule));
                        } else {
                            suffix_rules.push(rule);
                        }
                    }
                    // If we made a new word, add it
                    ret.push((newword, rule, None));
                }
            });

        // Loop our prefixed words that allow suffixes
        let double_matches = prefixed_words.iter().flat_map(|(word, pfxrule)| {
            // Collect suffix rules that match
            suffix_rules.iter().filter_map(|sfxrule| {
                sfxrule
                    .apply_pattern(word, RuleType::Suffix)
                    .map(|newword| (newword, *pfxrule, Some(*sfxrule)))
            })
        });

        ret.extend(double_matches);

        ret
    }
}

#[cfg(test)]
mod tests;
