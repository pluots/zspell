//! Tests for the command line interface

use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

#[test]
fn lev_basic_test() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("zspell")?;

    cmd.arg("lev")
        .arg("the quick brown fox")
        .arg("the slow brown flocks");
    cmd.assert().success().stdout(predicate::str::contains("9"));

    Ok(())
}
