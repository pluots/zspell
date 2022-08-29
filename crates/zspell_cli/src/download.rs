//! Things required to download dictionaries
//!
//!

use std::time::Duration;

use reqwest::blocking::Client;
use serde_json::Value;

// #[cfg(not(test))]
const ROOT_URL: &str = "https://api.github.com/repos/wooorm/dictionaries/contents/dictionaries";
static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[derive(Clone, Default)]
struct DictFile {
    affix: String,
    dictionary: String,
    license: String,
    lang: String,
}

fn download_dict(lang: &str) -> Result<DictFile, ()> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent(APP_USER_AGENT)
        .build()
        // TODO
        .unwrap();

    let resp = client.get(ROOT_URL).send();
    let txt = resp.unwrap().text().unwrap();
    let json: Value = serde_json::from_str(&txt).unwrap();

    let arr = match json {
        Value::Array(v) => v,
        _ => return Err(()),
    };
    let found_dir = arr
        .iter()
        .find(|x| x["name"] == lang)
        .map(|x| &x["url"])
        .unwrap();

    let dir_url = match found_dir {
        Value::String(s) => s,
        _ => return Err(()),
    };

    let resp = client.get(dir_url).send();
    let txt = resp.unwrap().text().unwrap();
    let json: Value = serde_json::from_str(&txt).unwrap();
    // https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=30e57b1769cfed178c039494f9e24f06
    Ok(DictFile::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::errors;
    // use std::{fs, io};
    // use tempfile::tempdir;

    #[test]
    fn thingy() {}
}
