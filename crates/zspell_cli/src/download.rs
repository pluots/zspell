//! Things required to download dictionaries
//!
//!

use std::process::ExitCode;
use std::{path::Path, time::Duration};

use anyhow::Context;
use cfg_if::cfg_if;
use reqwest::blocking::Client;
use serde_json::Value;

const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

cfg_if! {
    if #[cfg(not(test))] {
        const ROOT_URL: &str = "https://api.github.com/repos/wooorm/dictionaries/contents/dictionaries";
    } else {
        use lazy_static::lazy_static;
        use httpmock::prelude::*;

        lazy_static!{
            static ref TEST_SERVER: MockServer = MockServer::start();
        }
    }
}

/// Helper function for getting the root URL that we can use when testing
fn get_root_url() -> String {
    cfg_if! {
        if #[cfg(not(test))] {
            ROOT_URL.to_owned()
        } else {
            TEST_SERVER.url("/contents/dictionaries")
        }
    }
}

/// A simple struct we can use for download info
/// This may hold URLs, destinations, or content
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
struct DownloadInfo {
    affix: String,
    dictionary: String,
    license: String,
    lang: String,
}

/// Gather the URLs to download dictionary, affix, and license files from
///
/// Only collects the URLs, does not download them
fn retrieve_urls(lang: &str, client: &Client) -> anyhow::Result<DownloadInfo> {
    let root_json: Value = client
        .get(get_root_url())
        .send()
        .context("error while sending request")?
        .text()
        .map(|txt| {
            serde_json::from_str(&txt).context("error understanding server response at root")
        })??;

    // Get the URL of the directory to download
    let dir_url = root_json
        .as_array()
        .context("Data is incorrectly formatted")?
        .iter()
        .find(|x| x["name"] == lang && x["type"] == "dir")
        .map(|x| &x["url"])
        .context("Unable to locate language")?
        .as_str()
        .context("Data is incorrectly formatted")?;

    // Get the listing of that directory
    let dir_json: Value = client
        .get(dir_url)
        .send()
        .context("error while sending request")?
        .text()
        .map(|txt| {
            serde_json::from_str(&txt).context("error understanding server response at dir")
        })??;

    let dir_listing = &dir_json
        .as_array()
        .context("error listing server directory")?;

    let affix = get_dl_url_from_tree(dir_listing, |s| s.ends_with(".aff"))?;
    let dictionary = get_dl_url_from_tree(dir_listing, |s| s.ends_with(".dic"))?;
    let license = get_dl_url_from_tree(dir_listing, |s| s.ends_with("license"))?;

    let res = DownloadInfo {
        affix,
        dictionary,
        license,
        lang: lang.to_string(),
    };

    Ok(res)
}

/// Take in a JSON tree and locate one where the name matches the specified pattern
fn get_dl_url_from_tree<F: Fn(&str) -> bool>(tree: &Vec<Value>, f: F) -> anyhow::Result<String> {
    let res = tree
        .iter()
        .map(|x| {
            println!("name: {}", x["name"].as_str().expect("ERROR"));
            x
        })
        .find(|x| x["name"].as_str().map(&f).unwrap_or(false))
        .context("could not locate a necessary file")?
        .get("download_url")
        .context("download_url not found")?
        .as_str()
        .context("download_url not found")?;

    Ok(res.to_owned())
}

pub fn download_dict(lang: &str, _dest: &Path, _overwrite: bool) -> ExitCode {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent(APP_USER_AGENT)
        .build()
        // TODO
        .unwrap();

    let urls = match retrieve_urls(lang, &client) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error locatind files: {e}");
            return ExitCode::FAILURE;
        }
    };

    println!("{urls:?}");

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    use httpmock::prelude::*;
    use std::fs;

    #[test]
    // fn thingy() {download_dict("en", Path::new("~"), true);}

    fn retreive_urls_ok() {
        let dict_listing = TEST_SERVER.mock(|when, then| {
            when.method(GET).path("/contents/dictionaries");
            then.status(200)
                .header("content-type", "application/json; charset=utf-8")
                .body(
                    fs::read_to_string("tests/files/dict_listing.json")
                        .unwrap()
                        .replace(r"{{ROOT_URL}}", &TEST_SERVER.base_url()),
                );
        });
        let de_listing = TEST_SERVER.mock(|when, then| {
            when.method(GET).path("/contents/dictionaries/de-AT");
            then.status(200)
                .header("content-type", "application/json; charset=utf-8")
                .body(
                    fs::read_to_string("tests/files/de_at_listing.json")
                        .unwrap()
                        .replace(r"{{ROOT_URL}}", &TEST_SERVER.base_url()),
                );
        });

        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent(APP_USER_AGENT)
            .build()
            .unwrap();

        let urls = retrieve_urls("de-AT", &client).unwrap();
        let expected = DownloadInfo {
            affix: TEST_SERVER.url("/main/dictionaries/de-AT/index.aff"),
            dictionary: TEST_SERVER.url("/main/dictionaries/de-AT/index.dic"),
            license: TEST_SERVER.url("/main/dictionaries/de-AT/license"),
            lang: "de-AT".to_owned(),
        };

        dict_listing.assert();
        de_listing.assert();

        assert_eq!(urls, expected);
    }
}
