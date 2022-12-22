use std::env;
use std::fs::File;
use std::io::Error;
use std::path::{Path, PathBuf};

use clap::{Command, CommandFactory};
use clap_complete::generate_to;
use clap_complete::shells::Shell;

include!("src/cli/mod.rs");

fn build_shell_completion(cmd: &mut Command, outdir: &PathBuf) -> Result<(), Error> {
    // Generate shell completion scripts for our
    for shell in [
        Shell::Bash,
        Shell::Elvish,
        Shell::Fish,
        Shell::PowerShell,
        Shell::Zsh,
    ] {
        let path = generate_to(
            shell, cmd,      // We need to specify what generator to use
            "zspell", // We need to specify the bin name manually
            outdir,   // We need to specify where to write
        )?;

        println!("cargo:warning=completion file written to {:?}", path);
    }

    Ok(())
}

fn build_man_pages(cmd: Command, outdir: &Path) -> Result<(), Error> {
    // Generate man pages
    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Default::default();

    man.render(&mut buffer)?;

    let manpage_out = outdir.join("zspell.1");

    println!("cargo:warning=manpage written to {:?}", manpage_out);

    std::fs::write(manpage_out, buffer)?;

    Ok(())
}

fn main() -> Result<(), Error> {
    // Output directory will be a cargo-generated random directory
    let outdir = match env::var_os("OUT_DIR") {
        Some(outdir) => std::path::PathBuf::from(outdir),
        None => return Ok(()),
    };

    let profile = std::env::var("PROFILE").unwrap();

    // Don't generate outputs if we're in debug mode
    match profile.as_str() {
        "debug" => (),
        _ => {
            // Create a dummy file to help find the latest output
            let stamp_path = Path::new(&outdir).join("zspell-stamp");
            if let Err(err) = File::create(&stamp_path) {
                panic!("failed to write {}: {}", stamp_path.display(), err);
            }

            let mut cmd = Cli::command();

            build_shell_completion(&mut cmd, &outdir)?;
            build_man_pages(cmd, &outdir)?;
        }
    }

    Ok(())
}
