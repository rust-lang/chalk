# Crate breakdown

Chalk is broken up into a number of crates. This chapter explains the
role of each crate. This crate structure helps to serve Chalk's two goals:

* To serve as the trait engine for compilers and tools like rustc and rust-analyzer
* To be usable as a standalone REPL and testing harness

## Crates for embedding chalk into other programs

## Crates for 

Although there are many crates, there are two main "entry-point"
crates that define the different "levels" of chalk:

* the main `chalk` crate, along with the testing crate in the `tests` directory
    * These define the REPL and 
* the `chalk-solve` crate
* `chalk-integration`: The highest level crate. This is the "testing
  harness" and is also used by the REPL.
* `chalk-solve`: The mid-level crate where 
* `chalk-engine`: This is the lowest-level crate.

being both embeddable into compilers like rustc and rust-analyzer, 

One of the major goals of chalk is to be usable between many different
contexts:

* Chalk can be embedded into compilers and tools to serve as a Rust trait solving
  engine.
* Chalk can be used via a standalone testing harness.

