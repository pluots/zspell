use home::home_dir;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::os::unix::prelude::OsStrExt;
use std::{
    env,
    path::{Component, PathBuf},
};

use crate::errors::UsageError;
// use crate::errors::FileError;
use crate::Dictionary;

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, PartialEq, Eq)]
enum Plat {
    Windows,
    Posix,
}

/// All of these paths will be added to the relevant DIR_NAME lists
const ENV_VAR_NAMES: [&str; 5] = [
    "DICPATH",
    "XDG_DATA_HOME",
    "XDG_DATA_DIRS",
    "XDG_CONFIG_DIRS",
    "HOME",
];
const CHILD_DIR_NAMES: [&str; 8] = [
    ".zspell",
    "zspell",
    ".spell",
    ".myspell",
    "myspell",
    ".hunspell",
    "hunspell",
    "dicts",
];

const WIN_DIR_NAMES: [&str; 2] = ["~", r"C:\Program files\OpenOffice.org*\share\dict\ooo"];
const POSIX_DIR_NAMES: [&str; 15] = [
    "~",
    "~/.local/share",
    "/usr/share",
    "/usr/local/share",
    "/usr/share/myspell/dicts",
    "/Library/Spelling",
    "~/Library/Spelling",
    "/Library/Application Support",
    "~/Library/Application Support",
    "~/.openoffice.org/*/user/wordbook",
    "~/.openoffice.org*/user/wordbook",
    "/opt/openoffice.org/basis*/share/dict/ooo",
    "/usr/lib/openoffice.org/basis*/share/dict/ooo",
    "/opt/openoffice.org*/share/dict/ooo",
    "/usr/lib/openoffice.org*/share/dict/ooo",
];

fn get_plat() -> Plat {
    match env::consts::OS {
        "windows" => Plat::Windows,
        _ => Plat::Posix,
    }
}

/// Split $PATH-like variables by the apropriate separator
/// e.g. $PATH=/abc/def:/ghi/jkl:/mno -> [/abc/def, /ghi/jkl, /mno]
fn split_os_path_string(oss: OsString) -> Vec<PathBuf> {
    let mut ret = Vec::new();

    // Get the separator for $PATH-like values
    let path_sep: u8 = match get_plat() {
        Plat::Windows => 0x3b, // ";" on Windows
        _ => 0x3a,             // ":" on Posix
    };

    let byte_arr = oss.as_bytes();

    byte_arr
        .split(|x| *x == path_sep)
        .for_each(|x| ret.push(PathBuf::from(OsStr::from_bytes(x))));

    ret
}

/// Create a list of possible locations to find dictionary files.
/// Expands home; does not expand windcards
pub fn create_raw_paths() -> Vec<PathBuf> {
    let mut raw_vec_base: Vec<PathBuf> = Vec::new();

    // Add all environment variable paths to our raw list
    for env in ENV_VAR_NAMES {
        match env::var_os(env) {
            Some(v) => raw_vec_base.append(&mut split_os_path_string(v)),
            None => (),
        }
    }

    // Add all our raw path names
    match get_plat() {
        Plat::Windows => WIN_DIR_NAMES
            .iter()
            .for_each(|s| raw_vec_base.push(PathBuf::from(s))),
        _ => POSIX_DIR_NAMES
            .iter()
            .for_each(|s| raw_vec_base.push(PathBuf::from(s))),
    };

    let mut raw_vec = Vec::new();

    // Go through each pathbuf and add it, plus all possible suffixes, to the
    // search path
    for pathbuf in raw_vec_base {
        raw_vec.push(pathbuf.clone());

        for child_dir in CHILD_DIR_NAMES {
            let mut cloned = pathbuf.clone();
            cloned.push(child_dir);
            raw_vec.push(cloned);
        }
    }

    // Values to expand to the home path
    let home_options = [OsStr::new("$HOME"), OsStr::new("~")];
    let home_path = home_dir();

    let mut new_vec = Vec::new();

    // Expand "$HOME" and "~"
    for pathbuf in raw_vec {
        let mut working_new = PathBuf::new();

        // Iterate through each path section, e.g. "/a/b/c" -> [a, b, c]
        for comp in pathbuf.as_path().components() {
            match comp {
                Component::Normal(value) => {
                    if home_path.is_none() || !home_options.contains(&value) {
                        working_new.push(Component::Normal(value))
                    } else {
                        working_new.push(home_path.clone().unwrap())
                    }
                }
                // Anything that's not HOME, just add it
                any => working_new.push(any),
            }
        }

        new_vec.push(working_new);
    }

    new_vec
}

// Take in a path and load the dictionary
pub fn create_dict_from_path(basepath: &str) -> Result<Dictionary, UsageError> {
    let mut dic = Dictionary::new();

    let mut dict_file_path = basepath.to_owned();
    let mut affix_file_path = basepath.to_owned();

    dict_file_path.push_str(".dic");
    affix_file_path.push_str(".aff");

    // let aff_content = fs::read_to_string(&affix_file_path)?;
    // let dic_content = fs::read_to_string(&dict_file_path)?;

    // let e = FileError::Other(dic_content.unwrap_err().kind());

    match fs::read_to_string(&affix_file_path) {
        Ok(s) => dic.config.load_from_str(s.as_str()).unwrap(),
        Err(e) => {
            return Err(UsageError::FileError {
                fname: affix_file_path,
                orig_e: e,
            })
        }
    }

    match fs::read_to_string(&dict_file_path) {
        Ok(s) => dic.load_dict_from_str(s.as_str()),
        Err(e) => {
            return Err(UsageError::FileError {
                fname: dict_file_path,
                orig_e: e,
            })
        }
    }

    // dic.config.load_from_str(aff_content.as_str()).unwrap();
    // dic.load_dict_from_str(dic_content.as_str());
    dic.compile().expect("Error in dictionary compilation");

    Ok(dic)
}

// Need function to expand wildcard paths. Will need to look through the parent
// directory and see if anything is a RE match

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_os_path() {
        let s_ix = OsString::from("/aaa/bbb:/ccc:/ddd");
        let s_win = OsString::from(r"c:\aaa\bbb;d:\ccc;e:\ddd");

        if get_plat() == Plat::Posix {
            let v_split = vec![
                PathBuf::from("/aaa/bbb"),
                PathBuf::from("/ccc"),
                PathBuf::from("/ddd"),
            ];
            assert_eq!(split_os_path_string(s_ix), v_split);
        } else {
            let v_split = vec![
                PathBuf::from(r"c:\aaa\bbb"),
                PathBuf::from(r"d:\ccc"),
                PathBuf::from(r"e:\ddd"),
            ];
            assert_eq!(split_os_path_string(s_win), v_split);
        }
    }

    #[test]
    fn test_raw_paths() {
        let paths = create_raw_paths();
        assert!(paths.contains(&PathBuf::from("/usr/share")));
        assert!(paths.contains(&PathBuf::from("/usr/share/zspell")));
        assert!(paths.contains(&PathBuf::from("/usr/share/myspell")));
        assert!(paths.contains(&PathBuf::from("/usr/share/hunspell")));
        assert!(paths.contains(&PathBuf::from("/Library/Spelling/hunspell")));
        assert!(paths.contains(&PathBuf::from("/Library/Spelling/hunspell")));
    }
}
