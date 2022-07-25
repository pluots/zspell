use std::fs;
use zspell::Dictionary;

/// Run integration tests on a file located in tests/files
fn create_dic_from_file(fname: &str) -> Dictionary {
    let aff_name = format!("tests/files/{}.aff", fname);
    let dic_name = format!("tests/files/{}.dic", fname);

    let aff_content = fs::read_to_string(aff_name.clone())
        .expect(format!("error reading file {}", aff_name).as_str());
    let dic_content = fs::read_to_string(dic_name.clone())
        .expect(format!("error reading file {}", dic_name).as_str());

    let mut dic = Dictionary::new();
    dic.config
        .load_from_str(aff_content.as_str())
        .expect("config loading failure");
    dic.load_dict_from_str(dic_content.as_str())
        .expect("loading failure");
    dic.compile().expect("compiling failure");

    dic
}

/// Validate a dictionary's wordlist is correct
fn test_dic_wordlist(dic: &Dictionary, fname: &str) {
    let out_name = format!("tests/files/{}.words", fname);

    let out_content = fs::read_to_string(out_name.clone())
        .expect(format!("error reading file {}", out_name).as_str());
    let mut correct: Vec<_> = out_content.lines().collect();
    correct.sort_unstable();
    let mut result: Vec<_> = dic
        .iter_wordlist_items()
        .expect("Error getting wordlist")
        .collect();
    result.sort_unstable();

    assert_eq!(result, correct);
}

/// Test compiling the dictionary from our short test file
#[test]
fn test_short_compile() {
    let dic = create_dic_from_file("1_pfxsfx");
    test_dic_wordlist(&dic, "1_pfxsfx");
}

/// Test check functionality on our short file
#[test]
fn test_short_check() {
    let dic = create_dic_from_file("1_pfxsfx");

    // Test all ownership methods
    assert_eq!(dic.check("xxx"), Ok(true));
    assert_eq!(dic.check("yybb".to_string()), Ok(true));
    assert_eq!(dic.check("aazzzcc".to_owned()), Ok(true));
    assert_eq!(dic.check(&"zzz".to_string()), Ok(true));

    assert_eq!(dic.check("not contained"), Ok(false));
}
