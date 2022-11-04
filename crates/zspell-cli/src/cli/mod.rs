use clap::{Parser, Subcommand};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Path to a dictionary file. Specify e.g. dictionaries/de_DE if
    /// dictionaries/de_DE.aff and dictionaries/de_DE.dic exist
    #[arg(short, long)]
    pub dict_path: Option<String>,

    /// Whether to print misspelled words
    #[arg(short, long, default_value_t = false)]
    pub list_misspelled: bool,

    /// Print the a compiled dictionary's word list to stdout
    #[arg(long, default_value_t = false)]
    pub generate_wordlist: bool,

    /// Print the search path and found dictionaries
    #[arg(long, default_value_t = false)]
    pub show_dictionaries: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Levenshtein distance tools
    Lev {
        /// The start string to calculate distance from
        string_a: String,

        /// The end string to calculate distance to
        string_b: String,

        /// Specify a maximum difference limit for the levenshthein distance
        #[arg(short, long, default_value_t = 1000)]
        limit: u32,
    },
}
