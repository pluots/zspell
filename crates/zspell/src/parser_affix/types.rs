//! Parser representations of an affix file

#![allow(unused)]

use crate::affix::types::{
    CompoundPattern, CompoundSyllable, Conversion, Encoding, FlagType, Phonetic, RuleGroup,
};

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
    NoSuggestFlag(char),
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
    CompoundMinLen(u16),
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
    CompoundForceUpper(char),
    /// `COMPOUNDSYLLABLE`
    CompoundSyllable(CompoundSyllable),
    /// `SYLLABLENUM`
    SyllableNum(String),

    /*
        Affix Options
    */
    /// `PFX`
    Prefix(RuleGroup),
    /// `SFX`
    Suffix(RuleGroup),

    /*
        Other options
    */
    /// `CIRCUMFIX`
    AfxCircumfixFlag(char),
    /// `FORBIDDENWORD`
    ForbiddenWordFlag(char),
    /// `FULLSTRIP`
    AfxFullStrip,
    /// `KEEPCASE`
    AfxKeepCaseFlag(char),
    /// `ICONV`
    AfxInputConversion(Vec<Conversion>),
    /// `OCONV`
    AfxOutputConversion(Vec<Conversion>),
    /// `LEMMA_PRESENT` this flag is deprecated
    AfxLemmaPresentFlag(char),
    /// `NEEDAFFIX`
    AfxNeededFlag(char),
    /// `PSEUDOROOT` this flag is deprecated
    AfxPseudoRootFlag(char),
    /// `SUBSTANDARD`
    AfxSubstandardFlag(char),
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
