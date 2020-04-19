[![Build Status](https://github.com/rust-lang/chalk/workflows/CI/badge.svg)](https://github.com/rust-lang/chalk/actions?workflow=CI)
[![Chalk Book](https://img.shields.io/badge/book-chalk-blue.svg)](https://rust-lang.github.io/chalk/book/)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://rust-lang.github.io/chalk/chalk/)

# chalk

A [Prolog-ish][Prolog] interpreter written in Rust, intended perhaps for use in
the compiler, but also for experimentation.

See the [Chalk book](https://rust-lang.github.io/chalk/book/) for more information.

## FAQ

**How does chalk relate to rustc?** The plan is to have rustc use the `chalk-engine` crate (in this repo), which defines chalk's solver. The rest of chalk can then be considered an elaborate unit testing harness. For more details, see [the Traits chapter of the rustc-dev-guide](https://rustc-dev-guide.rust-lang.org/traits/index.html).

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

There is a repl mainly for debugging purposes which can be run by `cargo run`. Some basic examples are in [libstd.chalk](libstd.chalk):
```bash
$ cargo run
?- load libstd.chalk
?- Vec<Box<i32>>: Clone
Unique; substitution [], lifetime constraints []
```

## Contributing

If you'd like to contribute, consider joining the [Traits Working Group][working-group].
We hang out on the [rust-lang zulip][rust-lang-zulip] in the [#wg-traits][wg-traits-stream] stream.

See [the contributing chapter][contributing] in the chalk book for more info.

[working-group]: https://rust-lang.github.io/compiler-team/working-groups/traits/
[rust-lang-zulip]:https://rust-lang.zulipchat.com
[wg-traits-stream]: https://rust-lang.zulipchat.com/#narrow/stream/144729-wg-traits
[contributing]: https://rust-lang.github.io/chalk/book/contributing.html
