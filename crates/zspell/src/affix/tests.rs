//! Affix tests
use util::TestCollection;

use super::*;

#[test]
fn test_flagtype_convert_ok() {
    assert_eq!(FlagType::Ascii.convert_flag("T"), Ok(84));
}
