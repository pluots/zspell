//! Representation of an affix file

mod types;

use std::collections::BTreeMap;
use std::sync::Arc;

pub use self::types::{
    CompoundPattern, CompoundSyllable, Conversion, Encoding, FlagType, PartOfSpeech, Phonetic,
    RuleType,
};
use crate::dict::{AfxRule, DictEntry, FlagValue};
use crate::error::{BuildError, Error, ParseError, ParseErrorKind};
use crate::morph::MorphInfo;
use crate::parser_affix::{parse_affix, AffixNode, ParsedRule, ParsedRuleGroup};

/// A representation of an affix file
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedCfg {
    /*
        General Options
    */
    /// Charset to use, reference to an [`Encoding`] Currently this is
    /// unused; only UTF-8 is supported. However, the affix file must still have
    /// an accurate definition.
    encoding: Encoding,

    /// The type of flag in the `.dic` file
    flag_type: FlagType,

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
    nosuggest_flag: Option<u32>,

    /// Note rare (i.e. commonly misspelled) words with this flag
    warn_rare_flag: Option<u32>,

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
    afx_rule_groups: Vec<ParsedRuleGroup>,

    /*
        Other options
    */
    afx_circumflex_flag: Option<u32>,
    forbidden_word_flag: Option<u32>,
    afx_full_strip: bool,
    afx_keep_case_flag: Option<u32>,
    input_conversions: Vec<Conversion>,
    output_conversions: Vec<Conversion>,
    afx_needed_flag: Option<u32>,
    afx_substandard_flag: Option<u32>,
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
    flag: Option<u32>,

    /// Words with this flag may start a compound
    begin_flag: Option<u32>,

    /// Words with this flag may end a compound
    end_flag: Option<u32>,

    /// Words with this flag may be in the middle of a compound
    middle_flag: Option<u32>,

    /// Words with this flag can't be on their own, only in compounds
    only_flag: Option<u32>,

    /// Allow these words inside compounds
    permit_flag: Option<u32>,
    forbid_flag: Option<u32>,
    more_suffixes: bool,
    root_flag: Option<u32>,
    word_max: u16,
    forbid_dup: bool,
    forbid_repeat: bool,
    check_case: bool,
    check_triple: bool,
    simplify_triple: bool,
    forbid_pats: Vec<CompoundPattern>,
    force_upper_flag: Option<u32>,
    syllable: CompoundSyllable,
    syllable_num: String,
}

