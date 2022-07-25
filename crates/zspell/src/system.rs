use cfg_if::cfg_if;
use hashbrown::HashSet;
use home::home_dir;
use regex::{escape, Regex};
use std::ffi::OsStr;
use std::fs;
use std::{
    env,
    path::{Component, Path, PathBuf},
};

use crate::errors::DictError;
// use crate::errors::FileError;
use crate::{unwrap_or_ret, Dictionary};

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
        const BASE_DIR_NAMES: [&str; 2] = [
            "~",
            r"C:\Program files\OpenOffice.org*\share\dict\ooo"
        ];

    } else {
        // unix or WASM
        #[allow(dead_code)]
        const PLAT: Plat = Plat::Posix;
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
        // Split these by PATH separator (':' on Unix, ';' on Windows) and flatten
        .flat_map(|x| env::split_paths(&x).collect::<Vec<PathBuf>>());

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

#[inline]
pub fn find_matching_dirs(parent: &Path, pattern: &str) -> Vec<PathBuf> {
    let mut ret = Vec::new();

    // Escape anything unexpected, then convert wildcard -> regex
    let pattern_rep = escape(pattern).replace(r"\*", ".*").replace(r"\?", ".");

    let dir_items = unwrap_or_ret!(parent.read_dir(), ret);
    let re = unwrap_or_ret!(Regex::new(&pattern_rep), ret);

    // Start with our may-or-may-not-exist dir items
    let matched_iter = dir_items
        // Get Ok() values
        .filter_map(Result::ok)
        // Looks tricky, but just returns the path if our item is a directory
        .filter_map(|x| match x.file_type() {
            Ok(ft) => {
                if ft.is_dir() {
                    Some(x.path())
                } else {
                    None
                }
            }
            Err(_) => None,
        })
        // Get items that match
        .filter(|x| re.is_match(&x.to_string_lossy()));

    // Create a new item for the parent paths if exists
    for item in matched_iter {
        let mut parent_c = parent.to_path_buf();
        parent_c.push(item);
        ret.push(parent_c);
    }

    ret
}

/// Expand wildcards (*) in directory paths, and return only directories that
/// exist
///
/// This takes a mutable vector that will be drained (used as a stack).
#[inline]
pub fn expand_dir_wildcards(paths: &mut Vec<PathBuf>) -> HashSet<PathBuf> {
    // We will collect only the existing values here
    let mut ret = HashSet::new();

    // Work to empty our stack
    while let Some(top) = paths.pop() {
        // This will hold the "working" parent path
        let mut cur_base = PathBuf::new();
        let mut is_new = true;

        for comp in top.components() {
            // If our parent doesn't exist or is not a dir, we're done here
            if !is_new && (!cur_base.exists() || !cur_base.is_dir()) {
                break;
            }

            if let Component::Normal(value) = comp {
                // Enter here if this part of the
                let val_str = value.to_string_lossy();
                if val_str.contains('*') {
                    find_matching_dirs(&cur_base, &val_str);
                } else {
                    // Just add anything else
                    cur_base.push(comp);
                }
            } else {
                // Anything else just gets added on with no fanfare
                cur_base.push(comp);
            }

            is_new = false;
        }

        if cur_base.exists() && cur_base.is_dir() {
            ret.insert(cur_base);
        }
    }

    ret
}

/// Take in a path and load the dictionary
///
/// # Errors
///
/// Error when can't find dictionary
#[inline]
pub fn create_dict_from_path(basepath: &str) -> Result<Dictionary, DictError> {
    let mut dic = Dictionary::new();

    let mut dict_file_path = basepath.to_owned();
    let mut affix_file_path = basepath.to_owned();

    dict_file_path.push_str(".dic");
    affix_file_path.push_str(".aff");

    match fs::read_to_string(&affix_file_path) {
        Ok(s) => dic.config.load_from_str(s.as_str()).unwrap(),
        Err(e) => {
            return Err(DictError::FileError {
                fname: affix_file_path,
                orig_e: e.kind(),
            })
        }
    }

    match fs::read_to_string(&dict_file_path) {
        Ok(s) => dic.load_dict_from_str(s.as_str())?,
        Err(e) => {
            return Err(DictError::FileError {
                fname: dict_file_path,
                orig_e: e.kind(),
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

    #[test]
    fn test_matching_dirs() {
        // Create a temporary directory with contents
        // Ensure the function locates them using wildcards
    }
}
