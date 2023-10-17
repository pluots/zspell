//! Types and implementation of morphological analysis

use crate::affix::PartOfSpeech;
use crate::error::{ParseError, ParseErrorKind};

/// Morphographical information about a word, used by analysis methods
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
    /// Parse the kind of string that a dictionary file has, usually something like:
    ///
    /// ```text
    /// po:verb st:rootword ts:abcd
    /// ```
    #[inline]
    pub(crate) fn many_from_str(s: &str) -> Result<Vec<Self>, ParseError> {
        let mut res = Vec::new();
        for morph in s.split_whitespace() {
            res.push(MorphInfo::try_from(morph).map_err(|e| ParseError::new_nospan(e, morph))?);
        }
        Ok(res)
    }
}

impl TryFrom<&str> for MorphInfo {
    type Error = ParseErrorKind;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (tag, val) = value
            .split_once(':')
            .ok_or_else(|| ParseErrorKind::MorphInfoDelim(value.to_owned()))?;
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
            _ => return Err(ParseErrorKind::MorphInvalidTag(tag.to_owned())),
        };
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn morph_single_ok() {
        let tests = [
            ("st:stem", MorphInfo::Stem("stem".to_owned())),
            ("ip:abc", MorphInfo::InflecPfx("abc".to_owned())),
            ("pa:xyz", MorphInfo::CompPart("xyz".to_owned())),
        ];

        for (input, expected) in tests {
            assert_eq!(
                MorphInfo::try_from(input),
                Ok(expected),
                "failure parsing {input}"
            );
        }
    }

    #[test]
    fn morph_string_ok() {
        let input = "st:stem ip:abcd pa:xyz    st:some-stem\tal:def";
        let output = MorphInfo::many_from_str(input);
        let expected = vec![
            MorphInfo::Stem("stem".to_owned()),
            MorphInfo::InflecPfx("abcd".to_owned()),
            MorphInfo::CompPart("xyz".to_owned()),
            MorphInfo::Stem("some-stem".to_owned()),
            MorphInfo::Allomorph("def".to_owned()),
        ];

        assert_eq!(output, Ok(expected));
    }
}
