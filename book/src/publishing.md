# Publishing Chalk

**Note: this is mostly only useful for maintainers**

The following crates get published to crates.io:
- `chalk-macros`
- `chalk-derive`
- `chalk-engine`
- `chalk-ir`
- `chalk-rust-ir`
- `chalk-solve`

The following crates get versioned without publishing:
- `chalk-parse`
- `chalk-integration`
- `chalk` (root directory)

## Pre-publish
- Remove the `-dev` suffix from the versions in each `cargo.toml`
- Bump the dependency version for each crate
- Change the `Unreleased` section in `RELEASES.md` to the version getting published
- Create commit

## Publishing
- For each crate in the order above, run `cargo publish`
    - You will probably have to wait a couple seconds between each to let the index update

## Post-publish
- Bump the minor version in each `cargo.toml` and add a `-dev` suffix
- Bump the dependency version for each crate
- Add an `Unreleased` section in the `RELEASES.md`
- Run `cargo check`
- Tag release commit on github (e.g. `v0.10.0`)

