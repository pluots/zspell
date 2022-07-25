// use super::affix::Affix;
// use super::affix_serde::{ENCODING_CLASS_LIST, TOKEN_CLASS_LIST};

use std::string::ToString;

use regex::Regex;
use strum::EnumString;

use crate::affix::{t_data_unwrap, ProcessedToken, ProcessedTokenData};
use crate::errors::AffixError;
use crate::unwrap_or_ret_e;

/// All possible types found in hunspell affix files
/// This represents a generic token type that will have associated
#[non_exhaustive]
#[derive(
    Debug,
    Eq,
    PartialEq,
    EnumString,
    strum_macros::Display,
    strum_macros::EnumVariantNames,
    strum_macros::EnumProperty,
)]
pub enum TokenType {
    #[strum(to_string = "SET", props(dtype = "str"))]
    Encoding,

    #[strum(to_string = "FLAG", props(dtype = "str"))]
    FlagType,

    #[strum(to_string = "COMPLEXPREFIXES", props(dtype = "bool"))]
    ComplexPrefixes,

    #[strum(to_string = "LANG", props(dtype = "str"))]
    Language,

    #[strum(to_string = "IGNORE", props(dtype = "str"))]
    IgnoreChars,

    #[strum(to_string = "AF", props(dtype = "table"))]
    AffixFlag,

    #[strum(to_string = "AM", props(dtype = "table"))]
    MorphAlias,

    // Suggestion-related
    #[strum(to_string = "KEY", props(dtype = "str"))]
    NeighborKeys,

    #[strum(to_string = "TRY", props(dtype = "str"))]
    TryCharacters,

    #[strum(to_string = "NOSUGGEST", props(dtype = "str"))]
    NoSuggestFlag,

    #[strum(to_string = "MAXCPDSUGS", props(dtype = "int"))]
    CompoundSuggestionsMax,

    #[strum(to_string = "MAXNGRAMSUGS", props(dtype = "int"))]
    NGramSuggestionsMax,

    #[strum(to_string = "MAXDIFF", props(dtype = "int"))]
    NGramDiffMax,

    #[strum(to_string = "ONLYMAXDIFF", props(dtype = "bool"))]
    NGramLimitToDiffMax,

    #[strum(to_string = "NOSPLITSUGS", props(dtype = "bool"))]
    NoSpaceSubs,

    #[strum(to_string = "SUGSWITHDOTS", props(dtype = "bool"))]
    KeepTerminationDots,

    #[strum(to_string = "REP", props(dtype = "table"))]
    Replacement,

    #[strum(to_string = "MAP", props(dtype = "table"))]
    Mapping,

    #[strum(to_string = "PHONE", props(dtype = "table"))]
    Phonetic,

    #[strum(to_string = "WARN", props(dtype = "str"))]
    WarnRareFlag,

    #[strum(to_string = "FORBIDWARN", props(dtype = "bool"))]
    ForbidWarnWords,

    #[strum(to_string = "BREAK", props(dtype = "table"))]
    Breakpoint,

    // Compound-related
    #[strum(to_string = "COMPOUNDRULE", props(dtype = "table"))]
    CompoundRule,

    #[strum(to_string = "COMPOUNDMIN", props(dtype = "int"))]
    CompoundMinLength,

    #[strum(to_string = "COMPOUNDFLAG", props(dtype = "str"))]
    CompoundFlag,

    #[strum(to_string = "COMPOUNDBEGIN", props(dtype = "str"))]
    CompoundBeginFlag,

    #[strum(to_string = "COMPOUNDLAST", props(dtype = "str"))]
    CompoundEndFlag,

    #[strum(to_string = "COMPOUNDMIDDLE", props(dtype = "str"))]
    CompoundMiddleFlag,

    #[strum(to_string = "ONLYINCOMPOUND", props(dtype = "str"))]
    CompoundOnlyFlag,

    #[strum(to_string = "COMPOUNDPERMITFLAG", props(dtype = "str"))]
    CompoundPermitFlag,

    #[strum(to_string = "COMPOUNDFORBIDFLAG", props(dtype = "str"))]
    CompoundForbidFlag,

