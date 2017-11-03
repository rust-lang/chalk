*Note that this `Contribution.md` document is more or less a copy of the file
with the same name of the [Rust compiler](https://github.com/rust-lang/rust)
project.*

# Contributing to chalk

Thank you for your interest in contributing to chalk! There are many ways to
contribute, and we appreciate all of them.

* [Feature Requests](#feature-requests)
* [Bug Reports](#bug-reports)
* [Pull Requests](#pull-requests)
* [Writing Documentation](#writing-documentation)
* [Issue Triage](#issue-triage)
* [Helpful Links and Information](#helpful-links-and-information)

If you have questions, please join [our gitter channel](https://gitter.im/chalk-rs/Lobby).

As a reminder, all contributors are expected to follow our [Code of Conduct][coc].

[pound-rust-internals]: https://chat.mibbit.com/?server=irc.mozilla.org&channel=%23rust-internals
[internals]: https://internals.rust-lang.org
[coc]: https://www.rust-lang.org/conduct.html

## Feature Requests
[feature-requests]: #feature-requests

To request a change to the way that the Rust language works, please open an
issue in the [RFCs repository](https://github.com/rust-lang/rfcs/issues/new)
rather than this one. New features and other significant language changes
must go through the RFC process.

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

### Building
[building]: #building

Chalk has to be build with the nightly version of the rust compiler.

## Pull Requests
[pull-requests]: #pull-requests

Pull requests are the primary mechanism we use to change Rust. GitHub itself
has some [great documentation][pull-requests] on using the Pull Request feature.
We use the "fork and pull" model [described here][development-models], where
contributors push changes to their personal fork and create pull requests to
bring those changes into the source repository.

[pull-requests]: https://help.github.com/articles/about-pull-requests/
[development-models]: https://help.github.com/articles/about-collaborative-development-models/

Please make pull requests against the `master` branch.

## Writing Documentation
[writing-documentation]: #writing-documentation

Documentation improvements are very welcome. Documentation pull requests
function in the same way as other pull requests.

You can find documentation style guidelines in [RFC 1574][rfc1574].

[rfc1574]: https://github.com/rust-lang/rfcs/blob/master/text/1574-more-api-documentation-conventions.md#appendix-a-full-conventions-text

# Helpful Links and Information
[Helpful Links and Information]: #helpful-links-and-information

## Blog posts
There are several [blog posts][blog-posts] which describe the ideas and
machinery inside of chalk.

[blog-posts]: README.md#blog-posts

## Glossary

In addition to the blog posts there is a [glossary](GLOSSARY.md) explaining some
of the terminology used in chalk.
