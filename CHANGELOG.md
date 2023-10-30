# Changelog

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Added

### Changed

### Removed




## [0.5.1] - 2023-10-30

### Additions

Publish the work in progress command line interface under crate name
`zspell-cli`.

### Changed

- Add support for nonstandard morphological information types.
- Fix issue where all morph information for an affix rule were being reported,
  rather than just that for the relevant pattern. Fixes
  [#73](https://github.com/pluots/zspell/issues/73).
- Rewrite the `.dic` file parser.
- Improve handling of morphological information in dictionary files.

## [0.5.0] - 2023-10-30

Immediately superceded release, see 0.5.1.

## [0.4.1] - 2023-10-18

### Additions

- Add `Dictionary::{entry, entries}` which allow stemming and morphological
  analysis.

### Fixed

- Corrected error message for parsing affix headers
- Corrected pattern matching for groups that include hyphens
- Default no longer has a nosuggest flag
- Change output directory to use cargo directory rather than source

### Changed

- Remove features `unstable-analysis` and `unstable-stem` since functionality is
  now public
- Wordlist now correctly applies more than one affix rule if it is available
- Moved `DictBuilder::config` behind `zspell-unstable`
- [build] update CI workflows
- [internal] make some changes from `TryFrom` to `FromStr`
- [internal] refactor test system to support stemming and morphological analysis

## [0.4.0] - 2023-10-18

Immediately superceded release, see 0.4.1.

## [0.3.3] - 2023-01-01

### Changed

- [build] update python release workflow

## [0.3.2] - 2023-01-01

### Changed

- `.dic` parser now ignores lines that start with a tab (sometimes used for
  comments)
- Updated python documentation

## [0.3.1] - 2022-12-30

Minor patch to build system workflow

## [0.3.0] - 2022-12-30

This change is a huge rewrite of the library! Hopefully this will pave the way
forward for more features and easier growth.

### Changes

- Added `DictBuilder` to simplify dictionary creation
- Removed `affix::Config` as the representation was limiting & clunky
- The methods on `Dictionary` are now infallible since an uncompiled dictionary
  can no longer be created
- Rewrote the `error` module
- Simplified imports, everything needed is now top-level
- Rewrote affix file parser so it is much more efficient and now handles all
  known keys. We do not yet act on all possible values.
- Rewrote the dictionary & personal wordlist parsers

### Additions

- `check_indices` is now available to return better information about the
  location of errors
- Python modules now have correct bindings (horray!)

There are also a few new APIs that are feature gated. They should be considered
very unstable until those feature gates are removed.

- Suggestions
- Stemming
- Morphological analysis
- System tools. These were previously public but have been moved behind the
  feature gate.

## [0.2.2] - 2022-11-04

Minor bups in the dependency list

## [0.2.1] - 2022-11-04

### Changes

- Changed word breaking to use unicode segmentation, as suggested by
  @saona-raimundo

## [0.2.0] - 2022-11-04

### Additions

- Ability to automatically locate dictionaries on the system, WIP and not yet
  documented
- Command line option to download dictionaries

### Changes

- Rename helper CLI and py crates (only relevant within this project)

## [0.1.4] - 2022-08-17

### Additions

- Started generating manpages and autocomplete scripts on build
- Started generating a documentation book for the CLI

### Changes

- Better reserve & shrink vectors and hash sets to save a small ammount of
  overhead

## [0.1.3] - 2022-08-16

### Changes

- Correction to output generation

## [0.1.2] - 2022-08-16

### Additions

- Framework for locating files on a user's local machine

### Changes

- Updated binary output configuration

## [0.1.1] - 2022-07-25

### Changes

- Updated wheel release configuration

## [0.1.0] - 2022-07-25

### Changes

- Restructured project to make all modules public that might be needed to
  interface with this library.
- Restructuring to use `<Result>` for all functions that may error
- Behind the scenes work to prepare for automatic dictionary location

<!-- next-url -->
[Unreleased]: https://github.com/pluots/zspell/compare/v0.5.1...HEAD
[0.5.1]: https://github.com/pluots/zspell/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/pluots/zspell/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/pluots/zspell/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/pluots/zspell/compare/v0.3.3...v0.4.0
[0.3.3]: https://github.com/pluots/zspell/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/pluots/zspell/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/pluots/zspell/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/pluots/zspell/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/pluots/zspell/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/pluots/zspell/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/pluots/zspell/compare/v0.1.4...v0.2.0
[0.1.4]: https://github.com/pluots/zspell/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/pluots/zspell/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/pluots/zspell/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/pluots/zspell/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/pluots/zspell/compare/v0.0.1...v0.1.0
