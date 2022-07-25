use std::fs;
use zspell::Config;

#[test]
fn affix_create_words() {
    let mut afx = Config::new();

    let content = fs::read_to_string("tests/files/1_pfxsfx.aff").unwrap();

    afx.load_from_str(content.as_str()).unwrap();

    assert_eq!(
        afx.create_affixed_words("xxx", "A"),
        vec!["xxx".to_string(), "aaxxx".to_string()]
    );
    assert_eq!(
        afx.create_affixed_words("xxx", "B"),
        vec!["xxx".to_string(), "xxxcc".to_string()]
    );
    assert_eq!(
        afx.create_affixed_words("xxx", "AB"),
        vec![
            "xxx".to_string(),
            "aaxxx".to_string(),
            "xxxcc".to_string(),
            "aaxxxcc".to_string()
        ]
    );
}
