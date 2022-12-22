//! Things required to download dictionaries
//!
//! This is a work in progress; entire section is largely unfinished

#![allow(unused)] // WIP

use std::cmp::min;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, Write};
use std::path::Path;
use std::time::Duration;

use anyhow::{bail, Context};
use cfg_if::cfg_if;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde_json::Value;
use sha1::{Digest, Sha1};

const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

// For default use, we get the content listing from Github. For testing,
// we use a dummy server.
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

/// A simple struct we can use for download info
/// This may hold URLs, destinations, or content
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
struct DownloadInfo {
    affix: String,
    dictionary: String,
    license: String,
    lang: String,
}

/// Perform the function that Git does to calculate its hash
///
/// Implementation taken from the git help page, located here
/// https://git-scm.com/book/en/v2/Git-Internals-Git-Objects
fn calculate_git_hash(s: &str) -> [u8; 20] {
    let mut tmp = String::from("blob ");
    tmp.push_str(&s.len().to_string());
    tmp.push('\0');
    tmp.push_str(s);

    let mut hasher = Sha1::new();
    hasher.update(tmp.as_bytes());
    let res: [u8; 20] = hasher.finalize().into();
    res
}

fn calculate_git_hash_buf<R: Read>(mut reader: R, len: usize) -> anyhow::Result<[u8; 20]> {
    let mut tmp = String::from("blob ");
    tmp.push_str(&len.to_string());
    tmp.push('\0');

    let mut hasher = Sha1::new();
    hasher.update(tmp.as_bytes());

    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer).unwrap();
        if count == 0 {
            break;
        }

        hasher.update(&buffer[..count]);
    }

    let res: [u8; 20] = hasher.finalize().into();
    Ok(res)
}

/// Helper function for getting the root URL that we can "patch" for testing
fn get_root_url() -> String {
    #[cfg(not(test))]
    return ROOT_URL.to_owned();

    #[cfg(test)]
    return TEST_SERVER.url("/contents/dictionaries");
}

/// Gather the URLs to download dictionary, affix, and license files from a client
///
/// Only collects the URLs, does not download them. Uses [`get_root_url`]
/// as a base then navigates one layer deeper.
async fn retrieve_urls(lang: &str, client: &Client) -> anyhow::Result<DownloadInfo> {
    let root_json: Value = client
        .get(get_root_url())
        .send()
        .await
        .context("error while sending request")?
        .text()
        .await
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
        .await
        .context("error while sending request")?
        .text()
        .await
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
fn get_dl_url_from_tree<F: Fn(&str) -> bool>(tree: &[Value], f: F) -> anyhow::Result<String> {
    let ctx_str = "could not locate a necessary file";
    // Collect the SHA sum and download URL of a file
    let tmp = tree
        .iter()
        .find(|x| x["name"].as_str().map(&f).unwrap_or(false))
        .map(|x| (x.get("sha"), x.get("download_url")))
        .context(ctx_str)?;

    let mut res = String::from("sha1$");
    res.push_str(tmp.0.context(ctx_str)?.as_str().context(ctx_str)?);
    res.push('$');
    res.push_str(tmp.1.context(ctx_str)?.as_str().context(ctx_str)?);

    Ok(res)
}

/// Open an existing file or create a new one, depending on overwrite
/// parameters.
fn open_new_file(path: &Path, overwrite: bool) -> anyhow::Result<File> {
    let fname = path
        .file_name()
        .map(|x| x.to_string_lossy())
        .unwrap_or_default();
    let dir_os = path.with_file_name("");
    let dir = dir_os.to_string_lossy();

    if overwrite {
        // If overwriting is allowed, just create or open the file
        OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(path)
            .context(format!("unable to open \"{fname}\" in \"{dir}\""))
    } else {
        // Otherwise, use create_new to fail if it exists
        OpenOptions::new()
            .write(true)
            .read(true)
            .create_new(true)
            .open(path)
            .context(format!("file {fname} already exists in \"{dir}\""))
    }
}

// Download a single file to the given path, and create a progress bar while
// doing so
async fn download_file_with_bar(
    path: &Path,
    overwrite: bool,
    client: &Client,
    url: &str,
    sha: &[u8],
) -> anyhow::Result<()> {
    let mut buffer = open_new_file(path, overwrite)?;

    let res = client.get(url).send().await?;
    let total_size = res.content_length().unwrap_or(100);

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
        .progress_chars("#>-"));
    pb.set_message(format!("Downloading {}", url));

    let mut finished_bytes: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        buffer.write_all(&chunk)?;

        let new = min(finished_bytes + (chunk.len() as u64), total_size);
        finished_bytes = new;
        pb.set_position(new);
    }

    let buf_len = buffer.stream_position().unwrap();
    buffer.rewind().context("error writing file").unwrap();

    let hash = calculate_git_hash_buf(&buffer, buf_len.try_into()?).unwrap();

    if hash != sha {
        bail!("error downloading file; checksum failure");
    }

    pb.finish_with_message(format!("Downloaded {} to {}", url, path.to_string_lossy()));

    Ok(())
}

