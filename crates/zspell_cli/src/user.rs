use reqwest::blocking::Client;
use std::time::Duration;

// Probably want to move this to a separate mod, "user" or so
pub fn download_dict() -> Result<(), ()> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        // TODO
        .unwrap();
    let resp = client.get("http://httpbin.org/").send();
    println!("{resp:#?}");
    println!("{:#?}", resp.unwrap().text());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::errors;
    // use std::{fs, io};
    // use tempfile::tempdir;

    #[test]
    fn thingy() {
        download_dict();
    }
}
