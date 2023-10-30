//! Utilities intended to help with test collection
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use pretty_assertions::assert_eq;
use regex::Regex;
use zspell::{DictBuilder, Dictionary, MorphInfo};

/// Get the workspace root. We use this as a workaround because Github actions
/// seems to switch this around for some reason.
pub fn workspace_root() -> PathBuf {
    dbg!(std::env::current_dir().unwrap());
    // use github workspace directory if available, or `../../this_dir` if not
    let ret = match dbg!(std::env::var("GITHUB_WORKSPACE")) {
        Ok(v) => PathBuf::from(v),
        Err(_) => {
            let mut tmp = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            tmp.pop();
            tmp.pop();
            tmp.pop();
            tmp
        }
    };

    let paths = fs::read_dir(&ret).unwrap();
    for path in paths {
        println!("Name: {}", path.unwrap().path().display());
    }
    ret
}

/// A collection from a `.test` file that we can easily validate
///
/// See `0_example.test`  for descriptions of what this file should look like
#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct TestManager {
    description: String,
    fname: String,
    /// The affix file as a string
    afx_str: String,
    /// The dictionary file as a string
    dic_str: String,
    /// Personal dictionary file
    personal_str: String,
    /// These words/sentences will be checked with the check algorithm
    check_valid: Vec<String>,
    /// These words/sentences will be checked
    check_invalid: Vec<String>,
    wordlist: Option<Vec<String>>,
    wl_nosuggest: Option<Vec<String>>,
    wl_forbidden: Option<Vec<String>>,
    wordlist_allow_extra: bool,
    wl_nosuggest_allow_extra: bool,
    wl_forbidden_allow_extra: bool,
    /// Map types
    suggestions: BTreeMap<String, Vec<String>>,
    stems: BTreeMap<String, Vec<String>>,
    morphs: BTreeMap<String, Vec<MorphInfo>>,
}

impl TestManager {
    /// Load a `TestManager` from a string
    pub fn new_from_str(input: &str) -> Self {
        let mut ret = Self::default();
        // Remove comments, which start with "%%"
        let input_cleaned: String = input
            .lines()
            .filter(|line| matches!(determine_line(line), Line::Attribute(_) | Line::Normal(_)))
            .fold(String::new(), |mut a, b| {
                writeln!(a, "{b}").unwrap();
                a
            });
        let mut content_iter = input_cleaned.trim().split("====").filter(|s| !s.is_empty());

        while let Some(s_title) = content_iter.next() {
            let mut sec_attrs = Vec::new();
            let sec_title = s_title.trim();
            // The section content as a single string
            let mut sec_content = String::new();

            // Remove and store attributes, which can be things like `allow-extra` (don't
            // check exhaustive matches)
            for line in content_iter
                .next()
                .expect("Section title with no content")
                .lines()
            {
                match determine_line(line) {
                    Line::Comment => unreachable!(),
                    Line::Attribute(attr) => sec_attrs.push(attr),
                    Line::Normal(s) => writeln!(sec_content, "{s}").unwrap(),
                }
            }

            // Iterator over lines (just a helper)
            let lines_content: Vec<_> = sec_content
                .trim()
                .lines()
                .map(|line| line.to_owned())
                .collect();

            match sec_title {
                "afx" => ret.afx_str = sec_content.to_owned(),
                "dic" => ret.dic_str = sec_content.to_owned(),
                "personal" => ret.personal_str = sec_content.to_owned(),
                "valid" => ret.check_valid = lines_content,
                "invalid" => ret.check_invalid = lines_content,
                "wordlist" => {
                    ret.wordlist = Some(lines_content);
                    for attr in sec_attrs {
                        if attr == "allow-extra" {
                            ret.wordlist_allow_extra = true;
                        } else {
                            panic!("unknown attribute {attr}");
                        }
                    }
                }
                "nosuggest" => {
                    ret.wl_nosuggest = Some(lines_content);
                    for attr in sec_attrs {
                        if attr == "allow-extra" {
                            ret.wl_nosuggest_allow_extra = true;
                        } else {
                            panic!("unknown attribute {attr}");
                        }
                    }
                }
                "forbidden" => {
                    ret.wl_forbidden = Some(lines_content);
                    for attr in sec_attrs {
                        if attr == "allow-extra" {
                            ret.wl_forbidden_allow_extra = true;
                        } else {
                            panic!("unknown attribute {attr}");
                        }
                    }
                }
                "suggest" => {
                    ret.suggestions =
                        parse_map(&sec_content).unwrap_or_else(|e| ret.panic_with_ctx(&e))
                }
                "stem" => {
                    ret.stems = parse_map(&sec_content).unwrap_or_else(|e| ret.panic_with_ctx(&e))
                }
                "morph" => {
                    let tmp = parse_map(&sec_content).unwrap_or_else(|e| ret.panic_with_ctx(&e));
                    // Turn string morph indicators into MorphInfo
                    ret.morphs = tmp
                        .into_iter()
                        .map(|(k, v)| (k, v.into_iter().map(|v| v.as_str().into()).collect()))
                        .collect();
                }
                "end" => break,
                other => ret.panic_with_ctx(&format!("bad section heading '{other}'")),
            };
        }

        ret
    }

