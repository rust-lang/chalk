# Glossary
This is a glossary of terminology (possibly) used in the *chalk* crate.

## Binary connective
There are sixteen logical connectives on two boolean variables. The most
interesting in this context are listed below. There is also a truth table given
which encodes the possible results of the operations like this

```
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
For a formula in canonical form all its variables are assigned integer IDs with
increasing values starting at zero. For example `?3: A && ?4: B` is not in
canonical form but `?0: A && ?1: B` is.
(See [*DeBruijn Index*](#debrujin-index).)

## Clause
In the A clause is the disjunction of several expressions. For example the clause
`condition_1 || condition_2 || ...` states that at least one of the conditions
holds.

There are two notable special cases of clauses. A *Horn clause* has at most one
positive literal. A *Definite clause* has exactly one positive literal.

*Horn clauses* can be written in the form `A || !B || !C || ...` with `A` being
the optional positive literal. Due to the equivalence `(P => Q) <=> (!P || Q)`
the clause can be expressed as `B && C && ... => A` which means that A is true
if `B`, `C`, etc. are all true. All rules in *chalk* are in this form. For
example

```
struct A<T> {}
impl<T> B for A<T> where T: C + D {}
```

is expressed as the *Horn clause* `(T: C) && (T: D) => (A<T>: B)`. This formula
has to hold for all values of `T`. The second example

```
struct A {}
impl B for A {}
impl C for A {}
```

is expressed as the *Horn clause* `(A: B) && (A: C)`. Note the missing
consequence.

The rules in chalk are expressed as *hereditary Harrop* clauses which are an
extension to the *Horn clauses*. They allow to express *universial
quantification*

## DeBruijn Index
DeBruijn indices numerate literals that are bound in an unambiguous way. The
literal is given the number of its binder. The indices start at zero from the
innermost binder increasing from the inside out.

Given the example `forall<T> { exists<U> { T: Foo<Item=U> } }` the
literal names `U` and `T` are replaced with `0` and `1` respectively: `forall<0>
{ exists<1> { 0: Foo<Item=1> } }`.

See also [*Canonical form*](#canonical-form).

## Formula
A formula is a logical expression consisting of literals and constants connected
by logical operators.

## Goal
With a set of type variables, given types, traits and impls, a goal specifies a
problem which is solved by finding types for the type variables that satisfy the
formula. For example the goal `exists<T> { T: u32 }` can be solved with `T =
u32`.

## Iff
Iff means "If and only if", which is the same as `A <=> B`, which is the same
as `(A => B) && (B => A)`. See also *binary connective*.

## Literal
A literal is an atomic element of a formula together with the constants `true`
and `false`. It is equivalent to a variable in an algebraic expressions. Note
that literals are *not* the same as the type variables used in specifying a
goal.

## Normal form
To say that a statement is in a certain *normal form* means that the pattern in
which the subformulas are arranged fulfil certain rules. The individual patterns
have different advantages for their manipulation.

### Conjunctive normal form (CNF)
A formula in CNF is a disjunction of conjunctions. For example `(x1 || x2 ||
x3) && (x4 || x5 || x6)` is in CNF.

### Disjunctive normal form (DNF)
A formula in DNF is a conjunction of disjunctions. For example `(x1 && x2 &&
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
of *chalk* the values refer to types.

## Universe
A universe sets the scope in which a particular literal is bound. (See
*Binder*.) A universe can encapsulate other universes. A universe can
be contained by only one parent universe. Universes have therefore a tree-like
structure. A universe can access the variable names of itself and the parent
universes but not of the sibling universes.

## Well-formed
A formula is well-formed if it is constructed according to a predefined set of
syntactic rules.

In the context of the Rust type system this means that basic rules for type
construction have to be met. Two examples:

1) Given a struct definition

```rust
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

In the context of *chalk*, the existential quantifier usually demands the
existence of exactly one instance (i.e. type) that satisfies the formula (i.e.
type constraints). More than one instance means that the result is ambiguous.

### Universal quantifier
A formula with the universal quantifier `forall(x). P(x)` is satisfiable
if and only if the subformula `P(x)` is true for all possible values for x.

### Helpful equivalences
- `not(forall(x). P(x)) <=> exists(x). not(P(x))`
- `not(exists(x). P(x)) <=> forall(x). not(P(x))`

## Skolemization
Skolemization is a technique of transforming a logical formula with existential
quantifiers to a statement without them. The resulting statement is in general
not equivalent to the original statement but equisatisfiable.

The idea to accomplish this transformation by adding the type variable inside an
existential quantifier to the current universe. This new type variable has to be
unified with another type later on, otherwise the goal cannot be satisfied.

## Validity
An argument ("*premises* therefore *conclusion*") is valid iff there is no
valuation which makes the premisses true and the conclusion false.

Valid: `A && B therefore A || B`. Invalid: `A || B therefore A && B` because the
valuation `A = true, B = false` makes the premiss true and the conclusion false.

## Valuation
A valuation is an assignment of values to all variables inside a logical
formula.

# Literature
- Offline
    - "Introduction to Formal Logic", Peter Smith
    - "Handbook of Practical Logic and Automated Reasoning", John Harrison
    - "Types and Programming Languages", Benjamin C. Pierce
