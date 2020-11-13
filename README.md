[![Build Status](https://github.com/rust-lang/chalk/workflows/CI/badge.svg)](https://github.com/rust-lang/chalk/actions?workflow=CI)
[![Chalk Book](https://img.shields.io/badge/book-chalk-blue.svg)](https://rust-lang.github.io/chalk/book/)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://rust-lang.github.io/chalk/chalk/)

# chalk

Chalk is a library that implements the Rust trait system, based on [Prolog-ish][Prolog] logic rules.

See the [Chalk book](https://rust-lang.github.io/chalk/book/) for more information.

## FAQ

**How does chalk relate to rustc?** The plan is to have rustc use the
`chalk-solve` crate (in this repo) to answer questions about Rust programs, for
example, "Does `Vec<u32>` implement `Debug`?". Internally, chalk converts
Rust-specific information into logic and uses a logic engine to find the answer
to the original query. For more details, see
[this explanation in the chalk book][chalk-lowering-details].

**Where does the name come from?** `chalk` is named after [Chalkidiki], the area where [Aristotle] was
born. Since Prolog is a logic programming language, this seemed a
suitable reference.

[Prolog]: https://en.wikipedia.org/wiki/Prolog
[Chalkidiki]: https://en.wikipedia.org/wiki/Chalkidiki
[Aristotle]: https://en.wikipedia.org/wiki/Aristotle
[chalk-lowering-details]: https://rust-lang.github.io/chalk/book/#chalk-works-by-converting-rust-goals-into-logical-inference-rules

## Blog posts
[blog-posts]: #blog-posts
Here are some blog posts talking about chalk:

- [Lowering Rust Traits to Logic](https://smallcultfollowing.com/babysteps/blog/2017/01/26/lowering-rust-traits-to-logic/)
    - Explains the basic concepts at play
- [Unification in Chalk, Part 1](https://smallcultfollowing.com/babysteps/blog/2017/03/25/unification-in-chalk-part-1/)
    - An introduction to unification
- [Unification in Chalk, Part 2](https://smallcultfollowing.com/babysteps/blog/2017/04/23/unification-in-chalk-part-2/)
    - Extending the system for associated types
- [Negative reasoning in Chalk](https://aturon.github.io/blog/2017/04/24/negative-chalk/)
    - How to prove that something is not true
- [Query structure in chalk](https://smallcultfollowing.com/babysteps/blog/2017/05/25/query-structure-in-chalk/)
    - The basic chalk query structure, with pointers into the chalk implementation
- [Cyclic queries in chalk](https://smallcultfollowing.com/babysteps/blog/2017/09/12/tabling-handling-cyclic-queries-in-chalk/)
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
[contributing]: https://rust-lang.github.io/chalk/book/contribution_guide.html
