//! Type representations for affix file contents

/// A possible encoding type
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Encoding {
    /// UTF-8 encoding
    Utf8,
    /// ISO8859-1 encoding
    Iso8859t1,
    /// ISO8859-10 encoding
    Iso8859t10,
    /// ISO8859-13 encoding
    Iso8859t13,
    /// ISO8859-15 encoding
    Iso8859t15,
    /// KOI8-R encoding
    Koi8R,
    /// KOI8-U encoding
    Koi8U,
    /// cp1251 encoding
    Cp1251,
    /// ISCII-DEVANAGARI encoding
    IsciiDevanagari,
}

/// A representation of the flag type (the part after `/` in the `.dic` file)
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Flag {
    /// ASCII flags (default)
    Ascii,
    /// UTF8 flags
    Utf8,
    /// Double extended ASCII flags
    Long,
    /// Decimal flag type
    Number,
}

/// A simple input-to-output conversion mapping.
///
/// This is usually represented in an affix file via `REP`, `ICONV`, and
/// `OCONV`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conversion {
    pub(crate) input: String,
    pub(crate) output: String,
    pub(crate) bidirectional: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct CompoundSyllable {
    pub(super) count: u16,
    pub(super) vowels: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RuleType {
    Prefix,
    Suffix,
}

/// A simple prefix or suffix rule
///
/// This struct represents a prefix or suffix option that may be applied to any
/// base word. It contains multiple possible rule definitions that describe how
/// to apply the rule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleGroup {
    /// Character identifier for this specific affix, usually any uppercase
    /// letter
    pub(crate) flag: String,
    /// Prefix or suffix
    pub(crate) kind: RuleType,
    /// Whether or not this can be combined with the opposite affix
    pub(crate) can_combine: bool,
    /// Actual rules for replacing
    pub(crate) rules: Vec<AffixRule>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AffixRule {
    /// Characters to remove from the beginning or end
    pub(crate) stripping_chars: Option<String>,
    /// Affix to be added
    pub(crate) affix: String,
    /// Regex-based rule for when this rule is true
    pub(crate) condition: Option<String>,
    /// Morphological information
    pub(crate) morph_info: Option<Vec<MorphInfo>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PartOfSpeech {
    Noun,
    Verb,
    Adjective,
    Determiner,
    Adverb,
    Pronoun,
    Preposition,
    Conjunction,
    Interjection,
}

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MorphInfo {
    /// `st:` stem word
    Stem(String),
    /// `ph:` better phonetic transliteration if available
    Phonetic(String),
    /// `al:` allomorphs (e.g. sing -> sang, sung)
    Allomorph(String),
    /// `po:` part of speech
    Part(PartOfSpeech),
    /// `ds:` derivational suffix
    DerivSfx(String),
    /// `is:` inflectional suffix
    InflecSfx(String),
    /// `ts:` terminal suffix
    TerminalSfx(String),
    /// `dp:` derivational suffix
    DerivPfx(String),
    /// `ip:` inflectional suffix
    InflecPfx(String),
    /// `tp:` terminal suffix
    TermPfx(String),
    /// `sp:` surface prefix
    SurfacePfx(String),
    /// `pa:` parts of compound words
    CompPart(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phonetic {
    pub(crate) pattern: String,
    pub(crate) replace: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompoundPattern {
    pub(crate) endchars: String,
    pub(crate) endflag: Option<String>,
    pub(crate) beginchars: String,
    pub(crate) beginflag: Option<String>,
    pub(crate) replacement: Option<String>,
}