impl Default for ParsedCfg {
    #[allow(clippy::default_trait_access)]
    #[inline]
    fn default() -> Self {
        Self {
            encoding: Default::default(),
            flag_type: FlagType::Utf8,
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
            nosuggest_flag: Some('!' as u32),
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
            afx_rule_groups: Default::default(),
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
            root_flag: Default::default(),
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

impl ParsedCfg {
    pub fn flag_type(&self) -> FlagType {
        self.flag_type
    }

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
    #[allow(clippy::too_many_lines)]
    fn from_parsed(v: Vec<AffixNode>) -> Result<Self, Error> {
        let mut res = Self::default();
        let mut warnings: Vec<String> = Vec::new();

        if let Some(node) = v.iter().find(|node| matches!(node, AffixNode::FlagType(_))) {
            if let AffixNode::FlagType(v) = node {
                res.flag_type = *v;
            } else {
                unreachable!()
            }
        }

        for node in v {
            let name_str = node.name_str();
            match node {
                AffixNode::Encoding(v) => res.encoding = v,
                AffixNode::FlagType(v) => (),
                AffixNode::ComplexPrefixes => res.complex_prefixes = true,
                AffixNode::Language(v) => res.lang = v,
                AffixNode::IgnoreChars(v) => res.ignore_chars = v,
                AffixNode::AffixAlias(v) => res.affix_alias = v,
                AffixNode::MorphAlias(v) => res.morph_alias = v,
                AffixNode::NeighborKeys(v) => res.neighbor_keys = v,
                AffixNode::TryCharacters(v) => res.try_characters = v,
                AffixNode::NoSuggestFlag(v) => res.nosuggest_flag = Some(res.convert_flag(&v)?),
                AffixNode::CompoundSugMax(v) => res.compound_config.sug_max = v,
                AffixNode::NGramSugMax(v) => res.ngram_sug_max = v,
                AffixNode::NGramDiffMax(v) => res.ngram_diff_max = v,
                AffixNode::NGramLimitToDiffMax => res.ngram_limit_to_diff_max = true,
                AffixNode::NoSplitSuggestions => res.no_split_suggestions = true,
                AffixNode::KeepTermDots => res.keep_term_dots = true,
                AffixNode::Replacement(v) => res.replacements = v,
                AffixNode::Mapping(v) => res.maps = v,
                AffixNode::Phonetic(v) => res.phonetics = v,
                AffixNode::WarnRareFlag(v) => res.warn_rare_flag = Some(res.convert_flag(&v)?),
                AffixNode::ForbidWarnWords => res.forbid_warn_words = true,
                AffixNode::BreakSeparator(v) => res.compound_config.break_separators = v,
                AffixNode::CompoundRule(v) => res.compound_config.rules = v,
                AffixNode::CompoundMinLen(v) => res.compound_config.min_length = v,
                AffixNode::CompoundFlag(v) => {
                    res.compound_config.flag = Some(res.convert_flag(&v)?);
                }
                AffixNode::CompoundBeginFlag(v) => {
                    res.compound_config.begin_flag = Some(res.convert_flag(&v)?);
                }
                AffixNode::CompoundEndFlag(v) => {
                    res.compound_config.end_flag = Some(res.convert_flag(&v)?);
                }
                AffixNode::CompoundMiddleFlag(v) => {
                    res.compound_config.middle_flag = Some(res.convert_flag(&v)?);
                }
                AffixNode::CompoundOnlyFlag(v) => {
                    res.compound_config.only_flag = Some(res.convert_flag(&v)?);
                }
                AffixNode::CompoundPermitFlag(v) => {
                    res.compound_config.permit_flag = Some(res.convert_flag(&v)?);
                }
                AffixNode::CompoundForbidFlag(v) => {
                    res.compound_config.forbid_flag = Some(res.convert_flag(&v)?);
                }
                AffixNode::CompoundMoreSuffixes => res.compound_config.more_suffixes = true,
                AffixNode::CompoundRootFlag(v) => {
                    res.compound_config.root_flag = Some(res.convert_flag(&v)?);
                }
                AffixNode::CompoundWordMax(v) => res.compound_config.word_max = v,
                AffixNode::CompoundForbidDup => res.compound_config.forbid_dup = true,
                AffixNode::CompoundForbidRepeat => res.compound_config.forbid_repeat = true,
                AffixNode::CompoundCheckCase => res.compound_config.check_case = true,
                AffixNode::CompoundCheckTriple => res.compound_config.check_triple = true,
                AffixNode::CompoundSimplifyTriple => res.compound_config.simplify_triple = true,
                AffixNode::CompoundForbidPats(v) => res.compound_config.forbid_pats = v,
                AffixNode::CompoundForceUpFlag(v) => {
                    res.compound_config.force_upper_flag = Some(res.convert_flag(&v)?);
                }
                AffixNode::CompoundSyllable(v) => res.compound_config.syllable = v,
                AffixNode::SyllableNum(v) => res.compound_config.syllable_num = v,
                AffixNode::Prefix(v) => res.afx_rule_groups.push(v),
                AffixNode::Suffix(v) => res.afx_rule_groups.push(v),
                AffixNode::AfxCircumfixFlag(v) => {
                    res.afx_circumflex_flag = Some(res.convert_flag(&v)?);
                }
                AffixNode::ForbiddenWordFlag(v) => {
                    res.forbidden_word_flag = Some(res.convert_flag(&v)?);
                }
                AffixNode::AfxFullStrip => res.afx_full_strip = true,
                AffixNode::AfxKeepCaseFlag(v) => {
                    res.afx_keep_case_flag = Some(res.convert_flag(&v)?);
                }
                AffixNode::AfxInputConversion(v) => res.input_conversions = v,
                AffixNode::AfxOutputConversion(v) => res.output_conversions = v,
                AffixNode::AfxLemmaPresentFlag(v) => {
                    warnings.push(format!("flag {name_str} is deprecated"));
                }
                AffixNode::AfxNeededFlag(v) => res.afx_needed_flag = Some(res.convert_flag(&v)?),
                AffixNode::AfxPseudoRootFlag(v) => {
                    warnings.push(format!("flag {name_str} is deprecated"));
                }
                AffixNode::AfxSubstandardFlag(v) => {
                    res.afx_substandard_flag = Some(res.convert_flag(&v)?);
                }
                AffixNode::AfxWordChars(v) => res.afx_word_chars = v,
                AffixNode::AfxCheckSharps => res.afx_check_sharps = true,
                AffixNode::Comment => (),
                AffixNode::Name(v) => res.name = v,
                AffixNode::HomePage(v) => res.home_page = v,
                AffixNode::Version(v) => res.version = v,
            }
        }

        for w in warnings {
            eprintln!("warning: {w}");
        }

        Ok(res)
    }

    /// Convert a string to the internal flag type
    pub(crate) fn convert_flag(&self, flag: &str) -> Result<u32, ParseError> {
        self.flag_type
            .str_to_flag(flag)
            .map_err(|e| ParseError::new_nospan(e, flag))
    }

    /// Collect all relevant flags to a map. Returns an error if there are
    /// duplicates
    pub fn compile_flags(&self) -> Result<BTreeMap<u32, FlagValue>, Error> {
        let keysets = [
            (self.afx_circumflex_flag, FlagValue::AfxCircumfix),
            (self.afx_keep_case_flag, FlagValue::AfxKeepCase),
            (self.afx_needed_flag, FlagValue::AfxNeeded),
            (self.afx_substandard_flag, FlagValue::AfxSubstandard),
            (self.compound_config.flag, FlagValue::Compound),
            (self.compound_config.begin_flag, FlagValue::CompoundBegin),
            (self.compound_config.end_flag, FlagValue::CompoundEnd),
            (self.compound_config.forbid_flag, FlagValue::CompoundForbid),
            (
                self.compound_config.force_upper_flag,
                FlagValue::CompoundForceUp,
            ),
            (self.compound_config.middle_flag, FlagValue::CompoundMiddle),
            (self.compound_config.only_flag, FlagValue::CompoundOnly),
            (self.compound_config.permit_flag, FlagValue::CompoundPermit),
            (self.compound_config.root_flag, FlagValue::CompoundRoot),
            (self.forbidden_word_flag, FlagValue::ForbiddenWord),
            (self.nosuggest_flag, FlagValue::NoSuggest),
            (self.warn_rare_flag, FlagValue::WarnRare),
        ];

        let mut map: BTreeMap<u32, FlagValue> = BTreeMap::new();
        let mut morphs: Vec<MorphInfo> = Vec::new();

        for (key, value) in keysets
            .iter()
            .filter_map(|(kopt, val)| kopt.map(|keyval| (keyval, val)))
        {
            // Check for duplicate values
            if let Some(duplicate) = map.get(&key) {
                return Err(BuildError::DuplicateFlag {
                    flag: self.flag_type.flag_to_str(key),
                    t1: duplicate.clone(),
                    t2: Some(value.clone()),
                }
                .into());
            }
            map.insert(key, value.clone());
        }

        for group in &self.afx_rule_groups {
            let flag = self
                .flag_type
                .str_to_flag(&group.flag)
                .map_err(|e| ParseError::new_nospan(e, &group.flag))?;

            // Check for duplicate values
            if let Some(duplicate) = map.get(&flag) {
                return Err(BuildError::DuplicateFlag {
                    flag: group.flag.clone(),
                    t1: duplicate.clone(),
                    t2: None,
                }
                .into());
            }

            let rule = AfxRule::from_parsed_group(self, group);
            map.insert(flag, FlagValue::Rule(Arc::new(rule)));
        }

        Ok(map)
    }
}

/// Indicate a kind of flag

#[cfg(test)]
mod tests;
