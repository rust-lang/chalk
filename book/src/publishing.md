# Publishing Chalk

**Note: this is mostly only useful for maintainers**

The following crates get published to crates.io:
- `chalk-derive`
- `chalk-engine`
- `chalk-ir`
- `chalk-recursive`
- `chalk-solve`

The following crates get versioned without publishing:
- `chalk-parse`
- `chalk-integration`
- `chalk` (root directory)

## Release Automation
Releases are fully automated. Once a week (Sunday at midnight UTC) a GitHub
Actions job is executed which generates the changelog, bumps crate versions, and
publishes the crates. If there have not been any changes since the last version,
the release is skipped. However, if the job is manually triggered then the
release will be published even if there are no changes.

The release pipeline is located in [`publish.yml`].

[`publish.yml`]: https://github.com/rust-lang/chalk/blob/master/.github/workflows/publish.yml

### Changelog Generation
The changelog is generated using [`auto-changelog`] and is stored in
[`RELEASES.md`]. The template used for the changelog is in
[`releases-template.hbs`].

[`auto-changelog`]: https://www.npmjs.com/package/auto-changelog
[`RELEASES.md`]: https://github.com/rust-lang/chalk/blob/master/RELEASES.md
[`releases-template.hbs`]: https://github.com/rust-lang/chalk/blob/master/releases-template.hbs
