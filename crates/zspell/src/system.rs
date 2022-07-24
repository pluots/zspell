use cfg_if::cfg_if;
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

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
enum Plat {
    Windows,
    Posix,
}

cfg_if! {
    if #[cfg(windows)] {
        // windows config
        #[allow(dead_code)]
        const PLAT: Plat = Plat::Windows;
        // The separator for $PATH-like values; ";" on windows
        const ENV_PATH_SEP: u8 = 0x3b;
        const BASE_DIR_NAMES: [&str; 2] = [
            "~",
            r"C:\Program files\OpenOffice.org*\share\dict\ooo"
        ];

    } else {
        // unix or WASM
        #[allow(dead_code)]
        const PLAT: Plat = Plat::Posix;
        // The separator for $PATH-like values; ":" on posix
        const ENV_PATH_SEP: u8 = 0x3a;

        const BASE_DIR_NAMES: [&str; 15] = [
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
    }
}

/// All of these paths will be added to the relevant `DIR_NAME` lists
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

/// Split $PATH-like variables by the apropriate separator, e.g.
/// $PATH=/abc/def:/ghi/jkl:/mno -> [/abc/def, /ghi/jkl, /mno]
///
/// oss is an `OsString` (bytestring)
fn split_os_path_string(oss: &OsString) -> Vec<PathBuf> {
    oss.as_bytes()
        // Split by the path separator
        .split(|x| *x == ENV_PATH_SEP)
        // Re-load the bytes into an osstring
        .map(OsStr::from_bytes)
        // Create the pathbuf that we want
        .map(PathBuf::from)
        // And create the vec
        .collect()
}

/// Create a list of possible locations to find dictionary files. Expands home;
/// does not expand windcards
#[inline]
pub fn create_raw_paths() -> Vec<PathBuf> {
    // Please excuse the iterators but Rust is cool

    // Loop through all our environment variables
    let env_paths = ENV_VAR_NAMES
        .iter()
        // Get values of only the vars that exist
        .filter_map(env::var_os)
        // Split these into vectors of PathBufs, and flatten
        .flat_map(|val| split_os_path_string(&val));

    // Create a PathBuf for each of our non-env paths
    let base_paths = BASE_DIR_NAMES.iter().map(PathBuf::from);

    // Put our env paths and base paths together in a vector
    let mut search_paths_raw: Vec<PathBuf> = env_paths.chain(base_paths).collect();

    // Go through each pathbuf and add all possible suffixes to the search path.
    // Need to clone so we don't append while we iter.
    for pathbuf in search_paths_raw.clone() {
        for child_dir in CHILD_DIR_NAMES {
            let mut cloned = pathbuf.clone();
            cloned.push(child_dir);
            search_paths_raw.push(cloned);
        }
    }

    // Values to expand to the home path
    let home_options = [OsStr::new("$HOME"), OsStr::new("~")];
    let home_path = home_dir();

    let mut search_paths = Vec::new();

    // Expand "$HOME" and "~"
    for pathbuf in search_paths_raw {
        let mut working_new = PathBuf::new();

        // Iterate through each path section, e.g. "/a/b/c" -> [a, b, c]
        for comp in pathbuf.as_path().components() {
            match comp {
                Component::Normal(value) => {
                    if home_path.is_none() || !home_options.contains(&value) {
                        working_new.push(Component::Normal(value));
                    } else {
                        working_new.push(home_path.clone().unwrap());
                    }
                }
                // Anything that's not HOME, just add it
                other => working_new.push(other),
            }
        }

        search_paths.push(working_new);
    }

    search_paths
}

/// Use real directory
#[inline]
pub fn expand_dir_wildcards(paths: &Vec<PathBuf>) -> Vec<PathBuf> {
    let ret = Vec::new();
    // Loop through all paths; we will collect those that exist
    for path in paths {
        // This collects part of the path as we validate they exist
        let root = PathBuf::new();
        let testing_parents: Vec<PathBuf> = Vec::new();

        for comp in path.components() {
            match comp {
                Component::Normal(value) => {}
                other => {}
            }

            // if comp ==  {

            // }

            // .into_os_string()
            // .into_string().unwrap_or("").contains("*")
        }
    }
    // LOOP
    // Pop item
    //
    //
    //
    //
    //
    //
    //
    //

    ret
}

/// Take in a path and load the dictionary
///
/// # Errors
///
/// Error when can't find dictionary
#[inline]
pub fn create_dict_from_path(basepath: &str) -> Result<Dictionary, UsageError> {
    let mut dic = Dictionary::new();

    let mut dict_file_path = basepath.to_owned();
    let mut affix_file_path = basepath.to_owned();

    dict_file_path.push_str(".dic");
    affix_file_path.push_str(".aff");

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

        if PLAT == Plat::Posix {
            let v_split = vec![
                PathBuf::from("/aaa/bbb"),
                PathBuf::from("/ccc"),
                PathBuf::from("/ddd"),
            ];
            assert_eq!(split_os_path_string(&s_ix), v_split);
        } else {
            let v_split = vec![
                PathBuf::from(r"c:\aaa\bbb"),
                PathBuf::from(r"d:\ccc"),
                PathBuf::from(r"e:\ddd"),
            ];
            assert_eq!(split_os_path_string(&s_win), v_split);
        }
    }

    #[test]
    fn test_raw_paths() {
        // Just spot check what we have here
        let paths = create_raw_paths();

        match PLAT {
            Plat::Posix => {
                assert!(paths.contains(&PathBuf::from("/usr/share")));
                assert!(paths.contains(&PathBuf::from("/usr/share/zspell")));
                assert!(paths.contains(&PathBuf::from("/usr/share/myspell")));
                assert!(paths.contains(&PathBuf::from("/usr/share/hunspell")));
                assert!(paths.contains(&PathBuf::from("/Library/Spelling/hunspell")));
                assert!(paths.contains(&PathBuf::from("/Library/Spelling/hunspell")));
            }
            Plat::Windows => {
                assert!(paths.contains(&PathBuf::from(
                    r"C:\Program files\OpenOffice.org*\share\dict\ooo"
                )));
                assert!(paths.contains(&PathBuf::from(
                    r"C:\Program files\OpenOffice.org*\share\dict\ooo\hunspell"
                )));
            }
        }
    }
}
