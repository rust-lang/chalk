# Crates

The chalk library is composed of a number of crates.  This is partly
to help clarify its internal structure, but also to allow parts of
chalk to be embedded in other programs in a piecemeal fashion.

## Intermediate representations

Chalk contains three levels of IR:

- The **AST**, produced by . This is used purely for writing test cases
  with a Rust-like syntax. This is consumed by **lowering** code, which
  takes AST and products **Rust IR** (the next bullet point).
- The **Rust IR**. This is a "HIR-like" notation that defines the
  interesting properties of things like traits, impls, and structs.
  It is an input to the **rules** code, which produces
- The **Chalk IR**. This is most "Prolog-like" of the various IRs. It
  contains the definition of **types** as well as prolog-like concepts
  such as goals (things that must be proven true) and clauses (things
  that are assumed to be true).

## Reusable bits of functionality

There are several re-usable bits of functionality. Each of these
re-usable bits of code comes associated with a set of traits that
connect it to the broader whole. 

### The "integration" crate (`chalk`)

In the chalk project, the main `chalk` crate is the crate that
connects all the pieces together, but in a compiler like rustc (or
rust-analyzer) there would be other types that play that role. We call
this the **integration** crate.

The role of the integration crate is as follows:

- It configures the solver and stores its state in between solving goals.
- It defines "the program" -- that is, the set of all traits/impls etc.
- It gives access to information about the program, such as whether a given
  trait is coinductive, or the Rust IR for a


### The chalk-solve crate

| The `chalk-solve` crate | |
|---|--- |
| Purpose:  | to solve a given goal |
| Depends on IR:  | chalk-ir but not rust-ir   |
| Context required:  | `ChalkSolveDatabase` |

The `chalk-solve` crate exposes a key typ called `Solver`.  This is a
solver that, given a goal (expressed in chalk-ir) will solve the goal
and yield up a `Solution`. The solver caches intermediate data between
invocations, so solving the same goal twice in a row (or solving goals
with common subgoals) is faster.

The solver is configured by a type that implements the
`ChalkSolveDatabase` trait. This trait contains some callbacks that
provide needed context for the solver -- notably, the solver can ask:

- **What are the program clauses that might solve given rule?** This
  is answered by the code in the chalk-rules crate.
- **Is this trait coinductive?** This is answered by the rust-ir.

#### The chalk-engine crate

| The `chalk-engine` crate  |   |
|---|--- |
| Purpose:  | define the base solving strategy |
| IR:  | none   |
| Context required:  | `Context` trait |

For the purposes of chalk, the `chalk-engine` crate is effectively
encapsulated by `chalk-solve`.  It defines the base SLG engine. It is
written in a very generic style that knows next to nothing about Rust
itself. In particular, it does not depend on any of the Chalk IRs,
which allows it to be used by rustc (which currently doesn't use
chalk-ir). The engine can be configured via the traits defined in
`chalk_engine::context::Context`, which contain (for example)
associated types that define what a goal or clause is, as well as
functions that operate on those things.

### The chalk-rules crate

| The `chalk-rules` crate  |   |
|---|--- |
| Purpose:  | create chalk-ir goals/clauses given rust-ir |
| Depends on IR:  | chalk-ir and rust-ir   |
| Context required:  | `Context` trait |

The `chalk-rules` defines code that "lowers" rust-ir into chalk-ir,
producing both goals and clauses.

- For example, the `clauses` module defines a trait
  (`ToProgramClauses`) that is implemented for various bits of
  rust-ir.  It might (for example) lower an impl into a set of program
  clauses.
- The coherence rules are defined in the `coherence` module; these
  include code to check if an impl meets the orphan rules, and to
  check for overlap between impls.
  - These can also return information about the specialization tree
    for a given trait.
- Finally, the well-formedness rules are defined in the `wf` module.

The chalk-rules crate defines a `ChalkRulesDatabase` trait that contains
a number of callbacks that it needs. These callbacks are grouped into
two sub-traits:

- The `GoalSolver` trait, which exposes a `solve` method for solving
  goals.  This solving is ultimately done by the code in the
  `chalk-solve` crate.
- The `RustIrSource` trait, which offers a number of accessors to
  fetch rust-ir. For example, the `trait_datum` method returns the
  `TraitDatum` for a given `TraitId`.
  - Note that -- by design -- this trait does not include any
    "open-ended" methods that ask queries like "return all the impls
    in the program" or "return all structs". These sorts of open-ended
    walks are expected to be performed at an outer level (in our case,
    in the chalk crate).

## The flow

This section tries to document how the flow of information proceeds in
the main chalk testing harness. This can help give an idea how all the
parts of the system interact.

- To begin with, the integration crate is asked to solve some goal
  (via `ChalkRulesDatabase::solve`, for example).
- It will get access to its internal `Solver` (instantiating one, if
  one does not already exist) and invoke the `Solver::solve` method.
- The solver may periodically request the set of applicable program clauses
  for the main goal or some subgoal.
- The integration crate will examine the goal in question and use the code in the `chalk-rules`
  crate to instantiate program clauses.
  - This may, in the case of specialization, require recursively solving goals.
- Once the program clauses are known, the solver can continue. It may
  periodically ask the integration crate whether a given bit of IR is
  coinductive.
