mod spelling;

use std::process::ExitCode;

use clap::{Parser, Subcommand};

use spelling::spellcheck_cli;
use stringmetrics::levenshtein_limit;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
pub struct Cli {
    /// Path to a dictionary file. Specify e.g. dictionaries/de_DE if
    /// dictionaries/de_DE.aff and dictionaries/de_DE.dic exist
    #[clap(short, long, value_parser)]
    dict_path: Option<String>,

    /// Whether to print misspelled words
    #[clap(short, long, value_parser, default_value_t = false)]
    list_misspelled: bool,

    /// Print the a compiled dictionary's word list to stdout
    #[clap(long, value_parser, default_value_t = false)]
    generate_wordlist: bool,

    /// Print the search path and found dictionaries
    #[clap(long, value_parser, default_value_t = false)]
    show_dictionaries: bool,

    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Levenshtein distance tools
    Lev {
        /// The start string to calculate distance from
        #[clap(value_parser)]
        string_a: String,

        /// The end string to calculate distance to
        #[clap(value_parser)]
        string_b: String,

        /// Specify a maximum difference limit for the levenshthein distance
        #[clap(short, long, value_parser, default_value_t = 1000)]
        limit: u32,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Lev {
            string_a,
            string_b,
            limit,
        }) => {
            println!("{}", levenshtein_limit(string_a, string_b, *limit));
            return ExitCode::SUCCESS;
        }
        None => {}
    };

    spellcheck_cli(&cli);

    ExitCode::SUCCESS
}
