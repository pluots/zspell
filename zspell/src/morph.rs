//! Types and implementation of morphological analysis

use std::fmt;

use crate::affix::PartOfSpeech;

/// Morphological information about a word, used by analysis methods
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MorphInfo {
    /// `st:` stem word
    Stem(MorphStr),
    /// `ph:` better phonetic transliteration if available
    Phonetic(MorphStr),
    /// `al:` allomorphs (e.g. sing -> sang, sung)
    Allomorph(MorphStr),
    /// `po:` part of speech
    Part(PartOfSpeech),
    /// `ds:` derivational suffix
    DerivSfx(MorphStr),
    /// `is:` inflectional suffix
    InflecSfx(MorphStr),
    /// `ts:` terminal suffix
    TerminalSfx(MorphStr),
    /// `dp:` derivational suffix
    DerivPfx(MorphStr),
    /// `ip:` inflectional suffix
    InflecPfx(MorphStr),
    /// `tp:` terminal suffix
    TermPfx(MorphStr),
    /// `sp:` surface prefix
    SurfacePfx(MorphStr),
    /// `pa:` parts of compound words
    CompPart(MorphStr),
    /// Any unrecognized tag. This will be stored as written (e.g. `foo:bar` is stored as
    /// `foo:bar`, not just `bar`).
    Other(MorphStr),
}

impl MorphInfo {
    /// Parse the kind of string that a dictionary file has, usually something like:
    ///
    /// ```text
    /// po:verb st:rootword ts:abcd
    /// ```
    #[inline]
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn many_from_str(s: &str) -> impl Iterator<Item = Self> + '_ {
        s.split_whitespace().map(MorphInfo::from)
        // FIXME:dict-parser we should be able to handle the hungarian dictionary that
        // has entries like this:
        // üzletág/UmôŇyiYcÇ       üzletágak
        // but I am not sure what that means if it is not morph info...
        // res.push(MorphInfo::try_from(morph).map_err(|e| ParseError::new_nospan(e, morph))?);
    }
}

impl From<&str> for MorphInfo {
    #[inline]
    fn from(value: &str) -> Self {
        let Some((tag, val)) = value.split_once(':') else {
            return Self::Other(value.into());
        };

        match tag {
            "st" => Self::Stem(val.into()),
            "ph" => Self::Phonetic(val.into()),
            "al" => Self::Allomorph(val.into()),
            "po" => Self::Part(val.into()),
            "ds" => Self::DerivSfx(val.into()),
            "is" => Self::InflecSfx(val.into()),
            "ts" => Self::TerminalSfx(val.into()),
            "dp" => Self::DerivPfx(val.into()),
            "ip" => Self::InflecPfx(val.into()),
            "tp" => Self::TermPfx(val.into()),
            "sp" => Self::SurfacePfx(val.into()),
            "pa" => Self::CompPart(val.into()),
            _ => Self::Other(value.into()),
        }
    }
}

impl fmt::Display for MorphInfo {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MorphInfo::Stem(v) => write!(f, "st:{v}"),
            MorphInfo::Phonetic(v) => write!(f, "ph:{v}"),
            MorphInfo::Allomorph(v) => write!(f, "al:{v}"),
            MorphInfo::Part(v) => write!(f, "po:{v}"),
            MorphInfo::DerivSfx(v) => write!(f, "ds:{v}"),
            MorphInfo::InflecSfx(v) => write!(f, "is:{v}"),
            MorphInfo::TerminalSfx(v) => write!(f, "ts:{v}"),
            MorphInfo::DerivPfx(v) => write!(f, "dp:{v}"),
            MorphInfo::InflecPfx(v) => write!(f, "ip:{v}"),
            MorphInfo::TermPfx(v) => write!(f, "tp:{v}"),
            MorphInfo::SurfacePfx(v) => write!(f, "sp:{v}"),
            MorphInfo::CompPart(v) => write!(f, "pa:{v}"),
            MorphInfo::Other(v) => write!(f, "{v}"),
        }
    }
}

/// A string used as part of morphological analysis
///
/// This is a thin wrapper over a native string type to allow us to change
/// the implementation as needed.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MorphStr(Box<str>);

impl AsRef<str> for MorphStr {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<&str> for MorphStr {
    #[inline]
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

impl fmt::Display for MorphStr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Debug for MorphStr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
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
            ("foo:xyz", MorphInfo::Other("foo:xyz".into())),
        ];

        for (input, expected) in tests {
            assert_eq!(MorphInfo::from(input), expected, "failure parsing {input}");
        }
    }

    #[test]
    fn morph_string_ok() {
        let input = "st:stem ip:abcd pa:xyz    st:some-stem\tal:def";
        let output = MorphInfo::many_from_str(input);
        let expected = [
            MorphInfo::Stem("stem".into()),
            MorphInfo::InflecPfx("abcd".into()),
            MorphInfo::CompPart("xyz".into()),
            MorphInfo::Stem("some-stem".into()),
            MorphInfo::Allomorph("def".into()),
        ];

        assert_eq!(&output.collect::<Vec<_>>(), &expected);
    }
}