    /// Load a `TestManager` from a given file name. Assumes the file will be
    /// located in `zspell/tests/files`.
    pub fn new_from_file(path: impl AsRef<Path>) -> Self {
        let path_str = path.as_ref().to_string_lossy();
        let f_content =
            fs::read_to_string(&path).unwrap_or_else(|_| panic!("error reading file '{path_str}'"));

        let mut ret = Self::new_from_str(&f_content);
        ret.fname = path_str.into_owned();
        ret
    }

    pub fn panic_with_ctx(&self, msg: &str) -> ! {
        if std::env::var_os("SHOW_CTX").is_some() {
            panic!("{msg}. Collection:\n{self:#?}\n");
        } else {
            panic!("{msg}");
        }
    }

    pub fn panic_with_dict(&self, dict: &Dictionary, msg: &str) -> ! {
        if std::env::var_os("SHOW_DICT").is_some() {
            panic!("{msg}. Collection:\n{self:#?}\nDictionary:\n{dict:#?}\n");
        } else {
            self.panic_with_ctx(msg);
        }
    }

    /// Build the dictionary based on given input
    pub fn build_dict(&self) -> Dictionary {
        let mut builder = DictBuilder::new()
            .config_str(&self.afx_str)
            .dict_str(&self.dic_str);

        if !self.personal_str.is_empty() {
            builder = builder.personal_str(&self.personal_str);
        }

        builder.build().expect("error building dictionary")
    }

    /// Check everything in the file against our dictionary
    ///
    /// Panics with a message if there are any failures
    pub fn check_all(&self, dict: &Dictionary) {
        self.run_check_valid_invalid(dict);
        self.check_wordlists(dict);
        self.check_suggestions(dict);
        self.check_stems(dict);
        self.check_analysis(dict);
    }

    /// Validate all expected checks are correct
    fn run_check_valid_invalid(&self, dict: &Dictionary) {
        let valid_failures: Vec<_> = self
            .check_valid
            .iter()
            .filter(|item| !dict.check(item))
            .collect();

        let invalid_failures: Vec<_> = self
            .check_invalid
            .iter()
            .filter(|item| dict.check(item))
            .collect();

        let mut valid_fail_msg = None;
        let mut invalid_fail_msg = None;

        if !valid_failures.is_empty() {
            let msg = format!(
                "valid check failed in '{}'. missing: {valid_failures:#?}\n",
                self.fname
            );
            valid_fail_msg = Some(msg);
        } else if self.check_valid.is_empty() {
            eprintln!("Skipped check_valid testing")
        } else {
            eprintln!("Verified {} items as true", self.check_valid.len());
        }

        if !invalid_failures.is_empty() {
            let msg = format!(
                "invalid check failed in '{}'. missing: {invalid_failures:#?}\n",
                self.fname
            );
            invalid_fail_msg = Some(msg);
        } else if self.check_invalid.is_empty() {
            eprintln!("Skipped check_invalid testing")
        } else {
            eprintln!("Verified {} items as false", self.check_invalid.len());
        }

        if valid_fail_msg.is_some() || invalid_fail_msg.is_some() {
            panic!(
                "{}{}",
                valid_fail_msg.unwrap_or_default(),
                invalid_fail_msg.unwrap_or_default()
            );
        }
    }

    /// Validate all our word lists are equal
    fn check_wordlists(&self, dict: &Dictionary) {
        let check_lists = [
            (
                "wordlist",
                &self.wordlist,
                self.wordlist_allow_extra,
                dict.wordlist(),
            ),
            (
                "wordlist_nosuggest",
                &self.wl_nosuggest,
                self.wl_nosuggest_allow_extra,
                dict.wordlist_nosuggest(),
            ),
            (
                "wordlist_forbidden",
                &self.wl_forbidden,
                self.wl_forbidden_allow_extra,
                dict.wordlist_forbidden(),
            ),
        ];

        for (name, expected_ref, allow_extra, actual_ref) in check_lists.into_iter() {
            let Some(expected_ref) = expected_ref else {
                eprintln!("skipping {name} test");
                continue;
            };

            let mut expected = expected_ref.clone();
            expected.sort_unstable();

            let mut actual: Vec<String> = actual_ref
                .inner()
                .keys()
                .map(|v| v.as_ref().into())
                .collect();
            actual.sort_unstable();

            if allow_extra {
                for word in expected {
                    assert!(
                        actual.contains(&word),
                        "failed {name} checks for '{}': missing {word}",
                        self.fname
                    );
                }
            } else {
                assert_eq!(
                    expected, actual,
                    "failed {name} checks for '{}'",
                    self.fname
                );
            }
            eprintln!("testing for {name} succeeded");
        }
    }

