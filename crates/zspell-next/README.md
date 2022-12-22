# ZSpell

This project is a spellchecker written completely in rust, that maintains
compatibility with the venerable Hunspell dictionary format. It is entirely
native and does not rely on any other backends (Enchant, Hunspell, Aspell,
etc.). This library also has the goal of being usable via WASM.

The library side has a stabalized checker, but the suggestion API is not yet
finalized. The CLI side is usable but not yet considered stabalized.
See [Feature Status](#feature-status) for more information on what is available.

Here are some useful quick links:

- Crate info: <https://crates.io/crates/zspell>
- Crate CLI docs: <https://pluots.github.io/zspell/>
- Crate library docs: <https://docs.rs/zspell/>
- Python library page: <https://pypi.org/project/zspell/>
- Crate source: <https://github.com/pluots/zspell>

## Interfaces

This project exposes multiple interfaces to its spellchecker, listed in
this section.

### CLI Interface

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

There is a python wrapper for this library with prebuilt wheels, available
here: <https://pypi.org/project/zspell/>. Its source is located
in the [zspell-py crate](crates/zspell-py).

### Usage via WASM

The library API should work out of the box. Official WASM bindings will be
added at some point.

## Feature Status

| Feature                        | Available via Library | Available via CLI | Tracking Issue |
|--------------------------------|-----------------------|-------------------|----------------|
| Basic spellcheck functionality | ✓                     | ✓                 |                |
| Forbidden word handling        | ✕                     | ✕                 | [#17](https://github.com/pluots/zspell/issues/17) |
| Suggestions                    | ✕                     | ✕                 | [#16](https://github.com/pluots/zspell/issues/16) |
| Compound word handling         | ✕                     | ✕                 |                |
| Full Morph/Phone Handling      | ✕                     | ✕                 |                |
| Python Interface               | ✕                     | ✕                 | [#18](https://github.com/pluots/zspell/issues/18) |
| Prebuilt WASM bindings         | ✕                     | ✕                 | [#19](https://github.com/pluots/zspell/issues/19) |

## Performance

This repository has the goal of highly prioritizing the most expected usage,
i.e., that most words to be checked are correct. With optimizations based around
this concept and with the modern computers now able to store entire compiled
word lists in memory (~2 MiB), `zspell` tends to outperform other spellcheckers.

## License

See the LICENSE file for license information. The provided license does allow
for proprietary use and adaptation; that being said, I kindly suggest that if
you come up with an improvement, you submit a pull request and help us all out
:)

### Dictionary data license

The dictionaries provided in this repository for testing purposed have been
obtained under license. These files have been sourced from here:
[https://github.com/wooorm/dictionaries](https://github.com/wooorm/dictionaries)

These dictionaries are licensed under various licenses, different from that of
this project. Please see the applicable `.license` file withing the
`dictionaries/` directory.
