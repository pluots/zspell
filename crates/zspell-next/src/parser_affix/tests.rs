use super::*;

#[test]
fn test_line_key_parser() {
    let txt1 = "LANG apple";
    let txt2 = "LANG apple\nLANG banana";
    let txt3 = "LANG failure";

    fn get_lang(s: &str) -> Result<AffixNode, ParseError> {
        if s == "apple" {
            Ok(AffixNode::Language("apple".to_string()))
        } else {
            Err(ParseError {
                msg: "failure".to_owned(),
            })
        }
    }

    assert_eq!(
        line_key_parser(txt1, "LANG", get_lang),
        Ok(Some((AffixNode::Language("apple".to_owned()), "")))
    );
    assert_eq!(
        line_key_parser(txt2, "LANG", get_lang),
        Ok(Some((
            AffixNode::Language("apple".to_owned()),
            "\nLANG banana"
        )))
    );
    assert_eq!(
        line_key_parser(txt3, "LANG", get_lang),
        Err(ParseError::new_simple("failure"))
    );
}
