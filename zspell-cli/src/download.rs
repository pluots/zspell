//! Things required to download dictionaries from wooorm's repository
//!
//! This is a work in progress; entire section is largely unfinished
// TODO: should this move to `zspell` under a feature?

#![allow(unused)] // WIP

use std::cmp::{max, min};
use std::error::Error as StdError;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, Write};
use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;

use anyhow::{bail, ensure, Context};
use cfg_if::cfg_if;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use serde_json::Value;
use sha1::{Digest, Sha1};
use zspell_index::{DictionaryFormat, Downloadable, Index};

const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

// For default use, we get the content listing from Github. For testing,
// we use a dummy server.
cfg_if! {
    if #[cfg(not(test))] {
        const INDEX_URL:&str = "https://github.com/pluots/zspell-index/blob/main/zspell-index.json";

        fn get_index_url() -> String {
            INDEX_URL.to_owned()
        }
    } else {
        use httpmock::prelude::*;

        const INDEX_PATH: &str ="/zspell-index.json";

        fn test_server() -> &'static MockServer {
            static TEST_SERVER: OnceLock<MockServer> = OnceLock::new();
            TEST_SERVER.get_or_init(MockServer::start)
        }

        fn get_index_url() -> String {
            test_server().url(INDEX_PATH)
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
/// <https://git-scm.com/book/en/v2/Git-Internals-Git-Objects>
fn calculate_git_hash(bytes: &[u8]) -> [u8; 20] {
    let mut hasher = Sha1::new();
    let prefix = format!("blob {}\0", bytes.len());
    hasher.update(&prefix);
    hasher.update(bytes);
    hasher.finalize().into()
}

fn get_index(agent: &ureq::Agent) -> anyhow::Result<&Index> {
    static INDEX: OnceLock<Option<Index>> = OnceLock::new();
    static ERROR: OnceLock<String> = OnceLock::new();
    fn inner(agent: &ureq::Agent) -> anyhow::Result<Index> {
        agent
            .get(&get_index_url())
            .call()?
            .into_json()
            .map_err(Into::into)
    }

    let ret = INDEX
        .get_or_init(|| match inner(agent) {
            Ok(v) => Some(v),
            Err(e) => {
                ERROR.set(e.to_string()).unwrap();
                None
            }
        })
        .as_ref();

    match ret {
        Some(v) => Ok(v),
        None => bail!("{}", ERROR.get().unwrap()),
    }
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
            .truncate(true)
            .open(path)
            .context(format!("unable to open '{fname}' in '{dir}'"))
    } else {
        // Otherwise, use create_new to fail if it exists
        OpenOptions::new()
            .write(true)
            .read(true)
            .create_new(true)
            .open(path)
            .context(format!("file {fname} already exists in '{dir}'"))
    }
}

/// Download a single file to the given path, and create a progress bar while
/// doing so.
fn download_file_with_bar(
    path: &Path,
    overwrite: bool,
    agent: &ureq::Agent,
    // url: &str,
    // sha: &[u8],
    dl: &Downloadable,
) -> anyhow::Result<()> {
    const CHUNK_SIZE: usize = 100;

    let mut buffer = open_new_file(path, overwrite)?;

    let mut res_opt = None;
    let mut found_url = None;
    for url in dl.urls.iter() {
        res_opt = Some(agent.get(url).call());
        if matches!(res_opt, Some(Ok(..))) {
            found_url = Some(url);
            break;
        }
    }
    let Some(res) = res_opt else {
        bail!("no URLs found for the specified language");
    };
    let resp = res?;
    let url = found_url.unwrap();

    // Estimate content length for our buffer capacity & progress bar
    let expected_len: usize = match resp.header("Content-Length") {
        Some(hdr) => hdr.parse().expect("can't parse number"),
        None => dl.size.try_into().unwrap(),
    };

    let mut buf_len = 0usize;
    let mut buffer: Vec<u8> = Vec::with_capacity(expected_len);
    let mut reader = resp.into_reader().take(10_000_000);

    let pb = ProgressBar::new(expected_len.try_into().unwrap());
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
        .progress_chars("#>-"));
    pb.set_message(format!("Downloading {url}"));

    loop {
        buffer.extend_from_slice(&[0; CHUNK_SIZE]);
        let chunk = &mut buffer.as_mut_slice()[buf_len..buf_len + CHUNK_SIZE];
        let read_bytes = reader.read(chunk).expect("error reading stream");
        buf_len += read_bytes;
        pb.set_length(max(read_bytes, expected_len).try_into().unwrap());
        pb.set_position(buf_len.try_into().unwrap());

        if read_bytes == 0 {
            break;
        }
    }

    buffer.truncate(buf_len);

    match dl.hash.split_once(':') {
        Some(("sha1", digest)) => {
            let digest = hex::decode(digest).context("invalid hex {digest}")?;
            let hash = calculate_git_hash(&buffer);
            ensure!(hash == *digest, "error downloading file; checksum mismatch");
        }
        Some((alg, _)) => bail!("unsupported hash algorithm {alg}"),
        None => bail!("invalid hash string {}", dl.hash),
    }

    pb.finish_with_message(format!("Downloaded {} to {}", url, path.to_string_lossy()));

    Ok(())
}

