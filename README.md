# Stringmetrics

This is a Rust library for approximate string matching that implements simple
algorithms such has Hamming distance, Levenshtein distance, Jaccard similarity,
and more, as well as a competent spellchecker that handles Hunspell
dictionaries.

This package comes with a library for programatic use, as well as a command line
interface. The library is usable via WASM.

Crate info:
[https://crates.io/crates/stringmetrics](https://crates.io/crates/stringmetrics)

Crate docs:
[https://docs.rs/stringmetrics/](https://docs.rs/stringmetrics/).

Crate source:
[https://github.com/pluots/stringmetrics-rust](https://github.com/pluots/stringmetrics-rust)


## Stringmetric Algorithms

One of the main purposes of this library is to provide a variety of string
metric functions. These include a few Levenshtein implementations (including
limit/max, weighted, and generic), Jaccard index, and a Hamming implementation.
These are all found in the `algorithms` module.


## Spellcheck

This is a spellchecker written completely in Rust. While maintaining
compatibility with the venerable Hunspell dictionary format, it does not rely on
Hunspell or any other underlying checker. NOTE: Spellchecker is currently in
alpha.

Spellcheck functionality is found in the `spellcheck` module.

### Functionality

NOTE: The spellcheck portion of this project is still under development and is
not guaranteed to work properly. Completed and future planned support include:

- [x] Basic prefix/sufix dictionary files
- [ ] Forbidden word handling
- [ ]
- [ ] Morphological/Phonetic handling

### Performance

In general, this program has been shown to be quite fast. On an average laptop,
benchmarks give approximately 40-50 ns per word. This is fast enough to
spellcheck the entire million words of the Harry Potter series in about 40 ms.

<!-- In fact, it actually beats Hunspell in a simple spellcheck test on a very large
file. There is no guarantee that this performance will stay however, after
adding fuller features.

```bash
time hunspell -d dictionaries/en -l < tests/files/odyssey.txt > /dev/null
1.25s user 0.01s system 95% cpu 1.325 total

time ./target/release/stringmetrics spell -d dictionaries/en < tests/files/odyssey.txt > /dev/null
0.17s user 0.01s system 91% cpu 0.199 total
``` -->

Simple benchmarks:

```bash
Spellcheck: compile dictionary
                        time:   [127.40 ms 132.32 ms 138.44 ms]
Found 9 outliers among 100 measurements (9.00%)
  9 (9.00%) high severe

Spellcheck: 1 correct word
                        time:   [35.343 ns 35.446 ns 35.563 ns]
Found 15 outliers among 100 measurements (15.00%)
  11 (11.00%) high mild
  4 (4.00%) high severe

Spellcheck: 1 incorrect word
                        time:   [46.577 ns 46.700 ns 46.853 ns]
Found 16 outliers among 100 measurements (16.00%)
  6 (6.00%) high mild
  10 (10.00%) high severe

Spellcheck: 15 correct words
                        time:   [537.31 ns 552.10 ns 568.80 ns]
Found 12 outliers among 100 measurements (12.00%)
  2 (2.00%) high mild
  10 (10.00%) high severe

Spellcheck: 15 incorrect words
                        time:   [741.72 ns 747.44 ns 755.19 ns]
Found 15 outliers among 100 measurements (15.00%)
  4 (4.00%) high mild
  11 (11.00%) high severe

Spellcheck: 188 word paragraph
                        time:   [6.9062 us 6.9259 us 6.9485 us]
Found 11 outliers among 100 measurements (11.00%)
  4 (4.00%) high mild
  7 (7.00%) high severe
```

Note that dictionary compiling is only a one-time task after a file is loaded.

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

**Note: this project was previously named "textdistance". Please make sure to
update all references.**
