# Installation

## Using as a library

Most people will want to use ZSpell as a library. To do so, add the following to
`Cargo.toml`:

```toml
[dependencies]
zspell = "0.4"
```

For library usage, see the API reference at <https://docs.rs/zspell>.

## Installing ZSpell CLI

### Installing a prebuilt binary

The easiest way to get started is to download a prebuilt binary for your system.
Binaries are avilable for for Windows, Linux, and Mac on the x86_64 platform.
These do not require anything else to be installed.

Head to <https://github.com/pluots/zspell/releases> and download the latest
binary for your system. Simply extract the download and run the executable.

If you would like the tool to be accessible from anywhere on your system, you
will need to copy or link this executable to a location that is in your system
path.

### Installing via Cargo

If you already have rust installed and would like to install zspell via Cargo,
this is fairly straightforward:

```sh
cargo install zspell-cli --locked
```

### Building from source

If you would like to build the latest version (potentially unreleased) from
source without installing (e.g. for development purposes), that can be done as
follows:

```sh
git clone https://github.com/pluots/zspell
cd zspell
cargo build --package zspell-cli --release
```
