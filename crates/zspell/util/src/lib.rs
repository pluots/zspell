/// Utilities intended to help with test collection
use std::fs;

use zspell::Dictionary;

/// A collection from a .test file that we can easily validate
#[derive(Debug)]
pub struct TestCollection {
    /// The affix file as a string
    pub afx_str: String,
    /// The dictionary file as a string
    pub dic_str: String,
    /// These words will be checked with the check algorithm
    pub check_valid: Option<Vec<String>>,
    pub check_invalid: Option<Vec<String>>,
    pub wordlist: Option<Vec<String>>,
    pub wordlist_nosuggest: Option<Vec<String>>,
    pub wordlist_forbidden: Option<Vec<String>>,
    pub suggestions: Option<Vec<(String, Vec<String>)>>,
}

impl TestCollection {
    pub fn load(fname: &str) -> Self {
        let mut ret = Self {
            afx_str: String::new(),
            dic_str: String::new(),
            check_valid: None,
            check_invalid: None,
            wordlist: None,
            wordlist_nosuggest: None,
            wordlist_forbidden: None,
            suggestions: None,
        };

        let fname_new = format!("tests/files/{fname}");

        let f_content = fs::read_to_string(fname_new.clone())
            .unwrap_or_else(|_| panic!("error reading file '{fname_new}'"));

        let mut content_iter = f_content.trim().split("====").filter(|&x| !x.is_empty());

        while let Some(s_title) = content_iter.next() {
            let sec_title = s_title.trim();
            let sec_content = match content_iter.next() {
                Some(s) => s,
                None => panic!("Section title with no content"),
            };

            match sec_title {
                "afx_str" => ret.afx_str = sec_content.to_owned(),
                "dic_str" => ret.dic_str = sec_content.to_owned(),
                "check_valid" => {
                    ret.check_valid = Some(
                        sec_content
                            .trim()
                            .split('\n')
                            .map(|s| s.to_owned())
                            .collect::<Vec<_>>(),
                    )
                }
                "check_invalid" => {
                    ret.check_invalid = Some(
                        sec_content
                            .trim()
                            .split('\n')
                            .map(|s| s.to_owned())
                            .collect::<Vec<_>>(),
                    )
                }
                "wordlist" => {
                    ret.wordlist = Some(
                        sec_content
                            .trim()
                            .split('\n')
                            .map(|s| s.to_owned())
                            .collect::<Vec<_>>(),
                    )
                }
                "wordlist_nosuggest" => {
                    ret.wordlist_nosuggest = Some(
                        sec_content
                            .trim()
                            .split('\n')
                            .map(|s| s.to_owned())
                            .collect::<Vec<_>>(),
                    )
                }
                "wordlist_forbidden" => {
                    ret.wordlist_forbidden = Some(
                        sec_content
                            .trim()
                            .split('\n')
                            .map(|s| s.to_owned())
                            .collect::<Vec<_>>(),
                    )
                }
                "suggestions" => {
                    // Suggestions look like "appl > apple | Apfel | app"
                    // Turn into ("appl", ["apple", "Apfel", "app"])
                    let mut tmp_ret: Vec<_> = Vec::new();

                    let sug_split = sec_content.split_terminator('\n');
                    for suggestion in sug_split {
                        let tmp = suggestion.split_once('>').expect("Bad suggestion");
                        tmp_ret.push((
                            tmp.0.to_owned(),
                            tmp.1.split('|').map(|s| s.trim().to_owned()).collect(),
                        ))
                    }
                    ret.suggestions = Some(tmp_ret)
                }
                "end" => break,
                other => panic!("Bad section heading '{}'. Collection:\n{:#?}\n", other, ret),
            };
        }

        ret
    }

    /// Validate all expected checks are correct
    fn run_check_valid_invalid(&self, dic: &Dictionary) {
        match &self.check_valid {
            Some(v) => {
                for item in v {
                    let res = dic.check(item).expect("Dictionary error");
                    assert!(res, "{} failed check (expected true)", item);
                }
                println!("Validated {} items as true", v.len());
            }
            None => println!("Skipped check_valid testing"),
        };
        match &self.check_invalid {
            Some(v) => {
                for item in v {
                    let res = dic.check(item).expect("Dictionary error");
                    assert!(!res, "{} failed check (expected false)", item)
                }
                println!("Validated {} items as false", v.len());
            }
            None => println!("Skipped check_invalid testing"),
        };
    }

    /// Validate all our word lists are equal
    fn check_wordlists(&self, dic: &Dictionary) {
        let mut wordlist_v: Vec<_> = dic
            .iter_wordlist_items()
            .expect("Error getting wordlist")
            .map(|s| s.to_owned())
            .collect();
        wordlist_v.sort_unstable();

        let mut wordlist_ns_v: Vec<_> = dic
            .iter_wordlist_items()
            .expect("Error getting nosuggest wordlist")
            .map(|s| s.to_owned())
            .collect();
        wordlist_ns_v.sort_unstable();

        let mut wordlist_f_v: Vec<_> = dic
            .iter_wordlist_items()
            .expect("Error getting forbidden wordlist")
            .map(|s| s.to_owned())
            .collect();
        wordlist_f_v.sort_unstable();

        match &self.wordlist {
            Some(v) => {
                assert_eq!(*v, wordlist_v);
                println!("Validated wordlist against {} items", v.len());
            }
            None => println!("Skipped wordlist testing"),
        };

        match &self.wordlist_nosuggest {
            Some(v) => {
                assert_eq!(*v, wordlist_ns_v);
                println!("Validated wordlist_nosuggest against {} items", v.len());
            }
            None => println!("Skipped wordlist_nosuggest testing"),
        };

        match &self.wordlist_forbidden {
            Some(v) => {
                assert_eq!(*v, wordlist_f_v);
                println!("Validated wordlist_forbidden against {} items", v.len());
            }
            None => println!("Skipped wordlist_forbidden testing"),
        };
    }

    fn check_suggestions(&self, _dic: &Dictionary) {
        println!("Skpped suggestion testing");
    }

    pub fn validate(&self) {
        let mut dic = Dictionary::new();

        // Validate we can load the dictionary
        dic.config
            .load_from_str(self.afx_str.as_str())
            .expect("config loading failure");
        dic.load_dict_from_str(self.dic_str.as_str())
            .expect("loading failure");
        dic.compile().expect("compiling failure");

        // Now check everything we can
        self.run_check_valid_invalid(&dic);
        self.check_wordlists(&dic);
        self.check_suggestions(&dic);
    }
}
