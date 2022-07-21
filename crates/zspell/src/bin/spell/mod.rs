//! Helpers for CLI spelling features

use std::fs;
use std::io::{self, BufRead};
use zspell::Dictionary;

pub fn create_dict_from_path(basepath: &str) -> Dictionary {
    let mut dict_file_path = basepath.to_owned();
    let mut affix_file_path = basepath.to_owned();

    dict_file_path.push_str(".dic");
    affix_file_path.push_str(".aff");

    let aff_content = fs::read_to_string(&affix_file_path)
        .unwrap_or_else(|_| panic!("Unable to find .aff file. Does {} exist?", affix_file_path));
    let dic_content = fs::read_to_string(&dict_file_path)
        .unwrap_or_else(|_| panic!("Unable to find .dic file. Does {} exist?", dict_file_path));

    let mut dic = Dictionary::new();
    dic.config.load_from_str(aff_content.as_str()).unwrap();
    dic.load_dict_from_str(dic_content.as_str());
    dic.compile().expect("Error in dictionary compilation");

    dic
}

#[inline]
pub fn spellcheck_list_cli_runner(dic: &Dictionary) {
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let unwrapped = line.unwrap();

        for word in dic.check_return_list(unwrapped) {
            println!("{}", &word)
        }
    }
}
