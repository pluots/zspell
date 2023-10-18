use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// If specified, run spellchecking on a file
    pub file: Option<PathBuf>,

    /// Path to a dictionary file. Specify e.g. dictionaries/de_DE if
    /// dictionaries/de_DE.aff and dictionaries/de_DE.dic exist
    #[arg(short = 'd', long)]
    pub dict_path: Option<String>,

    /// Whether to print misspelled words
    #[arg(short = 'l', long, default_value_t = false)]
    pub misspelled_words: bool,

    /// Whether to print lines with misspelled words
    #[arg(short = 'L', long, default_value_t = false)]
    pub misspelled_lines: bool,

    /// Print the a compiled dictionary's word list to stdout and exit
    #[arg(long, default_value_t = false)]
    pub generate_wordlist: bool,

    /// Enable morpological analysis mode
    #[arg(short = 'm', long, default_value_t = false)]
    pub analyze: bool,

    /// Enable word stemming mode
    #[arg(short = 's', long, default_value_t = false)]
    pub stem: bool,

    /// Print the search path and found dictionaries
    #[arg(short = 'D', long, default_value_t = false)]
    pub show_dictionaries: bool,

    /// Add a text or personal dictionary
    #[arg(short = 't', long, default_value_t = false)]
    pub text_dictionary: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

impl Cli {
    pub fn validate(&self) -> Result<(), String> {
        if self.analyze && self.stem {
            Err("cannot use analysis and stemming together".into())
        } else {
            Ok(())
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Calculate levenshtein distance
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
