# Changelog

## [Unreleased]

## [0.1.52](https://github.com/rust-lang/cmake-rs/compare/v0.1.51...v0.1.52) - 2024-11-25

### Other

- Expose cc-rs no_default_flags for hassle-free cross-compilation ([#225](https://github.com/rust-lang/cmake-rs/pull/225))
- Add a `success` job to CI
- Change `--build` to use an absolute path
- Merge pull request [#195](https://github.com/rust-lang/cmake-rs/pull/195) from meowtec/feat/improve-fail-hint
- Improve hint for cmake not installed in Linux (code 127)

## [0.1.51](https://github.com/rust-lang/cmake-rs/compare/v0.1.50...v0.1.51) - 2024-08-15

### Added

- Add JOM generator support ([#183](https://github.com/rust-lang/cmake-rs/pull/183))
- Improve visionOS support ([#209](https://github.com/rust-lang/cmake-rs/pull/209))
- Use `Generic` for bare-metal systems ([#187](https://github.com/rust-lang/cmake-rs/pull/187))

### Fixed

- Fix cross compilation on android armv7 and x86 ([#186](https://github.com/rust-lang/cmake-rs/pull/186))

