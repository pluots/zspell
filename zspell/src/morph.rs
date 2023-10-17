//! Types and implementation of morphological analysis

use crate::affix::PartOfSpeech;
use crate::error::{ParseError, ParseErrorKind};

/// Morphographical information about a word, used by analysis methods
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MorphInfo {
    /// `st:` stem word
    Stem(Box<str>),
    /// `ph:` better phonetic transliteration if available
    Phonetic(Box<str>),
    /// `al:` allomorphs (e.g. sing -> sang, sung)
    Allomorph(Box<str>),
    /// `po:` part of speech
    Part(PartOfSpeech),
    /// `ds:` derivational suffix
    DerivSfx(Box<str>),
    /// `is:` inflectional suffix
    InflecSfx(Box<str>),
    /// `ts:` terminal suffix
    TerminalSfx(Box<str>),
    /// `dp:` derivational suffix
    DerivPfx(Box<str>),
    /// `ip:` inflectional suffix
    InflecPfx(Box<str>),
    /// `tp:` terminal suffix
    TermPfx(Box<str>),
    /// `sp:` surface prefix
    SurfacePfx(Box<str>),
    /// `pa:` parts of compound words
    CompPart(Box<str>),
}

impl MorphInfo {
    /// Parse the kind of string that a dictionary file has, usually something like:
    ///
    /// ```text
    /// po:verb st:rootword ts:abcd
    /// ```
    #[inline]
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn many_from_str(s: &str) -> Result<Vec<Self>, ParseError> {
        let mut res = Vec::new();
        for morph in s.split_whitespace() {
            if let Ok(v) = MorphInfo::try_from(morph) {
                res.push(v);
            }
            // FIXME: we should be able to handle the hungarian dictionary that
            // has entries like this:
            // üzletág/UmôŇyiYcÇ       üzletágak
            // but I am not sure what that means if it is not morph info...
            // res.push(MorphInfo::try_from(morph).map_err(|e| ParseError::new_nospan(e, morph))?);
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
            "st" => Self::Stem(val.into()),
            "ph" => Self::Phonetic(val.into()),
            "al" => Self::Allomorph(val.into()),
            "po" => Self::Part(val.try_into()?),
            "ds" => Self::DerivSfx(val.into()),
            "is" => Self::InflecSfx(val.into()),
            "ts" => Self::TerminalSfx(val.into()),
            "dp" => Self::DerivPfx(val.into()),
            "ip" => Self::InflecPfx(val.into()),
            "tp" => Self::TermPfx(val.into()),
            "sp" => Self::SurfacePfx(val.into()),
            "pa" => Self::CompPart(val.into()),
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
            ("st:stem", MorphInfo::Stem("stem".into())),
            ("ip:abc", MorphInfo::InflecPfx("abc".into())),
            ("pa:xyz", MorphInfo::CompPart("xyz".into())),
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
            MorphInfo::Stem("stem".into()),
            MorphInfo::InflecPfx("abcd".into()),
            MorphInfo::CompPart("xyz".into()),
            MorphInfo::Stem("some-stem".into()),
            MorphInfo::Allomorph("def".into()),
        ];

        assert_eq!(output, Ok(expected));
    }
}
