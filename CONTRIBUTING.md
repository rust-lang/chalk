# Contributing to chalk

Thank you for your interest in contributing to chalk! There are many ways to
contribute, and we appreciate all of them.

* [Bug Reports](#bug-reports)
* [Running and Debugging](#running-and-debugging)
* [Pull Requests](#pull-requests)
* [Writing Documentation](#writing-documentation)
* [Helpful Links and Information](#helpful-links-and-information)

If you'd like to contribute, consider joining the [Traits Working Group][traits-working-group].
We hang out on the [rust-lang zulip][rust-lang-zulip] in the [#wg-traits][wg-traits-stream] stream.

As a reminder, all contributors are expected to follow our [Code of Conduct][coc].

[traits-working-group]: https://rust-lang.github.io/compiler-team/working-groups/traits/
[rust-lang-zulip]:https://rust-lang.zulipchat.com
[wg-traits-stream]: https://rust-lang.zulipchat.com/#narrow/stream/144729-wg-traits
[coc]: https://www.rust-lang.org/conduct.html

## Bug Reports
[bug-reports]: #bug-reports

While bugs are unfortunate, they're a reality in software. We can't fix what we
don't know about, so please report liberally. If you're not sure if something
is a bug or not, feel free to file a bug anyway.

If you have the chance, before reporting a bug, please search existing issues,
as it's possible that someone else has already reported your error. This doesn't
always work, and sometimes it's hard to know what to search for, so consider
this extra credit. We won't mind if you accidentally file a duplicate report.

Sometimes, a backtrace is helpful, and so including that is nice. To get
a backtrace, set the `RUST_BACKTRACE` environment variable to a value
other than `0`. The easiest way to do this is to invoke `chalk` like this:

```bash
$ RUST_BACKTRACE=1 chalk ...
```

## Running and Debugging
[running-and-debugging]: #running-and-debugging
There is a repl mainly for debugging purposes which can be run by `cargo run`. Some basic examples are in [libstd.chalk](libstd.chalk):
```bash
$ cargo run
?- load libstd.chalk
?- Vec<Box<i32>>: Clone
Unique; substitution [], lifetime constraints []
```

More logging can be enabled by setting the `CHALK_DEBUG` environment variable. Set `CHALK_DEBUG=1` to see `info!(...)` output, and `CHALK_DEBUG=2` to see `debug!(...)` output as well.

## Pull Requests
[pull-requests]: #pull-requests

Pull requests are the primary mechanism we use to change Rust. GitHub itself
has some [great documentation][pull-request-documentation] on using the Pull Request feature.
We use the "fork and pull" model [described here][development-models], where
contributors push changes to their personal fork and create pull requests to
bring those changes into the source repository.

Please make pull requests against the `master` branch.

[pull-request-documentation]: https://help.github.com/articles/about-pull-requests/
[development-models]: https://help.github.com/articles/about-collaborative-development-models/

## Writing Documentation
[writing-documentation]: #writing-documentation

Documentation improvements are very welcome. Documentation pull requests
function in the same way as other pull requests.

You can find documentation style guidelines in [RFC 1574][rfc1574].

[rfc1574]: https://github.com/rust-lang/rfcs/blob/master/text/1574-more-api-documentation-conventions.md#appendix-a-full-conventions-text

## Helpful Links and Information
[Helpful Links and Information]: #helpful-links-and-information

### Blog posts
There are several [blog posts][blog-posts] which describe the ideas and
machinery inside of chalk.

[blog-posts]: README.md#blog-posts

### Glossary

In addition to the blog posts there is a [glossary](GLOSSARY.md) explaining some
of the terminology used in chalk.

### Trait solving in rustc-dev-guide
The rustc-dev-guide describes [new-style trait solving][trait-solving], which is slowly replacing the old trait resolution.

[trait-solving]: https://rustc-dev-guide.rust-lang.org/traits/chalk.html
