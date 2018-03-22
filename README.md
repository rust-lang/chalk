[![Join the chat at https://gitter.im/chalk-rs/Lobby](https://badges.gitter.im/chalk-rs/Lobby.svg)](https://gitter.im/rust-lang/WG-traits?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)
[![Build Status](https://travis-ci.org/rust-lang-nursery/rayon.svg?branch=master)](https://travis-ci.org/rust-lang-nursery/chalk)

# chalk

A [Prolog-ish][Prolog] interpreter written in Rust, intended perhaps for use in
the compiler, but also for experimentation.

## Origin of the name

`chalk` is named after [Chalkidiki], the area where [Aristotle] was
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

## Contributing
If you are interested in contributing to chalk, please take a look at the
[Contribution guide](CONTRIBUTING.md). You might also want to take a look at the
[glossary](GLOSSARY.md).
