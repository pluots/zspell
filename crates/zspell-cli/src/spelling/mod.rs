//! Helpers for CLI spelling features

use std::io::{self, BufRead, Write};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use zspell::errors::DictError;
use zspell::system::{create_dict_from_path, PKG_NAME, PKG_VERSION};
use zspell::Dictionary;

use crate::cli::Cli;

// A reminder that code is written by humans
const SALUTATIONS: [&str; 9] = [
    "goodbye",
    "auf Wiedersehen",
    "adios",
    "au revoir",
    "arrivederci",
    "annyeong",
    "sayōnara",
    "see you later calculator",
    "abyssinia",
];

#[inline]
pub fn spellcheck_stdin_runner(dic: &Dictionary) -> ExitCode {
    println!("started session");
    let stdin = io::stdin();

    // This is a false positive, see clippy #9135
    #[allow(clippy::significant_drop_in_scrutinee)]
    for line in stdin.lock().lines() {
        let unwrapped = match line {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Input error: {e}");
                return ExitCode::FAILURE;
            }
        };

        for word in dic.check_returning_list(unwrapped).unwrap() {
            println!("{}", &word)
        }
    }

    // Quick RNG without external crates
    let bye = SALUTATIONS[SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros() as usize
        % SALUTATIONS.len()];

    println!("\n\nsession ended, {bye}");

    ExitCode::SUCCESS
}

pub fn spellcheck_cli(cli: &Cli) -> ExitCode {
    print!("{PKG_NAME} {PKG_VERSION} loading dictionaries... ");

    io::stdout().flush().unwrap();

    let dict_path = if let Some(v) = cli.dict_path.as_ref() {
        v.as_str()
    } else {
        eprintln!("Dictionary path not specified. Please specify with `-d /path/to/dic`.");
        return ExitCode::FAILURE;
    };

    let dic = match create_dict_from_path(dict_path) {
        Ok(v) => v,
        Err(e) => {
            match e {
                DictError::FileError { fname, e } => {
                    eprintln!("Error opening '{fname}'; {e}")
                }
                _ => todo!(),
            };
            return ExitCode::FAILURE;
        }
    };

    if cli.generate_wordlist {
        for item in dic.iter_wordlist_items().unwrap() {
            println!("{item}");
        }
    } else {
        return spellcheck_stdin_runner(&dic);
    }

    ExitCode::SUCCESS
}