    /// Check all provided suggestions
    fn check_suggestions(&self, dict: &Dictionary) {
        for (input, expected) in &self.suggestions {
            let entry = dict.entry(input);
            let mut sug_dict = entry.suggest().unwrap_or_else(|| {
                self.panic_with_dict(dict, &format!("no suggestions '{input}'"))
            });
            let mut sug_exp: Vec<&str> = expected.iter().map(|s| s.as_str()).collect();
            sug_dict.sort_unstable();
            sug_exp.sort_unstable();
            assert_eq!(
                sug_dict, sug_exp,
                "failed suggestion checks for '{}'",
                self.fname
            );
        }
        eprintln!("all suggestions passed");
    }

    /// Check stemming
    fn check_stems(&self, dict: &Dictionary) {
        for (input, expected) in &self.stems {
            let entry = dict.entry(input);
            let mut stem_dict: Vec<&str> = entry
                .stems()
                .unwrap_or_else(|| self.panic_with_dict(dict, &format!("no stems for '{input}'")))
                .collect();
            let mut stem_exp: Vec<&str> = expected.iter().map(|s| s.as_str()).collect();
            stem_dict.sort_unstable();
            stem_exp.sort_unstable();
            assert_eq!(
                stem_dict, stem_exp,
                "failed stemming checks for '{input}' in '{}'",
                self.fname
            );
        }
        eprintln!("all stems passed");
    }

    /// Check morph analysis
    fn check_analysis(&self, dict: &Dictionary) {
        for (input, expected) in &self.morphs {
            let entry = dict.entry(input);
            let mut morph_dict: Vec<_> = entry
                .analyze()
                .unwrap_or_else(|| {
                    self.panic_with_dict(dict, &format!("no analysis for '{input}'"))
                })
                .collect();
            let mut morph_exp: Vec<_> = expected.iter().collect();
            morph_dict.sort_unstable();
            morph_exp.sort_unstable();
            assert_eq!(
                morph_dict, morph_exp,
                "failed morph checks for '{input}' in '{}'",
                self.fname
            );
        }
        eprintln!("all morphs passed");
    }

    pub fn afx_str(&self) -> &str {
        self.afx_str.as_str()
    }

    pub fn dic_str(&self) -> &str {
        self.dic_str.as_str()
    }

    pub fn check_valid(&self) -> &[String] {
        &self.check_valid
    }

    pub fn check_invalid(&self) -> &[String] {
        &self.check_invalid
    }

    pub fn wordlist(&self) -> &[String] {
        self.wordlist.as_deref().unwrap_or_default()
    }

    pub fn wordlist_nosuggest(&self) -> &[String] {
        self.wl_nosuggest.as_deref().unwrap_or_default()
    }

    pub fn wordlist_forbidden(&self) -> &[String] {
        self.wl_forbidden.as_deref().unwrap_or_default()
    }

    pub fn suggestions(&self) -> &BTreeMap<String, Vec<String>> {
        &self.suggestions
    }
}

/// What the contents of a line hold
enum Line<'a> {
    /// A comment, ignore this line
    Comment,
    /// An attribute, do something with the value but ignore this line
    Attribute(&'a str),
    /// Normal contents
    Normal(&'a str),
}

fn determine_line(line: &str) -> Line {
    const ATTR_RE: &str = r"\s*%%\s*attr:(.*)";
    const CMT_RE: &str = r"\s*%%.*";
    static ATTR: OnceLock<Regex> = OnceLock::new();
    static CMT: OnceLock<Regex> = OnceLock::new();

    let attr = ATTR.get_or_init(|| Regex::new(ATTR_RE).unwrap());
    let cmt = CMT.get_or_init(|| Regex::new(CMT_RE).unwrap());

    if let Some(caps) = attr.captures(line) {
        Line::Attribute(caps.get(1).unwrap().as_str().trim())
    } else if cmt.is_match(line) {
        Line::Comment
    } else {
        Line::Normal(line)
    }
}

/// Parse maps that look like `appl > apple | Apfel | app` into
/// `{"appl": ["apple", "Apfel", "app"]}`
fn parse_map(input: &str) -> Result<BTreeMap<String, Vec<String>>, String> {
    let mut map = BTreeMap::new();
    for (idx, line) in input.lines().filter(|s| !s.trim().is_empty()).enumerate() {
        let Some((key, values)) = line.split_once('>') else {
            return Err(format!("bad mapping at line {idx} in:\n{input}"));
        };
        let values = values
            .split_whitespace()
            .map(|s| s.trim().to_owned())
            .collect();
        let key = key.trim().into();
        assert!(!map.contains_key(&key), "key '{key}' specified twice");
        map.insert(key, values);
    }

    Ok(map)
}
