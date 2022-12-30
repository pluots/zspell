//! Affix tests

use super::*;

#[test]
fn test_flagtype_convert_ok() {
    assert_eq!(FlagType::Ascii.str_to_flag("T"), Ok(84));
}