    #[strum(to_string = "COMPOUNDMORESUFFIXES", props(dtype = "bool"))]
    CompoundMoreSuffixes,

    #[strum(to_string = "COMPOUNDROOT", props(dtype = "str"))]
    CompoundRoot,

    #[strum(to_string = "COMPOUNDWORDMAX", props(dtype = "int"))]
    CompoundWordMax,

    #[strum(to_string = "CHECKCOMPOUNDDUP", props(dtype = "bool"))]
    CompoundForbidDuplication,

    #[strum(to_string = "CHECKCOMPOUNDREP", props(dtype = "bool"))]
    CompoundForbidRepeat,

    #[strum(to_string = "CHECKCOMPOUNDCASE", props(dtype = "bool"))]
    CompoundForbidUpperBoundary,

    #[strum(to_string = "CHECKCOMPOUNDTRIPLE", props(dtype = "bool"))]
    CompoundForbidTriple,

    #[strum(to_string = "SIMPLIFIEDTRIPLE", props(dtype = "bool"))]
    CompoundSimplifyTriple,

    #[strum(to_string = "CHECKCOMPOUNDPATTERN", props(dtype = "table"))]
    CompoundForbidPatterns,

    #[strum(to_string = "FORCEUCASE", props(dtype = "str"))]
    CompoundForceUpper,

    #[strum(to_string = "COMPOUNDSYLLABLE", props(dtype = "str"))]
    CompoundForceSyllable,

    #[strum(to_string = "SYLLABLENUM", props(dtype = "str"))]
    CompoundSyllableNumber,

    // Affix-related
    #[strum(to_string = "PFX", props(dtype = "table"))]
    Prefix,

    #[strum(to_string = "SFX", props(dtype = "table"))]
    Suffix,

    #[strum(to_string = "CIRCUMFIX", props(dtype = "str"))]
    AffixCircumfixFlag,

    #[strum(to_string = "FORBIDDENWORD", props(dtype = "str"))]
    AffixForbiddenWordFlag,

    #[strum(to_string = "FULLSTRIP", props(dtype = "bool"))]
    AffixFullStrip,

    #[strum(to_string = "KEEPCASE", props(dtype = "str"))]
    AffixKeepCase,

    #[strum(to_string = "ICONV", props(dtype = "table"))]
    AffixInputConversion,

    #[strum(to_string = "OCONV", props(dtype = "table"))]
    AffixOutputConversion,

    #[strum(to_string = "LEMMA_PRESENT", props(dtype = "str"))]
    AffixLemmaPresentDeprecated,

    #[strum(to_string = "NEEDAFFIX", props(dtype = "str"))]
    AffixNeededFlag,

    #[strum(to_string = "PSEUDOROOT", props(dtype = "str"))]
    AffixPseudoRootFlagDeprecated,

    #[strum(to_string = "SUBSTANDARD", props(dtype = "str"))]
    AffixSubstandardFlag,

    #[strum(to_string = "WORDCHARS", props(dtype = "str"))]
    AffixWordChars,

    #[strum(to_string = "CHECKSHARPS", props(dtype = "bool"))]
    AffixCheckSharps,

    // Used to indicate start of token stream
    FileStart,
}

#[non_exhaustive]
#[derive(
    Debug, Eq, PartialEq, EnumString, strum_macros::Display, strum_macros::EnumVariantNames,
)]
pub enum EncodingType {
    #[strum(to_string = "UTF-8")]
    Utf8, // UTF-8
    #[strum(to_string = "ISO8859-1")]
    Iso8859t1, // ISO8859-1
    #[strum(to_string = "ISO8859-10")]
    Iso8859t10, // ISO8859-10
    #[strum(to_string = "ISO8859-13")]
    Iso8859t13, // ISO8859-13
    #[strum(to_string = "ISO8859-15")]
    Iso8859t15, // ISO8859-15
    #[strum(to_string = "KOI8-R")]
    Koi8r, // KOI8-R
    #[strum(to_string = "KOI8-U")]
    Koi8u, // KOI8-U
    #[strum(to_string = "cp1251")]
    Cp1251, // cp1251
    #[strum(to_string = "ISCII-DEVANAGARI")]
    IsciiDevanagari, // ISCII-DEVANAGARI
}

