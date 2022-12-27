//! Parser representations of an affix file

#![allow(unused)]

use super::ParsedRuleGroup;
use crate::affix::{CompoundPattern, CompoundSyllable, Conversion, Encoding, FlagType, Phonetic};

/// A single line entry in an affix file
///
/// We represent all flags as strings, and parse them later
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq)]
pub enum AffixNode {
    /*
        General ptions
    */
    /// `SET`
    Encoding(Encoding),
    /// `FLAG`
    FlagType(FlagType),
    /// `COMPLEXPREFIXES` twofold prefix stripping
    ComplexPrefixes,
    /// `LANG`
    Language(String),
    /// `IGNORE`
    IgnoreChars(Vec<char>),
    /// `AF`
    AffixAlias(Vec<String>),
    /// `AM`
    MorphAlias(Vec<String>),

    /*
        Suggestion Options
    */
    /// `KEY`
    NeighborKeys(Vec<String>),
    /// `TRY`
    TryCharacters(String),
    /// `NOSUGGEST`
    NoSuggestFlag(String),
    /// `MAXCPDSUGS`
    CompoundSugMax(u16),
    /// `MAXNGRAMSUGS`
    NGramSugMax(u16),
    /// `MAXDIFF`
    NGramDiffMax(u8),
    /// `ONLYMAXDIFF`
    NGramLimitToDiffMax,
    /// `NOSPLITSUGS`
    NoSplitSuggestions,
    /// `SUGSWITHDOTS`
    KeepTermDots,
    /// `REP`
    Replacement(Vec<Conversion>),
    /// `MAP`
    Mapping(Vec<(char, char)>),
    /// `PHONE`
    Phonetic(Vec<Phonetic>),
    /// `WARN`
    WarnRareFlag(String),
    /*
        Compounding Options
    */
    /// `FORBIDWARN`
    ForbidWarnWords,
    /// `BREAK`
    BreakSeparator(Vec<String>),
    /// `COMPOUNDRULE`
    CompoundRule(Vec<String>),
    /// `COMPOUNDMIN`
    CompoundMinLen(u16),
    /// `COMPOUNDFLAG`
    CompoundFlag(String),
    /// `COMPOUNDBEGIN`
    CompoundBeginFlag(String),
    /// `COMPOUNDLAST`
    CompoundEndFlag(String),
    /// `COMPOUNDMIDDLE`
    CompoundMiddleFlag(String),
    /// `ONLYINCOMPOUND`
    CompoundOnlyFlag(String),
    /// `COMPOUNDPERMITFLAG`
    CompoundPermitFlag(String),
    /// `COMPOUNDFORBIDFLAG`
    CompoundForbidFlag(String),
    /// `COMPOUNDMORESUFFIXES`
    CompoundMoreSuffixes,
    /// `COMPOUNDROOT`
    CompoundRootFlag(String),
    /// `COMPOUNDWORDMAX`
    CompoundWordMax(u16),
    /// `CHECKCOMPOUNDDUP`
    CompoundForbidDup,
    /// `CHECKCOMPOUNDREP`
    CompoundForbidRepeat,
    /// `CHECKCOMPOUNDCASE`
    CompoundCheckCase,
    /// `CHECKCOMPOUNDTRIPLE`
    CompoundCheckTriple,
    /// `SIMPLIFIEDTRIPLE`
    CompoundSimplifyTriple,
    /// `CHECKCOMPOUNDPATTERN`
    CompoundForbidPats(Vec<CompoundPattern>),
    /// `FORCEUCASE`
    CompoundForceUpFlag(String),
    /// `COMPOUNDSYLLABLE`
    CompoundSyllable(CompoundSyllable),
    /// `SYLLABLENUM`
    SyllableNum(String),

    /*
        Affix Options
    */
    /// `PFX`
    Prefix(ParsedRuleGroup),
    /// `SFX`
    Suffix(ParsedRuleGroup),

    /*
        Other options
    */
    /// `CIRCUMFIX`
    AfxCircumfixFlag(String),
    /// `FORBIDDENWORD`
    ForbiddenWordFlag(String),
    /// `FULLSTRIP`
    AfxFullStrip,
    /// `KEEPCASE`
    AfxKeepCaseFlag(String),
    /// `ICONV`
    AfxInputConversion(Vec<Conversion>),
    /// `OCONV`
    AfxOutputConversion(Vec<Conversion>),
    /// `LEMMA_PRESENT` this flag is deprecated
    AfxLemmaPresentFlag(String),
    /// `NEEDAFFIX`
    AfxNeededFlag(String),
    /// `PSEUDOROOT` this flag is deprecated
    AfxPseudoRootFlag(String),
    /// `SUBSTANDARD`
    AfxSubstandardFlag(String),
    /// `WORDCHARS`
    AfxWordChars(String),
    /// `CHECKSHARPS`
    AfxCheckSharps,
    /// `#` line
    Comment,
    /// `NAME`
    Name(String),
    /// `HOME`
    HomePage(String),
    /// `VERSION`
    Version(String),
}

