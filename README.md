[![Build Status](https://travis-ci.com/rust-lang/chalk.svg?branch=master)](https://travis-ci.com/rust-lang/chalk)

# chalk

A [Prolog-ish][Prolog] interpreter written in Rust, intended perhaps for use in
the compiler, but also for experimentation.

## FAQ

**How does chalk relate to rustc?** The plan is to have rustc use the `chalk-engine` crate (in this repo), which defines chalk's solver. The rest of chalk can then be considered an elaborate unit testing harness. For more details, see [the Traits chapter of the rustc-guide](https://rust-lang.github.io/rustc-guide/traits/index.html).

**Where does the name come from?** `chalk` is named after [Chalkidiki], the area where [Aristotle] was
born. Since Prolog is a logic programming language, this seemed a
suitable reference.

[Prolog]: https://en.wikipedia.org/wiki/Prolog
[Chalkidiki]: https://en.wikipedia.org/wiki/Chalkidiki
[Aristotle]: https://en.wikipedia.org/wiki/Aristotle

## Blog posts
[blog-posts]: #blog-posts
Here are some blog posts talking about chalk:

- [Lowering Rust Traits to Logic](http://smallcultfollowing.com/babysteps/blog/2017/01/26/lowering-rust-traits-to-logic/)
    - Explains the basic concepts at play
- [Unification in Chalk, Part 1](http://smallcultfollowing.com/babysteps/blog/2017/03/25/unification-in-chalk-part-1/)
    - An introduction to unification
- [Unification in Chalk, Part 2](http://smallcultfollowing.com/babysteps/blog/2017/04/23/unification-in-chalk-part-2/)
    - Extending the system for associated types
- [Negative reasoning in Chalk](http://aturon.github.io/blog/2017/04/24/negative-chalk/)
    - How to prove that something is not true
- [Query structure in chalk](http://smallcultfollowing.com/babysteps/blog/2017/05/25/query-structure-in-chalk/)
    - The basic chalk query structure, with pointers into the chalk implementation
- [Cyclic queries in chalk](http://smallcultfollowing.com/babysteps/blog/2017/09/12/tabling-handling-cyclic-queries-in-chalk/)
    - Handling cyclic relations and enabling the implementation of implied bounds and other long-desired features in an elegant way

## REPL

There is a repl mainly for debugging purposes which can be run by `cargo run`. Some basic examples are in [libstd.chalk](libstd.chalk).

## Contributing

If you're like to contribute, consider joining the [Traits Working Group](https://github.com/rust-lang/rust/issues/48416). We hang out on the [rust-lang discord](http://discord.gg/rust-lang) in the `#wg-traits` channel (Lang category).
