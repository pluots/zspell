use std::fs::File;
use std::io::Error;
use std::{env, path::Path};

use clap::CommandFactory;
use clap_complete::{generate_to, shells::Shell};

include!("src/bin/cli/mod.rs");

fn main() -> Result<(), Error> {
    // Output directory will be a cargo-generated random directory
    let outdir = match env::var_os("OUT_DIR") {
        Some(outdir) => std::path::PathBuf::from(outdir),
        None => return Ok(()),
    };

    // Create a dummy file
    let stamp_path = Path::new(&outdir).join("zspell-stamp");
    if let Err(err) = File::create(&stamp_path) {
        panic!("failed to write {}: {}", stamp_path.display(), err);
    }

    let mut cmd = Cli::command();

    // Generate shell completion scripts for our
    for shell in [
        Shell::Bash,
        Shell::Elvish,
        Shell::Fish,
        Shell::PowerShell,
        Shell::Zsh,
    ] {
        let path = generate_to(
            shell,
            &mut cmd,       // We need to specify what generator to use
            "zspell",       // We need to specify the bin name manually
            outdir.clone(), // We need to specify where to write
        )?;

        println!("cargo:warning=completion file written to {:?}", path);
    }

    // Generate man pages
    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;

    let manpage_out = outdir.join("zspell.1");

    println!("cargo:warning=manpage written to {:?}", manpage_out);

    std::fs::write(manpage_out, buffer)?;

    Ok(())
}