impl AffixNode {
    pub const fn name_str(&self) -> &'static str {
        match self {
            AffixNode::Encoding(_) => "SET",
            AffixNode::FlagType(_) => "FLAG",
            AffixNode::ComplexPrefixes => "COMPLEXPREFIXES",
            AffixNode::Language(_) => "LANG",
            AffixNode::IgnoreChars(_) => "IGNORE",
            AffixNode::AffixAlias(_) => "AF",
            AffixNode::MorphAlias(_) => "AM",
            AffixNode::NeighborKeys(_) => "KEY",
            AffixNode::TryCharacters(_) => "TRY",
            AffixNode::NoSuggestFlag(_) => "NOSUGGEST",
            AffixNode::CompoundSugMax(_) => "MAXCPDSUGS",
            AffixNode::NGramSugMax(_) => "MAXNGRAMSUGS",
            AffixNode::NGramDiffMax(_) => "MAXDIFF",
            AffixNode::NGramLimitToDiffMax => "ONLYMAXDIFF",
            AffixNode::NoSplitSuggestions => "NOSPLITSUGS",
            AffixNode::KeepTermDots => "SUGSWITHDOTS",
            AffixNode::Replacement(_) => "REP",
            AffixNode::Mapping(_) => "MAP",
            AffixNode::Phonetic(_) => "PHONE",
            AffixNode::WarnRareFlag(_) => "WARN",
            AffixNode::ForbidWarnWords => "FORBIDWARN",
            AffixNode::BreakSeparator(_) => "BREAK",
            AffixNode::CompoundRule(_) => "COMPOUNDRULE",
            AffixNode::CompoundMinLen(_) => "COMPOUNDMIN",
            AffixNode::CompoundFlag(_) => "COMPOUNDFLAG",
            AffixNode::CompoundBeginFlag(_) => "COMPOUNDBEGIN",
            AffixNode::CompoundEndFlag(_) => "COMPOUNDLAST",
            AffixNode::CompoundMiddleFlag(_) => "COMPOUNDMIDDLE",
            AffixNode::CompoundOnlyFlag(_) => "ONLYINCOMPOUND",
            AffixNode::CompoundPermitFlag(_) => "COMPOUNDPERMITFLAG",
            AffixNode::CompoundForbidFlag(_) => "COMPOUNDFORBIDFLAG",
            AffixNode::CompoundMoreSuffixes => "COMPOUNDMORESUFFIXES",
            AffixNode::CompoundRootFlag(_) => "COMPOUNDROOT",
            AffixNode::CompoundWordMax(_) => "COMPOUNDWORDMAX",
            AffixNode::CompoundForbidDup => "CHECKCOMPOUNDDUP",
            AffixNode::CompoundForbidRepeat => "CHECKCOMPOUNDREP",
            AffixNode::CompoundCheckCase => "CHECKCOMPOUNDCASE",
            AffixNode::CompoundCheckTriple => "CHECKCOMPOUNDTRIPLE",
            AffixNode::CompoundSimplifyTriple => "SIMPLIFIEDTRIPLE",
            AffixNode::CompoundForbidPats(_) => "CHECKCOMPOUNDPATTERN",
            AffixNode::CompoundForceUpFlag(_) => "FORCEUCASE",
            AffixNode::CompoundSyllable(_) => "COMPOUNDSYLLABLE",
            AffixNode::SyllableNum(_) => "SYLLABLENUM",
            AffixNode::Prefix(_) => "PFX",
            AffixNode::Suffix(_) => "SFX",
            AffixNode::AfxCircumfixFlag(_) => "CIRCUMFIX",
            AffixNode::ForbiddenWordFlag(_) => "FORBIDDENWORD",
            AffixNode::AfxFullStrip => "FULLSTRIP",
            AffixNode::AfxKeepCaseFlag(_) => "KEEPCASE",
            AffixNode::AfxInputConversion(_) => "ICONV",
            AffixNode::AfxOutputConversion(_) => "OCONV",
            AffixNode::AfxLemmaPresentFlag(_) => "LEMMA_PRESENT",
            AffixNode::AfxNeededFlag(_) => "NEEDAFFIX",
            AffixNode::AfxPseudoRootFlag(_) => "PSEUDOROOT",
            AffixNode::AfxSubstandardFlag(_) => "SUBSTANDARD",
            AffixNode::AfxWordChars(_) => "WORDCHARS",
            AffixNode::AfxCheckSharps => "CHECKSHARPS",
            AffixNode::Comment => "#",
            AffixNode::Name(_) => "NAME",
            AffixNode::HomePage(_) => "HOME",
            AffixNode::Version(_) => "VERSION",
        }
    }
}
