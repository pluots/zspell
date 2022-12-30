//! Utilities intended to help with test collection
#![allow(unused)]

use std::fs;
use std::path::PathBuf;

use pretty_assertions::{assert_eq, assert_str_eq};
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
#[derive(Clone, Debug, Default)]
pub struct TestManager {
    description: String,
    fname: String,
    /// The affix file as a string
    afx_str: String,
    /// The dictionary file as a string
    dic_str: String,
    /// Personal dictionary file
    personal_str: Option<String>,
    /// These words/sentences will be checked with the check algorithm
    check_valid: Vec<String>,
    /// These words/sentences will be checked
    check_invalid: Vec<String>,
    wordlist: Option<Vec<String>>,
    wordlist_nosuggest: Option<Vec<String>>,
    wordlist_forbidden: Option<Vec<String>>,
    suggestions: Option<Vec<(String, Vec<String>)>>,
    stems: Option<Vec<(String, String)>>,
    morphs: Option<Vec<(String, Vec<MorphInfo>)>>,
}

impl TestManager {
    /// Load a `TestManager` from a string
    pub fn new_from_str(input: &str) -> Self {
        let mut ret = Self::default();
        // Remove our comments that start with "%%"
        let input_cleaned: String = input
            .lines()
            .filter(|line| !line.trim().starts_with("%%"))
            .fold(String::new(), |mut a, b| {
                a.reserve(b.len() + 1);
                a.push_str(b);
                a.push('\n');
                a
            });
        let mut content_iter = input_cleaned.trim().split("====").filter(|s| !s.is_empty());

        while let Some(s_title) = content_iter.next() {
            let sec_title = s_title.trim();
            let sec_content = content_iter.next().expect("Section title with no content");
            let lines_content: Vec<_> = sec_content
                .trim()
                .lines()
                .map(|line| line.to_owned())
                .collect();

            match sec_title {
                "afx_str" => ret.afx_str = sec_content.to_owned(),
                "dic_str" => ret.dic_str = sec_content.to_owned(),
                "personal_str" => ret.personal_str = Some(sec_content.to_owned()),
                "check_valid" => ret.check_valid = lines_content,
                "check_invalid" => ret.check_invalid = lines_content,
                "wordlist" => ret.wordlist = Some(lines_content),
                "wordlist_nosuggest" => ret.wordlist_nosuggest = Some(lines_content),
                "wordlist_forbidden" => ret.wordlist_forbidden = Some(lines_content),
                "suggestions" => {
                    // Suggestions look like "appl > apple | Apfel | app"
                    // Turn into ("appl", ["apple", "Apfel", "app"])
                    let mut tmp_ret: Vec<_> = Vec::new();
                    for suggestion in lines_content {
                        let tmp = suggestion.split_once('>').expect("Bad suggestion");
                        tmp_ret.push((
                            tmp.0.to_owned(),
                            tmp.1.split('|').map(|s| s.trim().to_owned()).collect(),
                        ))
                    }
                    ret.suggestions = Some(tmp_ret)
                }
                "end" => break,
                other => panic!("Bad section heading '{other}'. Collection:\n{ret:#?}\n"),
            };
        }

        ret
    }
    /// Load a `TestManager` from a given file name. Assumes the file will be
    /// located in `zspell/tests/files`.
    pub fn new_from_file(fname: &str) -> Self {
        let mut fpath = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        fpath.pop();
        fpath.push("tests");
        fpath.push("files");
        fpath.push(fname);

        let f_content = fs::read_to_string(&fpath)
            .unwrap_or_else(|_| panic!("error reading file '{}'", fpath.to_string_lossy()));

        let mut ret = Self::new_from_str(&f_content);
        ret.fname = fname.to_owned();
        ret
    }

    /// Build the dictionary based on given input
    pub fn build_dict(&self) -> Dictionary {
        let mut builder = DictBuilder::new()
            .config_str(&self.afx_str)
            .dict_str(&self.dic_str);

        if let Some(personal) = &self.personal_str {
            builder = builder.personal_str(personal.as_str());
        }

        builder.build().expect("error building dictionary")
    }

    /// Check everything in the file against our dictionary
    ///
    /// Panics with a message if there are any failures
    pub fn check_all(&self, dict: &Dictionary) {
        self.run_check_valid_invalid(dict);
        self.check_wordlists(dict)
    }

    /// Validate all expected checks are correct
    fn run_check_valid_invalid(&self, dict: &Dictionary) {
        for item in &self.check_valid {
            assert!(
                dict.check(item),
                "'{item}' failed check (expected true) in {}",
                self.fname
            );
        }

        if self.check_valid.is_empty() {
            eprintln!("Skipped check_valid testing")
        } else {
            eprintln!("Validated {} items as true", self.check_valid.len());
        }

        for item in &self.check_invalid {
            assert!(
                !dict.check(item),
                "'{item}' failed check (expected false) in {}",
                self.fname
            );
        }

        if self.check_invalid.is_empty() {
            eprintln!("Skipped check_invalid testing")
        } else {
            eprintln!("Validated {} items as false", self.check_invalid.len());
        }
    }

    /// Validate all our word lists are equal
    fn check_wordlists(&self, dict: &Dictionary) {
        let check_lists = [
            ("wordlist", &self.wordlist, dict.wordlist()),
            (
                "wordlist_nosuggest",
                &self.wordlist_nosuggest,
                dict.wordlist_nosuggest(),
            ),
            (
                "wordlist_forbidden",
                &self.wordlist_forbidden,
                dict.wordlist_forbidden(),
            ),
        ];

        for (name, expected_ref, actual_ref) in check_lists.into_iter() {
            let Some (tmp) = expected_ref else {
                eprintln!("skipped testing for {name}");
                continue;
            };
            let mut expected = tmp.clone();
            expected.sort_unstable();

            let mut actual: Vec<String> = actual_ref.inner().keys().cloned().collect();
            actual.sort_unstable();

            assert_eq!(
                expected, actual,
                "failed {name} checks for '{}'",
                self.fname
            );
            eprintln!("testing for {name} succeeded");
        }
    }

    /// Check all provided suggestions
    fn check_suggestions(&self, dict: &Dictionary) {
        let Some(ref suggestion) = self.suggestions else {
            eprintln!("skipped suggestion testing");
            return;
        };

        for (input, expected) in suggestion {
            let mut sug_dict = dict.suggest_word(input).unwrap_err();
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

    fn check_stems(&self, dict: &Dictionary) {
        let Some(ref stems) = self.stems else {
            eprintln!("skipped stem testing");
            return;
        };
        todo!()
    }

    fn check_morphs(&self, dict: &Dictionary) {
        let Some(ref morphs) = self.stems else {
            eprintln!("skipped stem testing");
            return;
        };
        todo!()
    }

    pub fn afx_str(&self) -> &str {
        self.afx_str.as_str()
    }
    pub fn dic_str(&self) -> &str {
        self.dic_str.as_str()
    }
    pub fn check_valid(&self) -> &Vec<String> {
        &self.check_valid
    }
    pub fn check_invalid(&self) -> &Vec<String> {
        &self.check_invalid
    }
    pub fn wordlist(&self) -> Option<&Vec<String>> {
        self.wordlist.as_ref()
    }
    pub fn wordlist_nosuggest(&self) -> Option<&Vec<String>> {
        self.wordlist_nosuggest.as_ref()
    }
    pub fn wordlist_forbidden(&self) -> Option<&Vec<String>> {
        self.wordlist_forbidden.as_ref()
    }
    pub fn suggestions(&self) -> Option<&Vec<(String, Vec<String>)>> {
        self.suggestions.as_ref()
    }
}