// TODO: make pub
async fn download_dict(
    lang: &str,
    dest: &Path,
    overwrite: bool,
    _manifest: &Path,
) -> anyhow::Result<()> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent(APP_USER_AGENT)
        .build()
        .context("could not create HTTP client")?;

    let urls = retrieve_urls(lang, &client).await?;

    let fnames = DownloadInfo {
        affix: format!("{lang}.aff"),
        dictionary: format!("{lang}.dic"),
        license: format!("{lang}.license"),
        lang: String::default(),
    };

    // We control these strings, unwrap should be safe
    // Want to split "sha$some_sha_hex$some_url" into (some_sha_hex, some_url)

    fn split_url_sha(s: &str) -> (&str, &str) {
        s.split_once('$')
            .map(|x| x.1.split_once('$'))
            .unwrap()
            .unwrap()
    }

    let info_aff = split_url_sha(urls.affix.as_str());
    let info_dic = split_url_sha(urls.dictionary.as_str());
    let info_lic = split_url_sha(urls.license.as_str());

    download_file_with_bar(
        &dest.join(fnames.affix),
        overwrite,
        &client,
        info_aff.1,
        hex::decode(info_aff.0.as_bytes())?.as_slice(),
    )
    .await?;

    download_file_with_bar(
        &dest.join(fnames.dictionary),
        overwrite,
        &client,
        info_dic.1,
        hex::decode(info_dic.0.as_bytes())?.as_slice(),
    )
    .await?;

    download_file_with_bar(
        &dest.join(fnames.license),
        overwrite,
        &client,
        info_lic.1,
        hex::decode(info_lic.0.as_bytes())?.as_slice(),
    )
    .await?;

    // Download each with progress bar
    // Hash each file
    // Write download info to toml file

    println!("{urls:?}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use tempfile::tempdir;
    use test_mocks::*;

    use super::*;

    #[test]
    fn calculate_git_hash_ok() {
        // Use example from git help page
        assert_eq!(
            calculate_git_hash("what is up, doc?"),
            hex::decode("bd9dbf5aae1a3862dd1526723246b20206e5fc37")
                .unwrap()
                .as_slice()
        )
    }

    #[tokio::test]
    async fn retreive_urls_ok() {
        let mocks = mock_server_setup();
        let client = make_test_client();

        let urls = retrieve_urls("de-AT", &client).await.unwrap();
        // SHA sums joined with files
        let expected = DownloadInfo {
            affix: format!(
                "sha1${}${}",
                CONTENT_AFF_HASH,
                TEST_SERVER
                    .url("/main/dictionaries/de-AT/index.aff")
                    .as_str()
            ),
            dictionary: format!(
                "sha1${}${}",
                CONTENT_DIC_HASH,
                TEST_SERVER
                    .url("/main/dictionaries/de-AT/index.dic")
                    .as_str()
            ),
            license: format!(
                "sha1${}${}",
                CONTENT_LIC_HASH,
                TEST_SERVER.url("/main/dictionaries/de-AT/license").as_str()
            ),
            lang: "de-AT".to_owned(),
        };

        // TODO
        // mocks.dict_listing.assert();
        // mocks.de_at_listing.assert();

        assert_eq!(urls, expected);
    }

    #[tokio::test]
    async fn download_dict_ok() {
        let mocks = mock_server_setup();
        let dir = tempdir().unwrap();

        let res = download_dict("de-AT", dir.path(), false, &PathBuf::default()).await;

        println!("{res:?}");
        res.unwrap();

        let paths = fs::read_dir(dir.path()).unwrap();

        for path in paths {
            println!("Name: {}", path.unwrap().path().display())
        }

        // TODO: figure out why this isn't being asserted
        // mocks.dict_listing.assert();
        // mocks.de_at_listing.assert();
        // mocks.de_at_aff.assert();
        // mocks.de_at_dic.assert();
        // mocks.de_at_lic.assert();
    }
}

