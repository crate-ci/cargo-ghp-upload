# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

- Change CLI syntax so that it works as a subcommand (yanked broken 0.1)

## [0.1.1]

- Removed package.include key from Cargo.toml so more things are in the cargo package
- Improvements to CI

## [0.1.0]

- Split out of [roblabla/cargo-travis](https://github.com/roblabla/cargo-travis)
  - Includes rename from `cargo-doc-upload` to `cargo-ghp-upload`
  - Call as `cargo ghp-upload [FLAGS] [OPTIONS]`
- No longer links against `cargo`
- CLI changes:
  - New repeatable `-v`/`--verbose` flag for more logging (modulo #1)
  - New `--remove-index` flag to opt-out of maintaining `index.html`
    (It will still be clobbered if an index.html exists in the uploaded folder)
  - New `--directory <upload_directory>` option to upload from directories other than `./target/doc`
- Now can infer much more context from Git if not in Travis-like environment
- Default commit message changed

  [Unreleased]: https://github.com/crate-ci/cargo-ghp-upload/compare/0.1.1...master
  [0.1.1]: https://github.com/crate-ci/cargo-ghp-upload/compare/0.1.0...0.1.1
  [0.1.0]: https://github.com/crate-ci/cargo-ghp-upload/tree/0.1.0
