//! Tests for the levenshtein command line interface

use std::process::Command; // Run programs

use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions

#[test]
fn lev_basic() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("zspell")?;

    cmd.arg("lev")
        .arg("the quick brown fox")
        .arg("the slow brown flocks");
    cmd.assert().success().stdout(predicate::str::contains("9"));

    Ok(())
}
