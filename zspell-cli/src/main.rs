#![forbid(unsafe_code)]

use std::process::ExitCode;

use clap::Parser;

mod cli;
mod download;
mod spelling;

use spelling::spellcheck_cli;
use stringmetrics::levenshtein_limit;

fn main() -> ExitCode {
    let cli_parse = cli::Cli::parse();

    if let Some(cli::Commands::Lev {
        string_a,
        string_b,
        limit,
    }) = &cli_parse.command
    {
        println!("{}", levenshtein_limit(string_a, string_b, *limit));
        return ExitCode::SUCCESS;
    }

    spellcheck_cli(&cli_parse);

    ExitCode::SUCCESS
}
