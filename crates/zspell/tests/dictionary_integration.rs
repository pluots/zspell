// use std::fs;

use util::TestManager;
use zspell::Dictionary;

#[test]
fn test_pfx_sfx() {
    // TestManager::n
}

// /// Run integration tests on a file located in tests/files
// fn create_dic_from_file(fname: &str) -> Dictionary {
//     let aff_name = format!("tests/files/{fname}.aff");
//     let dic_name = format!("tests/files/{fname}.dic");

//     let aff_content =
//         fs::read_to_string(&aff_name).unwrap_or_else(|_| panic!("error reading file {aff_name}"));
//     let dic_content =
//         fs::read_to_string(dic_name).unwrap_or_else(|_| panic!("error reading file {aff_name}"));

//     let mut dic = Dictionary::new();
//     dic.config
//         .load_from_str(aff_content.as_str())
//         .expect("config loading failure");
//     dic.load_dict_from_str(dic_content.as_str())
//         .expect("loading failure");
//     dic.compile().expect("compiling failure");

//     dic
// }

// /// Test check functionality on a real file
// #[test]
// #[allow(clippy::unnecessary_to_owned)]
// fn test_short_check() {
//     let dic = create_dic_from_file("w1_eng_short");

//     // Test all ownership methods
//     assert_eq!(dic.check("bananas"), Ok(true));
//     assert_eq!(dic.check("pines".to_string()), Ok(true));
//     assert_eq!(dic.check("not contained"), Ok(false));
// }

// #[test]
// fn test_prefixes() {
//     let coll = TestCollection::load("1_pfxsfx.test");
//     coll.validate();
// }
