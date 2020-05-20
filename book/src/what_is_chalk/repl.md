# REPL

There is a repl mainly for debugging purposes which can be run by `cargo run`. Some basic examples are in [libstd.chalk](https://github.com/rust-lang/chalk/blob/master/libstd.chalk):
```bash
$ cargo run
?- load libstd.chalk
?- Vec<Box<i32>>: Clone
Unique; substitution [], lifetime constraints []
```
