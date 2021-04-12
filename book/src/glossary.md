# Glossary and terminology

This is a glossary of terminology (possibly) used in the chalk crate.

## Notation

### Basic notation

| Notation     | Meaning                                 |
|--------------|-----------------------------------------|
| `?0`         | [Type inference variable]               |
| `^0`, `^1.0` | [Bound variable]; bound in a [`forall`] |
| `!0`, `!1.0` | [Placeholder]                           |
| `A :- B`     | [Clause]; A is true if B is true        |

### Rules

- `forall<T> { (Vec<T>: Clone) :- (T: Clone)`: for every `T`, `Vec<T>`
  implements `Clone` if `T` implements `Clone`

### Queries

- `Vec<i32>: Clone`: does `Vec<i32>` implement `Clone`?
- `exists<T> { Vec<T>: Clone }`: does there exist a `T` such that `Vec<T>`
  implements `Clone`?

[Type inference variable]: ./types/rust_types.md#inference-variables
[Bound variable]: ./types/rust_types.md#bound-variables
[`forall`]: #debruijn-index
[Placeholder]: ./types/rust_types.md#placeholders
[Clause]: ./clauses/goals_and_clauses.md

## Binary connective
There are sixteen logical connectives on two boolean variables. The most
interesting in this context are listed below. There is also a truth table given
which encodes the possible results of the operations like this

```notrust
f(false, false) f(false, true) f(true, false) f(true, true).
```

As a shorthand the resulting truth table is encoded with `true = 1` and `false =
0`.

| Truth table | Operator symbol | Common name                      |
|-------------|-----------------|----------------------------------|
| 0001        | &&              | Conjunction; and                 |
| 1001        | <=>             | Equivalence; if and only if; iff |
| 1101        | =>              | Implication; if ... then         |

## Binder
A binder is an expression that binds a literal to a certain expression.
Examples for binders:

- The universal quantifier `forall(a)` states that a certain condition holds for
  all allowed values for `a`.
- A function definition `f(x) = a * x` is a binder for the variable `x` whereas
  `a` is a free variable.
- A sum `\sum_n x_n` binds the index variable `n`.

## Canonical Form
A formula in canonical form has the property that its De Bruijn indices are
minimized. For example when the formula `forall<0, 1> { 0: A && 1: B }` is
processed, both "branches" `0: A` and `1: B` are processed individually. The
first branch would be in canonical form, the second branch not since the
occurring De Bruijn index `1` could be replaced with `0`.

## Clause
A clause is the disjunction of several expressions. For example the clause
`condition_1 || condition_2 || ...` states that at least one of the conditions
holds.

There are two notable special cases of clauses. A *Horn clause* has at most one
positive literal. A *Definite clause* has exactly one positive literal.

*Horn clauses* can be written in the form `A || !B || !C || ...` with `A` being
the optional positive literal. Due to the equivalence `(P => Q) <=> (!P || Q)`
the clause can be expressed as `B && C && ... => A` which means that A is true
if `B`, `C`, etc. are all true. All rules in chalk are in this form. For example

```rust,ignore
struct A<T> {}
impl<T> B for A<T> where T: C + D {}
```

is expressed as the *Horn clause* `(T: C) && (T: D) => (A<T>: B)`. This formula
has to hold for all values of `T`. The second example

```rust,ignore
struct A {}
impl B for A {}
impl C for A {}
```

is expressed as the *Horn clause* `(A: B) && (A: C)`. Note the missing
consequence.

## De Bruijn Index
De Bruijn indices numerate literals that are bound in an unambiguous way. The
literal is given the number of its binder. The indices start at zero from the
innermost binder increasing from the inside out.

Given the example `forall<T> { exists<U> { T: Foo<Item=U> } }` the
literal names `U` and `T` are replaced with `0` and `1` respectively and the names are erased from the binders: `forall<_>
{ exists<_> { 1: Foo<Item=0> } }`.

As another example, in `forall<X, Y> { forall <Z> { X } }`, `X` is represented
as `^1.0`. The `1` represents the De Bruijn index of the variable and the `0`
represents the index in that scope: `X` is bound in the second scope counting
from where it is referenced, and it is the first variable bound in that scope.

## Formula
A formula is a logical expression consisting of literals and constants connected
by logical operators.

## Goal
With a set of type variables, given types, traits and impls, a goal specifies a
problem which is solved by finding types for the type variables that satisfy the
formula. For example the goal `exists<T> { T: u32 }` can be solved with `T =
u32`.

## Literal
A literal is an atomic element of a formula together with the constants `true`
and `false`. It is equivalent to a variable in an algebraic expressions. Note
that literals are *not* the same as the type variables used in specifying a
goal.

