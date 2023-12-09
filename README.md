# ZSpell

This project is a spellchecker written completely in Rust, that maintains
compatibility with the venerable Hunspell dictionary format. It is entirely
native and does not rely on any other backends (Enchant, Hunspell, Aspell,
etc.). This library also has the goal of being usable via WASM. Full Unicode
support is baked in.

The library side has a stabalized checker, but the suggestion API is not yet
finalized. The CLI is usable but not yet considered stabalized. See
[Feature Status](#feature-status) for more information on what is available.

Here are some useful quick links:

- Crate info: <https://crates.io/crates/zspell>
- Crate CLI docs (incomplete): <https://pluots.github.io/zspell/>
- Crate library docs: <https://docs.rs/zspell/>
- Python library page: <https://pypi.org/project/zspell/>
- Crate source: <https://github.com/pluots/zspell>

## Interfaces

This project exposes multiple interfaces to its spellchecker, listed in this
section.

### Command Line Interface

Just want to use this spellchecker from the command line? Check out the book,
located here <https://pluots.github.io/zspell/>, for a more in-depth explanation
of installation and usage.

If you don't want to read further, the easiest way to get started is to download
a prebuilt binary from here: <https://github.com/pluots/zspell/releases>.

### Rust Library Interface

This project also aims to create a fully functional spellchecking library, for
easy programmatic use. See the documentation for the library side here
<https://docs.rs/zspell/>. This also includes a lot of design methodology
discussions, for those who are interested.

### Python Interface

There is a python wrapper for this library with prebuilt wheels, available here:
<https://pypi.org/project/zspell/>. Its source is located in the
[zspell-py crate](zspell-py).

### Usage via WASM

The library API should work out of the box. Official WASM bindings will be added
at some point.

## Feature Status

| Feature                        | Available via Library | Available via CLI | Tracking Issue                                    |
| ------------------------------ | --------------------- | ----------------- | ------------------------------------------------- |
| Basic spellcheck functionality | ✓                     | ✓                 |                                                   |
| Forbidden word handling        | ✓                     | ✓                 | [#17](https://github.com/pluots/zspell/issues/17) |
| Stemming                       | ✓                     | ✓                 |                                                   |
| Morph analysis                 | ✓                     | ✓                 |                                                   |
| Suggestions                    | WIP                   | ✕                 | [#16](https://github.com/pluots/zspell/issues/16) |
| Compound word handling         | ✕                     | ✕                 |                                                   |
| Full Morph/Phone Handling      | WIP                   | ✕                 |                                                   |
| Python Interface               | Beta                  | N/A               | [#18](https://github.com/pluots/zspell/issues/18) |
| Prebuilt WASM bindings         | ✕                     | N/A               | [#19](https://github.com/pluots/zspell/issues/19) |

## Performance

This repository has the goal of highly prioritizing the most expected usage,
i.e., that most words to be checked are correct. With optimizations based around
this concept and with the modern computers now able to store entire compiled
word lists in memory (~20 MiB), `zspell` tends to outperform other
spellcheckers.

## MSRV

This library relies on features from Rust 1.65, so that is our current minimum
supported version. Our CI validates this for the library and examples.

The CLI and test runner require newer features and do not keep a specific MSRV.

## Test suite

This project keeps a test suite located in `zspell/test-suite` (symlinked to
`test-suite`). Each file has a simple format that combines a simple affix and
dictionary file. To add a test, just duplicate and edit `0-example.test`.

File names are as follows:

- `0-*`: meta tests that do not get run
- `b-*`: basic functionality tests
- `h-*`: tests that come from the Hunspell test suite
- `i000-*`: tests that address specific issues

## License

See the LICENSE file for license information. The provided license does allow
for proprietary use and adaptation; that being said, I kindly suggest that if
you come up with an improvement, you submit a pull request and help us all out
:)

### Test suite license

Some tests are taken from Hunspell's test suite. Hunspell has various licenses,
we select MPL and include a SPDX notice on relevant files.

### Dictionary data license

The dictionaries provided in this repository for testing purposed have been
obtained under license. These files have been sourced from here:
[https://github.com/wooorm/dictionaries](https://github.com/wooorm/dictionaries)

These dictionaries are licensed under various licenses, different from that of
this project. Please see the applicable `.license` file withing the
`dictionaries/` directory.
