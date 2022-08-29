//! Things required to download dictionaries
//!
//!

use std::{path::Path, time::Duration};

use anyhow::{bail, Context};
use reqwest::blocking::Client;
use serde_json::Value;

use zspell::unwrap_or_ret_err;

// #[cfg(not(test))]
const ROOT_URL: &str = "https://api.github.com/repos/wooorm/dictionaries/contents/dictionaries";
static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

/// A simple struct we can use for download info
/// This may hold URLs, destinations, or content
#[derive(Clone, Default)]
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
        .get(ROOT_URL)
        .send()
        .context("error while sending request")?
        .text()
        .map(|txt| serde_json::from_str(&txt).context("error understanding server response"))?
        .context("error understanding server resposne")?;

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
        .map(|txt| serde_json::from_str(&txt).context("error understanding server response"))?
        .context("error understanding server resposne")?;

    let tree = &dir_json["tree"]
        .as_array()
        .context("error listing server directory")?;

    let affix = get_dl_url_from_tree(tree, |s| s.ends_with(".aff"))?;
    let dictionary = get_dl_url_from_tree(tree, |s| s.ends_with(".dic"))?;
    let license = get_dl_url_from_tree(tree, |s| s.ends_with(".license"))?;

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
        .find(|x| x["name"].as_str().map(&f).unwrap_or(false))
        .context("could not locate affix file")?
        .get("download_url")
        .context("download_url not found")?
        .as_str()
        .context("download_url not found")?;
    Ok(res.to_owned())
}

fn download_dict(lang: &str, dest: &Path, overwrite: bool) -> anyhow::Result<DownloadInfo> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent(APP_USER_AGENT)
        .build()
        // TODO
        .unwrap();

    let x = retrieve_urls(lang, &client)?;

    Ok(DownloadInfo::default())
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