#[cfg(test)]
mod test_mocks {
    use std::fs;

    use httpmock::prelude::*;
    use httpmock::Mock;

    use super::*;

    pub struct TestMocks<'a> {
        pub dict_listing: Mock<'a>,
        pub de_at_listing: Mock<'a>,
        pub de_at_aff: Mock<'a>,
        pub de_at_dic: Mock<'a>,
        pub de_at_lic: Mock<'a>,
    }

    // Content for our mock server
    pub const CONTENT_DIC: &str = "Dictionary Content\n";
    pub const CONTENT_DIC_HASH: &str = "eee2f5c4eddac4175d67c00bc808032b02058b5d";
    pub const CONTENT_AFF: &str = "Affix Content\n";
    pub const CONTENT_AFF_HASH: &str = "a464def0d8bb136f20012d431b60faae2cc915b5";
    pub const CONTENT_LIC: &str = "License Content\n";
    pub const CONTENT_LIC_HASH: &str = "c4d083267263c478591c4856981f32f31690456d";

    macro_rules! make_resp {
        ($path:expr, $ctype:expr, $body:expr) => {
            TEST_SERVER.mock(|when, then| {
                when.method(GET).path($path);
                then.status(200)
                    .header("content-type", "$ctyle; charset=utf-8")
                    .body($body);
            })
        };
    }

    pub fn mock_server_setup<'a>() -> TestMocks<'a> {
        let dict_listing = make_resp!(
            "/contents/dictionaries",
            "application/json",
            fs::read_to_string("tests/files/dict_listing.json")
                .unwrap()
                .replace(r"{{ROOT_URL}}", &TEST_SERVER.base_url())
        );

        let de_at_listing = make_resp!(
            "/contents/dictionaries/de-AT",
            "application/json",
            fs::read_to_string("tests/files/de_at_listing.json")
                .unwrap()
                .replace(r"{{ROOT_URL}}", &TEST_SERVER.base_url())
        );

        let de_at_aff = make_resp!(
            "/main/dictionaries/de-AT/index.aff",
            "text/plain",
            CONTENT_AFF
        );
        let de_at_dic = make_resp!(
            "/main/dictionaries/de-AT/index.dic",
            "text/plain",
            CONTENT_DIC
        );
        let de_at_lic = make_resp!(
            "/main/dictionaries/de-AT/license",
            "text/plain",
            CONTENT_LIC
        );

        TestMocks {
            dict_listing,
            de_at_listing,
            de_at_aff,
            de_at_dic,
            de_at_lic,
        }
    }

    pub fn make_test_client() -> Client {
        Client::builder()
            .timeout(Duration::from_secs(5))
            .user_agent(APP_USER_AGENT)
            .build()
            .unwrap()
    }
}
