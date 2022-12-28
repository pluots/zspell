//! Tests for the `system` module

// use std::{fs, io};

// use tempfile::tempdir;

// use super::*;
// use crate::errors;

// #[test]
// #[cfg(windows)]
// fn test_raw_paths() {
//     // Just spot check what we have here
//     let paths = create_raw_paths();

//     assert!(paths.contains(&PathBuf::from(
//         r"C:\Program files\OpenOffice.org*\share\dict\ooo"
//     )));
//     assert!(paths.contains(&PathBuf::from(
//         r"C:\Program files\OpenOffice.org*\share\dict\ooo\hunspell"
//     )));
// }

// #[test]
// #[cfg(not(windows))]
// fn test_raw_paths() {
//     // Just spot check what we have here
//     let paths = create_raw_paths();

//     assert!(paths.contains(&PathBuf::from("/usr/share")));
//     assert!(paths.contains(&PathBuf::from("/usr/share/zspell")));
//     assert!(paths.contains(&PathBuf::from("/usr/share/myspell")));
//     assert!(paths.contains(&PathBuf::from("/usr/share/hunspell")));
//     assert!(paths.contains(&PathBuf::from("/Library/Spelling/hunspell")));
//     assert!(paths.contains(&PathBuf::from("/Library/Spelling/hunspell")));
// }

// #[test]
// fn test_matching_dirs() {
//     // Create a temporary directory with contents
//     // Ensure the function locates them using wildcards
//     let dir = tempdir().unwrap();

//     let mut paths = vec![
//         dir.path().join("a").join("b").join("c-x-cxd"),
//         dir.path().join("a").join("b").join("c-yz-cxd"),
//         dir.path().join("a").join("b").join("c-.abc-cxd"),
//     ];
//     paths.sort();

//     for path in &paths {
//         fs::create_dir_all(path).unwrap();
//     }

//     let mut ret = find_matching_dirs(&dir.path().join("a").join("b"), "c-*-c?d");
//     ret.sort();

//     assert_eq!(paths, ret);
// }

// #[test]
// fn test_expand_dir_wildcards() {
//     let dir = tempdir().unwrap();

//     let paths = vec![
//         dir.path().join("aaa").join("bbb-x").join("ccc"),
//         dir.path().join("aaa").join("bbb-y").join("ccc"),
//         dir.path().join("ddd"),
//     ];

//     for path in &paths {
//         fs::create_dir_all(path).unwrap();
//     }

//     let mut input = vec![
//         dir.path().join("aaa").join("bbb*").join("ccc"),
//         dir.path().join("ddd"),
//     ];

//     let mut expanded = Vec::from_iter(expand_dir_wildcards(&mut input));
//     expanded.sort_unstable();

//     assert_eq!(paths, expanded);
// }

// #[test]
// fn test_find_dict_from_path() {
//     let dir = tempdir().unwrap();

//     let fnames = vec![
//         dir.path().join("test_found.dic"),
//         dir.path().join("test_found.aff"),
//         dir.path().join("test_found.afx"),
//         dir.path().join("test.dict"),
//         dir.path().join("test.affix"),
//         dir.path().join("notfound.dic"),
//         dir.path().join("notfound.aff"),
//         dir.path().join("test"),
//     ];

//     let mut expected = vec![
//         DictPaths {
//             dictionary: fnames[0].clone(),
//             affix: fnames[1].clone(),
//         },
//         DictPaths {
//             dictionary: fnames[0].clone(),
//             affix: fnames[2].clone(),
//         },
//         DictPaths {
//             dictionary: fnames[3].clone(),
//             affix: fnames[4].clone(),
//         },
//     ];
//     expected.sort();

//     for fname in fnames {
//         fs::File::create(fname).unwrap();
//     }
//     fs::read_dir(dir.path()).unwrap();

//     let mut res = find_dicts_from_path(dir.path(), "test_found").unwrap();
//     res.sort();

//     assert_eq!(res, expected);
// }

// #[test]
// fn test_find_dict_from_path_err() {
//     let fakepath = tempdir().unwrap().path().join("fake");
//     let res = find_dicts_from_path(&fakepath, "test_found");

//     assert_eq!(
//         Err(errors::SystemError::IOError {
//             name: fakepath.to_string_lossy().to_string(),
//             e: io::ErrorKind::NotFound
//         }),
//         res
//     );
// }
