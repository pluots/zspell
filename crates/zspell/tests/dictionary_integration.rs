use std::fs;
use zspell::Dictionary;

fn fixture_create_short_dict() -> Dictionary {
    // Test that we correctly compile the short wordlist
    let mut dic = Dictionary::new();

    let aff_content = fs::read_to_string("tests/files/short.aff").unwrap();
    let dic_content = fs::read_to_string("tests/files/short.dic").unwrap();

    dic.config.load_from_str(aff_content.as_str()).unwrap();
    dic.load_dict_from_str(dic_content.as_str());
    dic.compile().unwrap();
    dic
}

fn fixture_create_en_dict() -> Dictionary {
    // Test that we correctly compile the short wordlist
    let mut dic = Dictionary::new();

    let aff_content = fs::read_to_string("../../dictionaries/en.aff").unwrap();
    let dic_content = fs::read_to_string("../../dictionaries/en.dic").unwrap();

    dic.config.load_from_str(aff_content.as_str()).unwrap();
    dic.load_dict_from_str(dic_content.as_str());
    dic.compile().unwrap();
    dic
}

// Test compiling the dictionary from our short test file
#[test]
fn test_short_compile() {
    let dic = fixture_create_short_dict();

    let items = dic.wordlist_items();

    assert_eq!(
        items,
        vec![
            "rexxx",
            "rezzz",
            "rezzzen",
            "xxx",
            "yyication",
            "yyy",
            "zzz",
            "zzzen"
        ]
    );
}

// Test check functionality on our short file
#[test]
fn test_short_check() {
    let dic = fixture_create_short_dict();

    assert_eq!(dic.check("xxx"), true);
    assert_eq!(dic.check("yyication".to_string()), true);
    assert_eq!(dic.check("xxy".to_owned()), false);
    assert_eq!(dic.check(&"z".to_string()), false);
}

// Test check functionality on our full dict file
#[test]
fn test_full_en_check() {
    let dic = fixture_create_en_dict();

    // Start of file
    assert_eq!(dic.check("0"), true);
    assert_eq!(dic.check("0's"), true);
    // NOTE: compound rule checks not yet done

    // Middle of file
    assert_eq!(dic.check("coziness"), true);
    assert_eq!(dic.check("coziness's"), true);
    assert_eq!(dic.check("cradle"), true);
    assert_eq!(dic.check("cradled"), true);
    assert_eq!(dic.check("cradles"), true);
    assert_eq!(dic.check("cradle's"), true);
    assert_eq!(dic.check("cradling"), true);

    assert_eq!(dic.check("rate"), true);
    assert_eq!(dic.check("ratings"), true);
    assert_eq!(dic.check("rations"), true);
    assert_eq!(dic.check("rate's"), true);
    assert_eq!(dic.check("rating"), true);
    assert_eq!(dic.check("ration"), true);
    assert_eq!(dic.check("rated"), true);
    assert_eq!(dic.check("rater"), true);
    assert_eq!(dic.check("rates"), true);

    // End of file
    assert_eq!(dic.check("zymurgy"), true);
    assert_eq!(dic.check("zymurgy's"), true);

    // Not in file
    assert_eq!(dic.check("notinthisfile"), false);
}