// TODO: make pub
/// Download a single dictionary
fn download_dict(lang: &str, dest: &Path, overwrite: bool, _manifest: &Path) -> anyhow::Result<()> {
    let agent = make_agent();
    let index = get_index(&agent)?;

    let Some(lang) = index.items.iter().find(|x| x.lang.as_ref() == lang) else {
        bail!("unable to located a dictionary for language {lang}");
    };
    let name = &lang.lang;

    download_file_with_bar(
        &dest.join(format!("{name}.lic")),
        overwrite,
        &agent,
        &lang.lic,
    )?;

    match &lang.format {
        DictionaryFormat::Hunspell { aff, dic } => {
            download_file_with_bar(&dest.join(format!("{name}.dic")), overwrite, &agent, dic)?;
            download_file_with_bar(&dest.join(format!("{name}.aff")), overwrite, &agent, aff)?;
        }
        DictionaryFormat::Wordlist(v) => {
            download_file_with_bar(&dest.join(format!("{name}.wordlist")), overwrite, &agent, v)?;
        }
    }

    Ok(())
}

fn make_agent() -> ureq::Agent {
    ureq::builder()
        .timeout(Duration::from_secs(10))
        .user_agent(APP_USER_AGENT)
        .build()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use httpmock::Mock;
    use tempfile::tempdir;
    use test_mocks::*;

    use super::*;

    #[test]
    fn calculate_git_hash_ok() {
        // Use example from git help page
        assert_eq!(
            calculate_git_hash("what is up, doc?".as_bytes()),
            hex::decode("bd9dbf5aae1a3862dd1526723246b20206e5fc37")
                .unwrap()
                .as_slice()
        )
    }

    #[test]
    fn download_dict_ok() {
        let mocks = mock_de_at_index();
        let dir = tempdir().unwrap();

        download_dict("de-AT", dir.path(), false, &PathBuf::default()).unwrap();

        // Verify we created the expected paths
        let mut created_files = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .map(|x| x.file_name())
            .collect::<Vec<_>>();
        created_files.sort_unstable();

        assert_eq!(created_files, ["de-AT.aff", "de-AT.dic", "de-AT.lic"]);

        mocks.iter().for_each(Mock::assert);
    }
}

#[cfg(test)]
mod test_mocks {
    use std::fs;

    use httpmock::prelude::*;
    use httpmock::Mock;

    use super::*;

    // Content for our mock server
    pub const CONTENT_AFF: &str = "Affix Content\n";
    pub const CONTENT_AFF_HASH: &str = "a464def0d8bb136f20012d431b60faae2cc915b5";
    pub const CONTENT_DIC: &str = "Dictionary Content\n";
    pub const CONTENT_DIC_HASH: &str = "eee2f5c4eddac4175d67c00bc808032b02058b5d";
    pub const CONTENT_LIC: &str = "License Content\n";
    pub const CONTENT_LIC_HASH: &str = "c4d083267263c478591c4856981f32f31690456d";

    macro_rules! make_resp {
        ($path:expr, $ctype:expr, $body:expr) => {
            test_server().mock(|when, then| {
                when.method(GET).path($path);
                then.status(200)
                    .header("content-type", format!("{}; charset=utf-8", $ctype))
                    .body($body);
            })
        };
    }

    /// Create mocks to be used. Just store these in a dictionary for easy lookup
    pub fn mock_de_at_index<'a>() -> Vec<Mock<'a>> {
        vec![
            make_resp!(
                INDEX_PATH,
                "application/json",
                include_str!("../tests/files/sample-index.json")
                    .replace(r"{{ROOT_URL}}", &test_server().base_url())
            ),
            make_resp!("/dictionaries/de-AT/index.aff", "text/plain", CONTENT_AFF),
            make_resp!("/dictionaries/de-AT/index.dic", "text/plain", CONTENT_DIC),
            make_resp!("/dictionaries/de-AT/license", "text/plain", CONTENT_LIC),
        ]
    }
}
