//! Type representations for affix file contents

/// A possible encoding type
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq)]
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
#[derive(Debug, PartialEq, Eq)]
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
#[derive(Debug, PartialEq, Eq)]
pub struct Conversion {
    pub(crate) input: String,
    pub(crate) output: String,
    pub(crate) bidirectional: bool,
}

#[derive(Debug, PartialEq, Eq)]
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
#[derive(Debug, PartialEq, Eq)]
pub struct RuleGroup {
    /// Character identifier for this specific affix, usually any uppercase
    /// letter
    key: String,
    /// Prefix or suffix
    atype: RuleType,
    /// Whether or not this can be combined with the opposite affix
    can_combine: bool,
    /// Actual rules for replacing
    rules: Vec<AffixRule>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AffixRule {
    /// Characters to remove from the beginning or end
    stripping_chars: Option<String>,
    /// Affix to be added
    affix: String,
    /// Regex-based rule for when this rule is true
    condition: String,
    /// Morphological information
    morph_info: Vec<MorphInfo>,
    /// Shortcut regex checks if this is true
    condition_always_true: bool,
}

#[derive(Debug, PartialEq, Eq)]
enum PartOfSpeech {
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
#[derive(Debug, PartialEq, Eq)]
enum MorphInfo {
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

#[derive(Debug, PartialEq, Eq)]
pub struct Phonetic {
    pub(super) pattern: String,
    pub(super) replace: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CompoundPattern {
    pub(super) endchars: String,
    pub(super) endflag: Option<String>,
    pub(super) beginchars: String,
    pub(super) beginflag: Option<String>,
    pub(super) replacement: Option<String>,
}