## Normal form
To say that a statement is in a certain *normal form* means that the pattern in
which the subformulas are arranged fulfill certain rules. The individual patterns
have different advantages for their manipulation.

### Conjunctive normal form (CNF)
A formula in CNF is a conjunction of disjunctions. For example `(x1 || x2 ||
x3) && (x4 || x5 || x6)` is in CNF.

### Disjunctive normal form (DNF)
A formula in DNF is a disjunction of conjunctions. For example `(x1 && x2 &&
x3) || (x4 && x5 && x6)` is in DNF.

### Negation normal form (NNF)
A formula in NNF consists only of literals, the connectives `&&` and `||` and
`true` and `false`.

### Prenex normal form (PNF)
All quantifiers are on the highest level of a formula and do not occur inside
the subformulas of the expression.

- `forall(x). exists(y). forall(z). P(x) && P(y) => P(z)` is in PNF.
- `(exists(x). P(x)) => exists(y). P(y) && forall(z). P(z)` is *not* in PNF.

## Normalization
Normalization is the process of converting an associated type to a concrete
type. In the case of an iterator this would mean that the associated `Item` type
is replaced with something more meaningful with respect to the individual
context (e.g. `u32`).

## Projection
Projection is the reference to a field or (in the context of Rust) to a type
from another type.

## Satisfiability
A formula is satisfiable iff there is a valuation for the atoms inside the
formula that makes it true.

## Unification
Unification is the process of solving a formula. That means unification finds
values for all the free literals of the formula that satisfy it. In the context
of chalk the values refer to types.

## Universe
A universe sets the scope in which a particular variable name is bound. (See
*Binder*.) A universe can encapsulate other universes. A universe can
be contained by only one parent universe. Universes have therefore a tree-like
structure. A universe can access the variable names of itself and the parent
universes but not of the sibling universes.

## Well-formed
A formula is well-formed if it is constructed according to a predefined set of
syntactic rules.

In the context of the Rust type system this means that basic rules for type
construction have to be met. Two examples: 1) Given a struct definition

```rust,ignore
struct HashSet<T: Hash>
```
then a type `HashSet<i32>` is well-formed since `i32` implements `Hash`. A type
`HashSet<NoHash>` with a type `NoHash` that does not implement the `Hash` trait
is not well-formed.

2) If a trait demands by its definition the implementation of further traits
for a certain type then these secondary traits have to be implemented as well.
If a type `Foo` implements `trait Eq: PartialEq` then this type has to implement
`trait PartialEq` as well. If it does not, then the type `Foo: Eq` is not well
formed according to Rust type building rules.

## Quantifier

### Existential quantifier
A formula with the existential quantifier `exists(x). P(x)` is satisfiable if
and only if there exists at least one value for all possible values of x which
satisfies the subformula `P(x)`.

In the context of chalk, the existential quantifier usually demands the
existence of exactly one instance (i.e. type) that satisfies the formula (i.e.
type constraints). More than one instance means that the result is ambiguous.

### Universal quantifier
A formula with the universal quantifier `forall(x). P(x)` is satisfiable
if and only if the subformula `P(x)` is true for all possible values for x.

### Helpful equivalences
- `not(forall(x). P(x)) <=> exists(x). not(P(x))`
- `not(exists(x). P(x)) <=> forall(x). not(P(x))`

## Skolemization
Skolemization is a technique of transferring a logical formula with existential
quantifiers to a statement without them. The resulting statement is in general
not equivalent to the original statement but equisatisfiable.

## Validity
An argument (*premise* therefore *conclusion*) is valid iff there is no
valuation which makes the premise true and the conclusion false.

Valid: `A && B therefore A || B`. Invalid: `A || B therefore A && B` because the
valuation `A = true, B = false` makes the premise true and the conclusion false.

## Valuation
A valuation is an assignment of values to all variables inside a logical
formula.

## Fixed-Points
A fixed-point of a function `f` is a value `x` for which `f(x)=x`.
Similarly a pre-fixed-point is defined as `x ≤ f(x)`, whereas for a post-fixed-point it holds that `f(x) ≤ x`.

A least fixed-point (lfp) of `f` is the fixed-point `x` of `f` for which all other fixed-points `y` are greater or equal (i.e. if `f(y)=y` then `x ≤ y`).
Similarly, a greatest fixed-point (gfp) is greater or equal than all other fixed-points.
If `f` is a function on sets, the least fixed-point is defined as the intersection of all pre-fixed-points, which are then defined as sets `x` for which `x ⊆ f(x)`.
The greatest fixed-point is in this case the union of all post-fixed-points, respectively.

This simple definition of lfp and gfp can also be lifted to general lattices.
The results for Chalk goals form such a lattice and, thus, every solver for such goals tries to find such fixed-points.