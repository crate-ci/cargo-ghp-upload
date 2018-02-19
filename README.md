# cargo-ghp-upload
[![Travis Status](https://travis-ci.org/crate-ci/example-base.svg?branch=master)](https://travis-ci.org/crate-ci/cargo-ghp-upload)
[![Gitter](https://badges.gitter.im/Join%20Chat.svg)](https://gitter.im/crate-ci/general)
[![Average time to resolve an issue](http://isitmaintained.com/badge/resolution/crate-ci/cargo-ghp-upload.svg)](http://isitmaintained.com/project/crate-ci/cargo-ghp-upload "Average time to resolve an issue")
[![Percentage of issues still open](http://isitmaintained.com/badge/open/crate-ci/cargo-ghp-upload.svg)](http://isitmaintained.com/project/crate-ci/cargo-ghp-upload "Percentage of issues still open")
[![Dependency CI Status](https://dependencyci.com/github/crate-ci/example-base/badge)](https://dependencyci.com/github/crate-ci/cargo-ghp-upload)
[![Dependency Status](https://deps.rs/repo/github/crate-ci/cargo-ghp-upload/status.svg)](https://deps.rs/repo/github/crate-ci/cargo-ghp-upload)
[![Crates.io](https://img.shields.io/crates/v/cargo-ghp-upload.svg)](https://crates.io/crates/cargo-ghp-upload)

## Usage

The main use-case for `ghp-upload` is providing built documentation for your crate on github.io.
To do that, you can minimally include the following in your Travis matrix:

```yaml
matrix:
  include:
  - env: GHP_UPLOAD_VERSION=0.3.1
    install:
    - cargo install --version $GHP_UPLOAD_VERSION cargo-ghp-upload
    script:
    - cargo doc --verbose && cargo ghp-upload -vv
```

and that will

- on builds for the `master` branch
- when the build is not triggered by a PR
- build the standard documentation
- and publish it at `https://[user].github.io/[repository]/master`.

This requires Travis to have write-access to your repository. The simplest (and reasonably secure) way to achieve this
is to create a [Persional API Access Token](https://github.com/blog/1509-personal-api-tokens) with `public_repoo` scope.
Then on Travis, [define the secure environment variable](https://docs.travis-ci.com/user/environment-variables/#Defining-Variables-in-Repository-Settings)
`GH_TOKEN` with the value being the new token.

If you want to provide more scoped access, you can use a [deploy key](https://github.com/blog/2024-read-only-deploy-keys)
for repo-specific access. If no token is provided, the script will use SSH to clone from and write to the repository.
[Travis Pro handles the deploy key automatically](https://blog.travis-ci.com/2012-07-26-travis-pro-update-deploy-keys),
and free users can use [Travis encrypt-file](https://docs.travis-ci.com/user/encrypting-files/) plus a script to move
the private key into the correct location.

This also means that `cargo ghp-upload` works locally so long as you have `ssh` set up for your account. Branch and
origin context are collected from Git instead of the CI environment. This means that `cargo ghp-upload` will work on CI
other than Travis, but you currently have to manually prevent it from running on PR builds if you don't wan't it to.

## Details

This crate currently uses `./target/ghp` as scratch space. (This may change in the future; do not rely on it.)
Messing with the directory outside of this script could break things.
This crate will not change the contents of the uploaded directory.

## Customization

```
cargo-ghp-upload 0.3.1
CAD97 <cad97@cad97.com>
Upload documentation straight to GitHub Pages, maintaining branch separation and history

USAGE:
    cargo ghp-upload [FLAGS] [OPTIONS]

FLAGS:
        --remove-index    Remove `branch/index.html` if it exists
    -h, --help            Prints help information
    -r, --publish-tags    Publish documentation for tag builds (GitHub releases)
    -V, --version         Prints version information
    -v, --verbose         Enable more verbose logging [repeatable (max 4)]

OPTIONS:
        --deploy <deploy_branch>          The branch used for GitHub Pages [default: gh-pages]
        --message <message>               Message for the git commit [default: ghp-upload script]
        --branch <publish_branch>...      Branches to publish [default: master]
        --token <token>                   GitHub Personal Access token [default: $GH_TOKEN]
        --directory <upload_directory>    The directory to publish the files from [default: ./target/doc]
```

The power of `ghp-upload` comes from further customization from the default.

`ghp-upload` _does not remove_ `[branch]/index.html` if it exists on the deploy branch but not the deployed folder.
You can use this to set up a redirect to an appropriate page:

```html
<meta http-equiv="refresh" content="0; url=my_crate/index.html">
<a href="my_crate/index.html">Redirect</a>
```

Commit the above file as an `index.html` to the `gh-pages` branch and it will stay present. (`--remove-index` opts out
of this behavior.) The same goes for anything in the root that does not conflict with the branch folders.

Use `--branch` to set branches to upload for. Note that the default is overridden if you specify any explicit branches,
so if you want to upload documentation for `master` and `next`, use `--branch master --branch next`.

Use `--directory` to change what directory is published to GitHub Pages. This means you can build whatever structure you
want within that directory to upload to GitHub Pages in your branch's folder: `cargo doc` different configurations to
subfolders of `./target/doc` or compile your mdbook to `./target/book` and upload those to GitHub Pages with history.
Or do both, combine them into one directory, and serve your guide-style and reference-style docs in one location!

## Stability

This project follows Semantic Versioning with the `0.MAJOR.PATCH` extension. (This is effectively what `cargo` uses.)
The `MAJOR` version will be incremented for major changes as defined in [Rust RFC #1105
](https://rust-lang.github.io/rfcs/1105-api-evolution.html). The `MINOR` version will be incremented for minor changes
as defined in Rust RFC #1105. All other changes will be as specified in the Semantic Versioning spec.

As this is a binary-only distribution, this stability only applies to the presence of command line arguments.
New arguments will be a minor version, and removed or changed meaning will be a major version.

Upping the required minimum version of Rust is considered a minor breaking change, and will be noted in the changelog.
`cargo-ghp-pages` will always support the current stable and at least two stable versions back.
If this sliding guarantee is not enough for your use case (such as pinned CI builds),
we recommend using a `~` version requirement or pinning the exact version.

The current minimum Rust version is 1.22.0.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

(Note that this binary transitively depends on two WTFPL-licensed libraries. See [TeXitoi/structopt#71
](https://github.com/TeXitoi/structopt/pull/71) for the existing movement to make this not an issue.)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
