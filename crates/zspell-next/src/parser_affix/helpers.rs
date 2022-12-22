use std::string::ParseError;

use lazy_static::lazy_static;
use regex::Regex;

use super::types::AffixNode;
use super::{line_splitter, ParseResult};
use crate::affix::types::RuleGroup;

lazy_static! {
    static ref RE_AFX_RULE_HEADER: Regex = Regex::new(r"^(?P<flag>\w+)\s(?P<xprod>\w+)\s(?P<num>\w+)$").unwrap();
    static ref RE_AFX_RULE_BODY: Regex = Regex::new(r"^(?P<flag>\w+)\s+(?P<strip_chars>\w+)\s+(?P<affix>\w+)\s+(?P<condition>\S+)(?:$|\s+(?P<morph>.+)$)").unwrap();
}

fn affix_table_parser<'a, F>(s: &'a str, key: &str, f: F) -> ParseResult<'a>
where
    F: FnOnce(Vec<RuleGroup>) -> Result<AffixNode, ParseError>,
{
    let Some((work, mut residual)) = line_splitter(s, key) else {
        return Ok(None);
    };

    todo!()
}
