# ZSpell

This is a Rust library implementing a spellchecker that handles Hunspell
dictionaries. It comes with a library for programmatic use, as well as a command
line interface. The library is usable via WASM.

Crate info:
[https://crates.io/crates/zspell](https://crates.io/crates/zspell)

Crate docs:
[https://docs.rs/zspell/](https://docs.rs/zspell/).

Crate source:
[https://github.com/pluots/zspell](https://github.com/pluots/zspell)


## Spellcheck

This is a spellchecker written completely in Rust. While maintaining
compatibility with the venerable Hunspell dictionary format, it does not rely on
Hunspell or any other underlying checker. NOTE: Spellchecker is currently in
alpha.

Spellcheck functionality is found in the `spellcheck` module.

### Functionality

NOTE: The spellcheck portion of this project is still under development and is
not guaranteed to work properly. Completed and future planned support include:

- [x] Basic prefix/suffix dictionary files
- [ ] Forbidden word handling
- [ ]
- [ ] Morphological/Phonetic handling

### Performance

In general, this program has been shown to be quite fast. On an average laptop,
benchmarks give approximately 40-50 ns per word. This is fast enough to
spellcheck the entire million words of the Harry Potter series in about 40 ms.

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