/// A simple input-to-output conversion mapping.
///
/// This is usually represented in an affix file via `REP`, `ICONV`, and
/// `OCONV`.
#[derive(Debug, PartialEq, Eq)]
pub struct Conversion {
    input: String,
    output: String,
    bidirectional: bool,
}

impl Conversion {
    /// Perform conversion
    ///
    /// # Errors
    #[inline]
    pub fn from_processed_token(
        pt: ProcessedToken,
        bidirectional: bool,
    ) -> Result<Vec<Self>, AffixError> {
        let tab = t_data_unwrap!(pt, Table);
        let mut iter = tab.iter();

        // First line just contains the row count
        iter.next().unwrap();
        let mut ret = Vec::new();

        for row in iter {
            ret.push(Self {
                input: match row.first() {
                    Some(v) => (*v).to_owned(),
                    None => return Err(AffixError::NoConversionInput),
                },
                output: match row.get(1) {
                    Some(v) => (*v).to_owned(),
                    None => return Err(AffixError::NoConversionOutput),
                },
                bidirectional,
            });
        }
        Ok(ret)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RuleType {
    Prefix,
    Suffix,
}

#[derive(Debug)]
pub struct AffixRuleDef {
    atype: RuleType,
    stripping_chars: Option<String>,
    affix: String,
    /// Regex-based rule for when this rule is true
    condition: String,
    morph_info: Vec<String>, // Eventually may need its own type
    /// Compiled version of condition
    condition_re: Option<Regex>,
    /// Shortcut regex checks if this is true
    condition_always_true: bool,
}

// Can't derive because we hold the re_pattern
impl PartialEq for AffixRuleDef {
    fn eq(&self, other: &Self) -> bool {
        self.atype == other.atype
            && self.stripping_chars == other.stripping_chars
            && self.affix == other.affix
            && self.condition == other.condition
            && self.morph_info == other.morph_info
    }
}

// Just need to indicate the `Eq` relation applies
impl Eq for AffixRuleDef {}

impl AffixRuleDef {
    /// Create from the information we would expect to have in a table
    pub fn from_table_creation(
        atype: RuleType,
        strip_text: &str,
        affix_text: &str,
        condition_text: &str,
        morph_info: Vec<String>,
    ) -> Self {
        let mut ruledef = Self {
            atype,
            stripping_chars: match strip_text {
                "" => None,
                "0" => None,
                _ => Some(strip_text.to_owned()),
            },
            affix: affix_text.to_owned(),
            condition: condition_text.to_owned(),
            morph_info,
            condition_re: None,
            condition_always_true: false,
        };

        ruledef.compile_re();

        ruledef
    }

    /// Compile the regex if not yet available
    fn compile_re(&mut self) {
        if &self.condition == "." {
            self.condition_always_true = true;
            return;
        }
        self.condition_always_true = false;

        // Position at start
        let mut re_pattern = String::with_capacity(self.condition.len() + 4);

        re_pattern.push('^');

        // Build the rest of the pattern
        match self.atype {
            RuleType::Prefix => {
                re_pattern.push_str(self.condition.clone().as_str());
                re_pattern.push_str(".*");
            }
            RuleType::Suffix => {
                re_pattern.push_str(".*");
                re_pattern.push_str(self.condition.clone().as_str());
            }
        };

        // Position at end
        re_pattern.push('$');
        self.condition_re = Some(Regex::new(re_pattern.as_str()).unwrap());
    }

    /// See whether the "condition" here is applicable
    pub fn check_condition(&self, s: &str) -> bool {
        if self.condition_always_true {
            return true;
        }
        self.condition_re.as_ref().unwrap().is_match(s)
    }

    // Verify the match condition and apply this rule
    pub fn apply_pattern(&self, s: &str) -> Option<String> {
        // No return if condition doesn't match
        if !self.check_condition(s) {
            return None;
        }

        let mut working = s;

        match self.atype {
            RuleType::Prefix => {
                // If stripping chars exist, strip them from the prefix
                // If not or if no prefix to strip, working is unchanged
                working = match &self.stripping_chars {
                    Some(sc) => match working.strip_prefix(sc) {
                        Some(w) => w,
                        None => working,
                    },
                    None => working,
                };

                let mut w_s = self.affix.clone();
                w_s.push_str(working);
                Some(w_s)
            }
            RuleType::Suffix => {
                // Same logic as above
                working = match &self.stripping_chars {
                    Some(sc) => match working.strip_suffix(sc) {
                        Some(w) => w,
                        None => working,
                    },
                    None => working,
                };
                let mut w_s = working.to_owned();
                w_s.push_str(&self.affix);
                Some(w_s)
            }
        }
    }
}

/// A simple prefix or suffix rule
///
/// This struct represents a prefix or suffix option that may be applied to any
/// base word. It contains multiple possible rule definitions that describe how
/// to apply the rule.
#[derive(Debug, PartialEq, Eq)]
pub struct Rule {
    /// Character identifier for this specific affix, usually any uppercase
    /// letter
    pub key: String,
    /// Prefix or suffix
    pub atype: RuleType,
    /// Whether or not this can be combined with the opposite affix
    pub combine_pfx_sfx: bool,
    /// Actual rules for replacing
    rules: Vec<AffixRuleDef>,
}

impl Rule {
    /// Load the affix rule from a processed token
    ///
    /// # Errors
    ///
    /// Error if unable to load token in
    #[inline]
    pub fn from_processed_token(pt: ProcessedToken) -> Result<Self, AffixError> {
        let tab = t_data_unwrap!(pt, Table);
        let mut iter = tab.iter();

        // First line contains general info about the rule
        let start = iter.next().unwrap();

        let mut ruledefs = Vec::new();

        let atype = match pt.ttype {
            TokenType::Prefix => RuleType::Prefix,
            TokenType::Suffix => RuleType::Suffix,
            _ => return Err(AffixError::BadTokenType),
        };

        // Create rule definitions for that identifier
        for rule in iter {
            let strip_text = unwrap_or_ret_e!(rule.get(1), AffixError::Syntax(rule.join("")));
            let affix_text = unwrap_or_ret_e!(rule.get(2), AffixError::Syntax(rule.join("")));
            let condition = unwrap_or_ret_e!(rule.get(3), AffixError::Syntax(rule.join("")));

            ruledefs.push(AffixRuleDef::from_table_creation(
                atype,
                strip_text,
                affix_text,
                condition,
                rule.as_slice()
                    .get(4..)
                    .expect("Error processing token")
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
            ));
        }

        // Populate with information from the first line
        Ok(Self {
            atype,
            key: (*unwrap_or_ret_e!(start.first(), AffixError::MissingIdentifier)).to_owned(),
            combine_pfx_sfx: match *unwrap_or_ret_e!(start.get(1), AffixError::BadCrossProduct) {
                "Y" => true,
                "N" => false,
                _ => return Err(AffixError::BadCrossProduct),
            },
            rules: ruledefs,
        })
    }

    /// Apply this rule to a root string
    ///
    /// Does not pay attention to prf/sfx combinations, that must be done
    /// earlier.
    #[inline]
    pub fn apply(&self, rootword: &str) -> Option<String> {
        for rule in &self.rules {
            if let Some(applied) = rule.apply_pattern(rootword) {
                return Some(applied);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use super::*;
    use strum::{EnumProperty, VariantNames};

    // Spot check deserialization of encoding
    #[test]
    fn test_encoding_deser() {
        assert_eq!(EncodingType::try_from("UTF-8").unwrap(), EncodingType::Utf8);
        assert_eq!(
            EncodingType::try_from("ISCII-DEVANAGARI").unwrap(),
            EncodingType::IsciiDevanagari
        );
    }

    // Spot check serializatino of encoding
    #[test]
    fn test_encoding_ser() {
        assert_eq!(EncodingType::Utf8.to_string(), "UTF-8");
        assert_eq!(EncodingType::Iso8859t15.to_string(), "ISO8859-15");
    }

    // Spot check deserialization of tokens
    #[test]
    fn test_token_deser() {
        assert_eq!(TokenType::try_from("PFX").unwrap(), TokenType::Prefix);
        assert_eq!(
            TokenType::try_from("KEEPCASE").unwrap(),
            TokenType::AffixKeepCase
        );
    }

    // Spot check serialization of tokens
    #[test]
    fn test_token_ser() {
        assert_eq!(TokenType::IgnoreChars.to_string(), "IGNORE");
        assert_eq!(TokenType::MorphAlias.to_string(), "AM");
        println!("{:?}", TokenType::VARIANTS);
    }

    // Spot check deserialization of tokens
    #[test]
    fn test_token_props() {
        assert_eq!(TokenType::Encoding.get_str("dtype"), Some("str"));
    }

    #[test]
    fn test_rule_def_condition() {
        let mut ard = AffixRuleDef {
            atype: RuleType::Suffix,
            stripping_chars: None,
            affix: "".into(),
            condition: "[^aeiou]y".into(),
            morph_info: Vec::new(),
            condition_re: None,
            condition_always_true: false,
        };
        ard.compile_re();

        // General tests, including with pattern in the middle
        assert_eq!(ard.check_condition("xxxy"), true);
        assert_eq!(ard.check_condition("xxxay"), false);
        assert_eq!(ard.check_condition("xxxyxx"), false);

        // Test with prefix
        ard.condition = "y[^aeiou]".into();
        ard.atype = RuleType::Prefix;
        ard.compile_re();
        assert_eq!(ard.check_condition("yxxx"), true);
        assert_eq!(ard.check_condition("yaxxx"), false);
        assert_eq!(ard.check_condition("xxxyxxx"), false);

        // Test other real rules
        ard.condition = "[sxzh]".into();
        ard.atype = RuleType::Suffix;
        ard.compile_re();
        assert_eq!(ard.check_condition("access"), true);
        assert_eq!(ard.check_condition("abyss"), true);
        assert_eq!(ard.check_condition("accomplishment"), false);
        assert_eq!(ard.check_condition("mmms"), true);
        assert_eq!(ard.check_condition("mmsmm"), false);

        // Check with default condition
        ard.condition = ".".into();
        ard.compile_re();
        assert_eq!(ard.check_condition("xxx"), true);
    }

    #[test]
    fn test_rule_apply() {
        let mut ard = AffixRuleDef {
            atype: RuleType::Suffix,
            stripping_chars: Some("y".into()),
            affix: "zzz".into(),
            condition: "[^aeiou]y".into(),
            morph_info: Vec::new(),
            condition_re: None,
            condition_always_true: false,
        };
        ard.compile_re();

        assert_eq!(ard.apply_pattern("xxxy"), Some("xxxzzz".to_string()));

        ard.atype = RuleType::Prefix;
        ard.condition = "y[^aeiou]".into();
        ard.compile_re();
        assert_eq!(ard.apply_pattern("yxxx"), Some("zzzxxx".to_string()));

        ard.atype = RuleType::Suffix;
        ard.condition = ".".into();
        ard.compile_re();
        assert_eq!(ard.apply_pattern("xxx"), Some("xxxzzz".to_string()));
    }

    #[test]
    fn test_affix_rule_apply_words() {
        let ar = Rule {
            atype: RuleType::Suffix,
            key: "A".into(),
            combine_pfx_sfx: true,
            rules: vec![
                AffixRuleDef::from_table_creation(
                    RuleType::Suffix,
                    "y",
                    "iness",
                    "[^aeiou]y",
                    Vec::new(),
                ),
                AffixRuleDef::from_table_creation(
                    RuleType::Suffix,
                    "0",
                    "ness",
                    "[aeiou]y",
                    Vec::new(),
                ),
                AffixRuleDef::from_table_creation(
                    RuleType::Suffix,
                    "0",
                    "ness",
                    "[^y]",
                    Vec::new(),
                ),
            ],
        };

        assert_eq!(ar.apply("blurry"), Some("blurriness".to_string()));
        assert_eq!(ar.apply("coy"), Some("coyness".to_string()));
        assert_eq!(ar.apply("acute"), Some("acuteness".to_string()));
    }
}
