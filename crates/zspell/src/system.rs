use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;

use std::{
    env,
    path::{Component, Path, PathBuf},
};

use cfg_if::cfg_if;
use home::home_dir;
use regex::{escape, Regex};
use sys_locale::get_locale;

// use crate::errors::FileError;
use crate::errors::{DictError, SystemError};
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

const AFF_EXTENSIONS: [&str; 3] = ["aff", "afx", "affix"];
const DIC_EXTENSIONS: [&str; 3] = ["dic", "dict", "dictionary"];

/// Get the user's language from their system
///
/// Currently this only uses locale and defaults to en-US. Eventually we will
/// have it adapt to the user's config file in ~/.zspell.
#[inline]
pub fn get_preferred_lang() {
    get_locale().unwrap_or_else(|| String::from("en-US"));
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

/// Find a directory that uses wildcards
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
        // Looks tricky, but just filters out anything that isn't a directory
        .filter(|dir_entry| dir_entry.file_type().map_or(false, |v| v.is_dir()))
        .map(|dir_entry| dir_entry.path())
        .filter(|path_buf| re.is_match(&path_buf.to_string_lossy()));

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
pub fn expand_dir_wildcards(path_queue: &mut Vec<PathBuf>) -> HashSet<PathBuf> {
    // We will collect only the existing values here
    let mut ret = HashSet::new();

    // Work to empty our stack
    'loop_queue: while let Some(path) = path_queue.pop() {
        // This will hold the "working" parent path
        let mut cur_base = PathBuf::new();
        let mut comp_iter = path.components();
        let mut is_first_comp = true;

        // The first component will be
        'loop_comps: while let Some(comp) = comp_iter.next() {
            // If our parent doesn't exist or is not a dir, we're done here
            // Don't check this on the first loop when we have an empty buffer
            if !(is_first_comp || cur_base.exists() && cur_base.is_dir()) {
                break 'loop_comps;
            }

            match comp {
                Component::Normal(value) if value.to_string_lossy().contains('*') => {
                    // This block handles strings with wildcards

                    // Optimizer should get this reuse
                    let val_str = value.to_string_lossy();
                    let remaining_path: PathBuf = comp_iter.collect();

                    // Find directories that match the wildcard
                    let mut matching_dirs = find_matching_dirs(&cur_base, &val_str);

                    // Append the rest of our buffer to each of them
                    for matching_path_buf in &mut matching_dirs {
                        matching_path_buf.push(remaining_path.clone());
                    }

                    // Save the existing paths to our queue
                    path_queue.append(&mut matching_dirs);
                    continue 'loop_queue;
                }
                _ => {
                    // Anything else just gets added on with no fanfare
                    cur_base.push(comp);
                }
            }

            is_first_comp = false;
        }

        // Check if our base exists and is valid; if so, add it
        if cur_base.exists() && cur_base.is_dir() {
            ret.insert(cur_base);
        }
    }

    ret
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct PathInfo {
    buf: PathBuf,
    stem: String,
    extension: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DictPaths {
    pub dictionary: PathBuf,
    pub affix: PathBuf,
}

/// Given a path and a language, find any potential dictionary files
///
/// # Errors
///
/// If the directory cannot be accessed, return an error. This wraps a
/// [`std::io::ErrorKind`] error.
#[inline]
pub fn find_dicts_from_path<T: AsRef<str>>(
    path: &Path,
    lang: T,
) -> Result<Vec<DictPaths>, SystemError> {
    let lang_ref = lang.as_ref().to_lowercase();

    // Sometimes we get something that's e.g. en_US and sometimes en-US
    // Might have duplicates here, no problem since we only use `contains`
    let loc_bases = [
        lang_ref.clone(),
        lang_ref.replace('-', "_"),
        lang_ref.replace('_', "-"),
        lang_ref.split(['-', '_']).next().unwrap().to_owned(),
    ];

    let dir_iter = match fs::read_dir(path) {
        Ok(v) => v,
        Err(e) => {
            return Err(SystemError::IOError {
                name: path.to_string_lossy().to_string(),
                e: e.kind(),
            })
        }
    };

    // Collect all paths that are files and are named the location base
    let possible_paths: Vec<PathInfo> = dir_iter
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        // Keep the path, file name, and extension together
        .map(|path| PathInfo {
            stem: path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase(),
            extension: path
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase(),
            buf: path,
        })
        // Only get possible matching locale file names
        .filter(|pinfo| loc_bases.contains(&pinfo.stem))
        .collect();

    let existing_file_maps: Vec<DictPaths> = possible_paths
        .iter()
        // Start with a dictionary file
        .filter(|pinfo| DIC_EXTENSIONS.contains(&pinfo.extension.as_str()))
        // Existing same names with an affix file
        .flat_map(|pinfo| {
            possible_paths
                .iter()
                .filter(|pi| pi.stem == pinfo.stem)
                .filter(|pi| AFF_EXTENSIONS.contains(&pi.extension.as_str()))
                // Put matches into a struct
                .map(|pi| DictPaths {
                    dictionary: pinfo.buf.clone(),
                    affix: pi.buf.clone(),
                })
        })
        .collect();

    Ok(existing_file_maps)
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
                e: e.kind(),
            })
        }
    }

    match fs::read_to_string(&dict_file_path) {
        Ok(s) => dic.load_dict_from_str(s.as_str())?,
        Err(e) => {
            return Err(DictError::FileError {
                fname: dict_file_path,
                e: e.kind(),
            })
        }
    }

    // dic.config.load_from_str(aff_content.as_str()).unwrap();
    // dic.load_dict_from_str(dic_content.as_str());
    dic.compile().expect("Error in dictionary compilation");

    Ok(dic)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors;
    use std::{fs, io};
    use tempfile::tempdir;

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
        let dir = tempdir().unwrap();

        let mut paths = vec![
            dir.path().join("a").join("b").join("c-x-cxd"),
            dir.path().join("a").join("b").join("c-yz-cxd"),
            dir.path().join("a").join("b").join("c-.abc-cxd"),
        ];
        paths.sort();

        for path in &paths {
            fs::create_dir_all(path).unwrap();
        }

        let mut ret = find_matching_dirs(&dir.path().join("a").join("b"), "c-*-c?d");
        ret.sort();

        assert_eq!(paths, ret);
    }

    #[test]
    fn test_expand_dir_wildcards() {
        let dir = tempdir().unwrap();

        let paths = vec![
            dir.path().join("aaa").join("bbb-x").join("ccc"),
            dir.path().join("aaa").join("bbb-y").join("ccc"),
            dir.path().join("ddd"),
        ];

        for path in &paths {
            fs::create_dir_all(path).unwrap();
        }

        let mut input = vec![
            dir.path().join("aaa").join("bbb*").join("ccc"),
            dir.path().join("ddd"),
        ];

        let mut expanded = Vec::from_iter(expand_dir_wildcards(&mut input));
        expanded.sort_unstable();

        assert_eq!(paths, expanded);
    }

    #[test]
    fn test_find_dict_from_path() {
        let dir = tempdir().unwrap();

        let fnames = vec![
            dir.path().join("test_found.dic"),
            dir.path().join("test_found.aff"),
            dir.path().join("test_found.afx"),
            dir.path().join("test.dict"),
            dir.path().join("test.affix"),
            dir.path().join("notfound.dic"),
            dir.path().join("notfound.aff"),
            dir.path().join("test"),
        ];

        let mut expected = vec![
            DictPaths {
                dictionary: fnames[0].clone(),
                affix: fnames[1].clone(),
            },
            DictPaths {
                dictionary: fnames[0].clone(),
                affix: fnames[2].clone(),
            },
            DictPaths {
                dictionary: fnames[3].clone(),
                affix: fnames[4].clone(),
            },
        ];
        expected.sort();

        for fname in fnames {
            fs::File::create(fname).unwrap();
        }
        fs::read_dir(dir.path()).unwrap();

        let mut res = find_dicts_from_path(dir.path(), "test_found").unwrap();
        res.sort();

        assert_eq!(res, expected);
    }

    #[test]
    fn test_find_dict_from_path_err() {
        let fakepath = tempdir().unwrap().path().join("fake");
        let res = find_dicts_from_path(&fakepath, "test_found");

        assert_eq!(
            Err(errors::SystemError::IOError {
                name: fakepath.to_string_lossy().to_string(),
                e: io::ErrorKind::NotFound
            }),
            res
        )
    }
}
