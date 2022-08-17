use clap::{Parser, Subcommand};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
pub struct Cli {
    /// Path to a dictionary file. Specify e.g. dictionaries/de_DE if
    /// dictionaries/de_DE.aff and dictionaries/de_DE.dic exist
    #[clap(short, long, value_parser)]
    pub dict_path: Option<String>,

    /// Whether to print misspelled words
    #[clap(short, long, value_parser, default_value_t = false)]
    pub list_misspelled: bool,

    /// Print the a compiled dictionary's word list to stdout
    #[clap(long, value_parser, default_value_t = false)]
    pub generate_wordlist: bool,

    /// Print the search path and found dictionaries
    #[clap(long, value_parser, default_value_t = false)]
    pub show_dictionaries: bool,

    #[clap(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
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
