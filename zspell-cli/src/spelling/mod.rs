//! Helpers for CLI spelling features

use std::io::{self, BufRead, Write};
use std::process::ExitCode;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use zspell::error::Error;
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

pub fn spellcheck_cli(cli: &Cli) -> ExitCode {
    eprint!("{PKG_NAME} {PKG_VERSION} loading dictionaries... ");

    io::stdout().flush().unwrap();

    let dict_path = if let Some(v) = cli.dict_path.as_ref() {
        v.as_str()
    } else {
        eprintln!("Dictionary path not specified. Please specify with `-d /path/to/dic`.");
        return ExitCode::FAILURE;
    };

    let load_start = Instant::now();
    let dict = match create_dict_from_path(dict_path) {
        Ok(v) => v,
        Err(e) => {
            match e {
                Error::Io(e) => eprintln!("IO error: {e}"),
                Error::Parse(e) => eprintln!("Error parsing: {e}"),
                Error::Build(e) => eprintln!("Error building: {e}"),
                Error::Regex(e) => eprintln!("Regex error: {e}"),
                _ => unreachable!(),
            };
            return ExitCode::FAILURE;
        }
    };
    let load_time = load_start.elapsed().as_secs_f32();
    let wc = dict.wordlist().inner().len() + dict.wordlist_nosuggest().inner().len();
    eprintln!("loaded {wc} words in {load_time:.2}s. started session");

    if cli.generate_wordlist {
        todo!();
        // for item in dic.iter_wordlist_items().unwrap() {
        //     println!("{item}");
        // }
    } else if cli.analyze {
        runner_morph_analysis(&dict);
    } else if cli.stem {
        runner_stemming(&dict);
    } else {
        runner_spellcheck(&dict);
    }

    // Quick RNG without external crates
    let bye = SALUTATIONS[SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros() as usize
        % SALUTATIONS.len()];

    eprintln!("\n\nsession ended, {bye}");

    ExitCode::SUCCESS
}

fn runner_spellcheck(dict: &Dictionary) {
    stdin_interactive_runner(|line| {
        for (_, misspelled) in dict.check_indices(&line) {
            println!("{misspelled}");
        }
    })
}

/// From hunspell:
///
/// ```text
/// banana foo drinkable dofjjos
/// banana banana
///
/// foo foo
///
/// drinkable drinkable
/// drinkable drink
///
/// dofjjos
/// ```
fn runner_stemming(dict: &Dictionary) {
    stdin_interactive_runner(|line| {
        for entry in dict.entries(&line) {
            if let Some(stems) = entry.stems() {
                for stem in stems {
                    println!("{} {stem}", entry.word());
                }
            } else {
                println!("{}", entry.word())
            }
        }
        println!();
    })
}

fn runner_morph_analysis(dict: &Dictionary) {
    stdin_interactive_runner(|line| {
        for entry in dict.entries(&line) {
            print!("{}", entry.word());

            if let Some(morphs) = entry.analyze() {
                for morph in morphs {
                    print!(" {morph}");
                }
            } else {
                println!("{}", entry.word())
            }
        }
        println!("\n");
    })
}

/// Common runner interface that calls a function once per stdin line
// FIXME: if not a tty, lock output once before writing
fn stdin_interactive_runner<F>(f: F)
where
    F: Fn(String),
{
    let stdin = io::stdin();
    // This is a false positive, see clippy #9135
    // #[allow(clippy::significant_drop_in_scrutinee)]
    for line in stdin.lock().lines() {
        let line_val = line.expect("received invalid input from IO!");
        f(line_val);
    }
}
