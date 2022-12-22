//! Parser representations of an affix file

#![allow(unused)]

use crate::affix::types::{
    CompoundPattern, CompoundSyllable, Conversion, Encoding, Flag, Phonetic, RuleGroup,
};

#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum AffixNode {
    /*
        General ptions
    */
    /// `SET`
    Encoding(Encoding),
    /// `FLAG`
    FlagType(Flag),
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
    NoSuggestFlag(char),
    /// `MAXCPDSUGS`
    CompoundSuggestionsMax(u16),
    /// `MAXNGRAMSUGS`
    NGramSuggestionsMax(u16),
    /// `MAXDIFF`
    NGramDiffMax(u8),
    /// `ONLYMAXDIFF`
    NGramLimitToDiffMax,
    /// `NOSPLITSUGS`
    NoSplitSuggestions,
    /// `SUGSWITHDOTS`
    KeepTerminationDots,
    /// `REP`
    Replacement(Vec<Conversion>),
    /// `MAP`
    Mapping(Vec<(char, char)>),
    /// `PHONE`
    Phonetic(Vec<Phonetic>),
    /// `WARN`
    WarnRareFlag(char),
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
    CompoundMinLength(u16),
    /// `COMPOUNDFLAG`
    CompoundFlag(char),
    /// `COMPOUNDBEGIN`
    CompoundBeginFlag(char),
    /// `COMPOUNDLAST`
    CompoundEndFlag(char),
    /// `COMPOUNDMIDDLE`
    CompoundMiddleFlag(char),
    /// `ONLYINCOMPOUND`
    CompoundOnlyFlag(char),
    /// `COMPOUNDPERMITFLAG`
    CompoundPermitFlag(char),
    /// `COMPOUNDFORBIDFLAG`
    CompoundForbidFlag(char),
    /// `COMPOUNDMORESUFFIXES`
    CompoundMoreSuffixes,
    /// `COMPOUNDROOT`
    CompoundRoot(char),
    /// `COMPOUNDWORDMAX`
    CompoundWordMax(u16),
    /// `CHECKCOMPOUNDDUP`
    CompoundForbidDuplication,
    /// `CHECKCOMPOUNDREP`
    CompoundForbidRepeat,
    /// `CHECKCOMPOUNDCASE`
    CompoundCheckCase,
    /// `CHECKCOMPOUNDTRIPLE`
    CompoundCheckTriple,
    /// `SIMPLIFIEDTRIPLE`
    CompoundSimplifyTriple,
    /// `CHECKCOMPOUNDPATTERN`
    CompoundForbidPatterns(Vec<CompoundPattern>),
    /// `FORCEUCASE`
    CompoundForceUpper(char),
    /// `COMPOUNDSYLLABLE`
    CompoundSyllable(CompoundSyllable),
    /// `SYLLABLENUM`
    SyllableNum(String),

    /*
        Affix Options
    */
    /// `PFX`
    Prefix(Vec<RuleGroup>),
    /// `SFX`
    Suffix(Vec<RuleGroup>),

    /*
        Other options
    */
    /// `CIRCUMFIX`
    AffixCircumfixFlag(char),
    /// `FORBIDDENWORD`
    ForbiddenWordFlag(char),
    /// `FULLSTRIP`
    AffixFullStrip,
    /// `KEEPCASE`
    AffixKeepCaseFlag(char),
    /// `ICONV`
    AffixInputConversion(Vec<Conversion>),
    /// `OCONV`
    AffixOutputConversion(Vec<Conversion>),
    /// `LEMMA_PRESENT` this flag is deprecated
    AffixLemmaPresentFlag(char),
    /// `NEEDAFFIX`
    AffixNeededFlag(char),
    /// `PSEUDOROOT` this flag is deprecated
    AffixPseudoRootFlag(char),
    /// `SUBSTANDARD`
    AffixSubstandardFlag(char),
    /// `WORDCHARS`
    AffixWordChars(String),
    /// `CHECKSHARPS`
    AffixCheckSharps,
    /// `#` line
    Comment,
    /// `NAME`
    Name(String),
    /// `HOME`
    HomePage(String),
    /// `VERSION`
    Version(String),
}
