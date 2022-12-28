//! Types and implementation of morphological analysis

use crate::affix::PartOfSpeech;
use crate::error::{ParseError, ParseErrorType};

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

impl MorphInfo {
    pub(crate) fn many_from_str(s: &str) -> Result<Vec<Self>, ParseError> {
        let mut res = Vec::new();
        for morph in s.split_whitespace() {
            res.push(MorphInfo::try_from(morph).map_err(|e| ParseError::new_nospan(e, morph))?);
        }
        Ok(res)
    }
}

impl TryFrom<&str> for MorphInfo {
    type Error = ParseErrorType;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (tag, val) = value
            .split_once(':')
            .ok_or_else(|| ParseErrorType::MorphInfoDelim(value.to_owned()))?;
        let ret = match tag {
            "st" => Self::Stem(val.to_owned()),
            "ph" => Self::Phonetic(val.to_owned()),
            "al" => Self::Allomorph(val.to_owned()),
            "po" => Self::Part(val.try_into()?),
            "ds" => Self::DerivSfx(val.to_owned()),
            "is" => Self::InflecSfx(val.to_owned()),
            "ts" => Self::TerminalSfx(val.to_owned()),
            "dp" => Self::DerivPfx(val.to_owned()),
            "ip" => Self::InflecPfx(val.to_owned()),
            "tp" => Self::TermPfx(val.to_owned()),
            "sp" => Self::SurfacePfx(val.to_owned()),
            "pa" => Self::CompPart(val.to_owned()),
            _ => return Err(ParseErrorType::MorphInvalidTag(tag.to_owned())),
        };
        Ok(ret)
    }
}
