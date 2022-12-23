//! Representation of an affix file

pub(crate) mod types;
pub(crate) mod types_impl;

use self::types::{
    CompoundPattern, CompoundSyllable, Conversion, Encoding, Flag, Phonetic, RuleGroup,
};
use crate::parser_affix::types::AffixNode;

#[derive(Default, Debug)]
pub struct Config {
    /*
        General Options
    */
    /// Charset to use, reference to an [`EncodingType`] Currently this is
    /// unused; only UTF-8 is supported. However, the affix file must still have
    /// an accurate definition.
    encoding: Encoding,

    /// The type of flag in the `.dic` file
    flagtype: Flag,

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

    /// Maximum compound word suggestions
    compound_sug_max: u16,

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

        Still quite a few missing from this section
    */
    /// Something like `-` to indicate whether both sides should be checked
    /// Prefer COMPOUNDRULE instead
    break_separators: Vec<String>,

    /// Regex-like rules for compound words
    compound_rules: Vec<String>,

    /// Minimum length of words used in a compound
    compound_min_length: u16,

    /// Words with this flag may be in compounds
    compound_flag: Option<char>,

    /// Words with this flag may start a compound
    compound_begin_flag: Option<char>,

    /// Words with this flag may end a compound
    compound_end_flag: Option<char>,

    /// Words with this flag may be in the middle of a compound
    compound_middle_flag: Option<char>,

    /// Words with this flag can't be on their own, only in compounds
    compound_only_flag: Option<char>,

    /// Allow these words inside compounds
    compound_permit_flag: Option<char>,
    compound_forbid_flag: Option<char>,
    compound_more_suffixes: bool,
    compound_root: Option<char>,
    compound_word_max: u16,
    compound_forbid_dup: bool,
    compound_forbid_repeat: bool,
    compound_check_case: bool,
    compound_check_triple: bool,
    compound_simplify_triple: bool,
    compound_forbid_pats: Vec<CompoundPattern>,
    compound_force_upper_flag: Option<char>,
    compound_syllable: CompoundSyllable,
    syllable_num: String,

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

impl Config {
    fn from_parsed(v: Vec<AffixNode>) -> Self {
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
                AffixNode::CompoundSugMax(v) => res.compound_sug_max = v,
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
                AffixNode::BreakSeparator(v) => res.break_separators = v,
                AffixNode::CompoundRule(v) => res.compound_rules = v,
                AffixNode::CompoundMinLen(v) => res.compound_min_length = v,
                AffixNode::CompoundFlag(v) => res.compound_flag = Some(v),
                AffixNode::CompoundBeginFlag(v) => res.compound_begin_flag = Some(v),
                AffixNode::CompoundEndFlag(v) => res.compound_end_flag = Some(v),
                AffixNode::CompoundMiddleFlag(v) => res.compound_middle_flag = Some(v),
                AffixNode::CompoundOnlyFlag(v) => res.compound_only_flag = Some(v),
                AffixNode::CompoundPermitFlag(v) => res.compound_permit_flag = Some(v),
                AffixNode::CompoundForbidFlag(v) => res.compound_forbid_flag = Some(v),
                AffixNode::CompoundMoreSuffixes => res.compound_more_suffixes = true,
                AffixNode::CompoundRoot(v) => res.compound_root = Some(v),
                AffixNode::CompoundWordMax(v) => res.compound_word_max = v,
                AffixNode::CompoundForbidDup => res.compound_forbid_dup = true,
                AffixNode::CompoundForbidRepeat => res.compound_forbid_repeat = true,
                AffixNode::CompoundCheckCase => res.compound_check_case = true,
                AffixNode::CompoundCheckTriple => res.compound_check_triple = true,
                AffixNode::CompoundSimplifyTriple => res.compound_simplify_triple = true,
                AffixNode::CompoundForbidPats(v) => res.compound_forbid_pats = v,
                AffixNode::CompoundForceUpper(v) => res.compound_force_upper_flag = Some(v),
                AffixNode::CompoundSyllable(v) => res.compound_syllable = v,
                AffixNode::SyllableNum(v) => res.syllable_num = v,
                AffixNode::Prefix(v) => res.affix_rules.push(v),
                AffixNode::Suffix(v) => res.affix_rules.push(v),
                AffixNode::AfxCircumfixFlag(v) => res.afx_circumflex_flag = Some(v),
                AffixNode::ForbiddenWordFlag(v) => res.forbidden_word_flag = Some(v),
                AffixNode::AfxFullStrip => res.afx_full_strip = true,
                AffixNode::AfxKeepCaseFlag(v) => res.afx_keep_case_flag = v,
                AffixNode::AfxInputConversion(v) => res.input_conversions = v,
                AffixNode::AfxOutputConversion(v) => res.output_conversions = v,
                AffixNode::AfxLemmaPresentFlag(v) => {
                    warnings.push(format!("flag {name_str} is deprecated"))
                }
                AffixNode::AfxNeededFlag(v) => res.afx_needed_flag = v,
                AffixNode::AfxPseudoRootFlag(v) => {
                    warnings.push(format!("flag {name_str} is deprecated"))
                }
                AffixNode::AfxSubstandardFlag(v) => res.afx_substandard_flag = v,
                AffixNode::AfxWordChars(v) => res.afx_word_chars = v,
                AffixNode::AfxCheckSharps => res.afx_check_sharps = true,
                AffixNode::Comment => todo!(),
                AffixNode::Name(v) => res.name = v,
                AffixNode::HomePage(v) => res.home_page = v,
                AffixNode::Version(v) => res.version = v,
            }
        }

        res
    }
}
