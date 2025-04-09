# Chalk Coherence

This document was previously prepared for the initial design of coherence rules in Chalk. It was copy-pasted here on 2020-10-06, but has not been vetted for accuracy of the current implementation or edited for clarity.

## Coherence
> The idea of trait coherence is that, given a trait and some set of types for its type parameters, there should be exactly one impl that applies. So if we think of the trait `Display`, we want to guarantee that if we have a trait reference like `MyType : Display`, we can uniquely identify a particular impl.
> 
> The role of the orphan rules in particular is basically to prevent you from implementing external traits for external types. So continuing our simple example of `Display`, if you are defining your own library, you could not implement `Display` for `Vec<T>`, because both `Display` and `Vec` are defined in the standard library. But you can implement `Display` for `MyType`, because you defined `MyType`. However, if you define your own trait `MyTrait`, then you can implement `MyTrait` for any type you like, including external types like `Vec<T>`. To this end, the orphan rule intuitively says “either the trait must be local or the self-type must be local”.
> 
> -- [Little Orphan Impls](https://smallcultfollowing.com/babysteps/blog/2015/01/14/little-orphan-impls/) by Niko Matsakis

To check for coherence, the Rust compiler completes two separate but related checks:

- orphan check - ensures that each impl abides by the orphan rules, or in other words, that an impl is potentially implementable by the crate adding it
    - A consequence of the orphan rules: for every impl that could exist, it only exists in **one** place — this is key to having a coherent system
- overlap check - ensures that no two impls overlap in your program **or** **in any** ***compatible*** **world**
    - **compatible** - any semver compatible world
# Resources About Coherence
- [Coherence - talk by withoutboats](https://www.youtube.com/watch?v=AI7SLCubTnk&t=43m19s)
- [Little Orphan Impls](https://smallcultfollowing.com/babysteps/blog/2015/01/14/little-orphan-impls/)
- [RFC 1023 Rebalancing Coherence](https://rust-lang.github.io/rfcs/1023-rebalancing-coherence.html)
- [Type classes: confluence, coherence and global uniqueness](https://web.archive.org/web/20250308110404/https://blog.ezyang.com/2014/07/type-classes-confluence-coherence-global-uniqueness/)
## Axioms & Properties of Coherence
> Historical Note: We used to use the term “external” instead of “upstream”.


- **Axiom 1:** crates upstream to you should be able to implement their own traits for their own types
- **Axiom 2:** crates downstream from you should be able to implement your traits
- **Property:** Upstream crates must assume that downstream crates will add any impls that compile. Downstream crates are allowed to assume that upstream crates will not add any semver incompatible impls.
# Chalk: Orphan Check

The purpose of the orphan check is to ensure that an impl is only definable in a single crate. This check is what makes it impossible for other crates to define impls of your traits for your types.

**We want to capture some rule:** Given `impl<T0…Tn> for Trait<P1…Pn> for P0`, `LocalImplAllowed(P0: Trait<P1…Pn>)` is true if and only if this impl is allowed in the current (local) crate. 

This check is applied to all impls in the current crate. Upstream impls are not checked with this rule.

## The Orphan Rules

In order to model the orphan check in chalk, we need a precise description of the orphan rules as they are implemented in rustc today.

There are several resources which can be used to figure out the orphan rules in rustc.

- [RFC 1023: Rebalancing Coherence](https://rust-lang.github.io/rfcs/1023-rebalancing-coherence.html)
- [*Trait Implementation Coherence*](https://doc.rust-lang.org/reference/items/implementations.html#trait-implementation-coherence) [in the](https://doc.rust-lang.org/reference/items/implementations.html#trait-implementation-coherence) [*Rust Reference*](https://doc.rust-lang.org/reference/items/implementations.html#trait-implementation-coherence)
- [E0210: A violation of the orphan rules in the](https://doc.rust-lang.org/error-index.html#E0210) [*Rust Error Index*](https://doc.rust-lang.org/error-index.html#E0210)
- [*Little Orphan Impls*](https://smallcultfollowing.com/babysteps/blog/2015/01/14/little-orphan-impls/) [by Niko Matsakis](https://smallcultfollowing.com/babysteps/blog/2015/01/14/little-orphan-impls/)

Of all of these, RFC 1023 is probably considered the most authoritative source on the orphan rules. The orphan rules as proposed in that RFC are as follows:

Given an impl `impl<T1...Tn> Trait<P1...Pn> for P0`, either `Trait` must be local to the current crate, or:

1. At least one type must meet the `LT` pattern defined above. Let `Pi` be the first such type.
2. No type parameters `T1...Tn` may appear in the type parameters that precede `Pi` (that is, `Pj` where `j < i`).

The `LT` pattern being referred to basically means that the type is a “local type” including the affects of fundamental types. That means that `Ti` is either a local type, or a fundamental type whose first parameter is a local type.

This definition is good. Once you read it a few times and it makes sense, it is fairly unambiguous. That being said, the RFC was written quite a while ago and we have since found [unsoundness](https://github.com/rust-lang/rust/issues/43355) in some of the parts of the compiler that were implemented based on that RFC.

Thus, it is probably best to look at the only *truly authoritative* source on the Rust compiler: the rustc source code itself! Indeed, if you think of the rustc source code as an executable specification of how the Rust programming language is meant to work, you can look at it and determine the true behaviour of the orphan rules.

## The Orphan Check in rustc

The orphan check as implemented today in the Rust compiler takes place in the [`orphan_check`](https://github.com/rust-lang/rust/blob/b7c6e8f1805cd8a4b0a1c1f22f17a89e9e2cea23/src/librustc/traits/coherence.rs#L236) function which is called [for every declared impl](https://github.com/rust-lang/rust/blob/b7c6e8f1805cd8a4b0a1c1f22f17a89e9e2cea23/src/librustc_typeck/coherence/orphan.rs#L45). Since implementations for locally defined traits are always defined, that function returns OK if the trait being implemented is local. Otherwise, it dispatches to the [`orphan_check_trait_ref`](https://github.com/rust-lang/rust/blob/b7c6e8f1805cd8a4b0a1c1f22f17a89e9e2cea23/src/librustc/traits/coherence.rs#L343) function which does the major orphan rules checking.

Recall that the impls we are dealing with are in the form `impl<T0…Tn> Trait<P1…Pn> for P0`.

The `orphan_check_trait_ref` function takes a **trait ref** which is essentially `Trait` and its parameters `P0…Pn` (notice that the `Self` type `P0` is included). The parameters `P0…Pn` are known as the **input types** of the trait. The function goes through each input type from `P0` to `Pn` looking for the first local type `Pi`. For each type parameter `Pj` found before that, the function checks that it does not contain any of the placeholder types `T0…Tn` at any level. That means that `Pj` cannot have any of the types `T0…Tn` at any level recursively. When the first local type `Pi` is found, we check to make sure any type parameters used in it are covered by a local type. Since we don’t have any fundamental types with more than one type parameter, this check is probably extraneous.

## The Orphan Rules in rustc

Thus, based on the source code, the orphan rules in Rust are as follows:

Given an impl of the form `impl<T0…Tn> Trait<P1…Pn> for P0`, the impl is allowed if:


- `Trait` is local to the current crate
- `Trait` is upstream to the current crate and:
    - There is at least one type parameter `Pi` which, taking fundamental types into account, is **local** to the current crate
    - Within the type `Pi`,  all type parameters are covered by `Pi`
        - This only really applies if we allowed fundamental types with multiple type parameters
        - Since we don’t do that yet, we can ignore this for the time being
    - All types `Pj` such that `j < i` do not contain `T0…Tn` at any level of depth (i.e. the types are **fully visible** **—** “visible” meaning that the type is a known type and not a type parameter or variable)
## Modeling The Orphan Check

Determining how to model these rules in chalk is actually quite straightforward at this point. We have an exact specification of how the rules are meant to work and we can translate that directly. 

Here’s how the lowering rules would look: 

For each trait `Trait`,

- If `Trait` is local to the current crate, we generate:
    `forall<Self, P1…Pn> { LocalImplAllowed(Self: Trait<P1...Pn>) }`
    This models that any impls are allowed if the trait is local to the current crate.
- If `Trait` is upstream to the current crate, we need a rule which models the additional conditions on which impls are allowed:
```ignore
forall<Self, P1...Pn> { LocalImplAllowed(Self: Trait<P1...Pn>) :- IsLocal(Self) }
forall<Self, P1...Pn> {
  LocalImplAllowed(Self: Trait<P1...Pn>) :- IsFullyVisible(Self), IsLocal(P1)
}
forall<Self, P1...Pn> {
  LocalImplAllowed(Self: Trait<P1...Pn>) :-
    IsFullyVisible(Self),
    IsFullyVisible(P1),
    IsLocal(P2)
}
forall<Self, P1...Pn> {
  LocalImplAllowed(Self: Trait<P1...Pn>) :-
    IsFullyVisible(Self),
    IsFullyVisible(P1),
    IsFullyVisible(P2),
    IsLocal(P3)
}
...
forall<Self, P1...Pn> {
  LocalImplAllowed(Self: Trait<P1...Pn>) :-
    IsFullyVisible(Self),
    IsFullyVisible(P1),
    IsFullyVisible(P2),
    ...
    IsFullyVisible(Pn-1),
    IsLocal(Pn)
}
```
Here, we have modeled every possible case of `P1` to `Pn` being local and then checked if all prior type parameters are fully visible. This truly is a direct translation of the rules listed above!

Now, to complete the orphan check, we can iterate over each impl of the same form as before and check if `LocalImplAllowed(P0: Trait<P1…Pn>)` is provable.

# Chalk: Overlap Check
> Note: A key assumption for the overlap check is that the orphan check runs before it. That means that any impl that the overlap check encounters already abides by the orphan rules. This is very important to how the check works and it wouldn’t work without the orphan check also present before it.

The purpose of the overlap check is to ensure that there is only up to one impl that can apply to a method call at a given time. In order to accomplish this, the overlap check looks at all pairs of impls and tries to ensure that there is no “overlap” between the sets of types that both impls can apply to. It accomplishes this by attempting to take the “intersection” of the constraints of both impls and then ensuring that this intersection cannot possibly apply to any types. If this turns out to be provable, the types are truly disjoint.

This is a simple application of the mathematical law:

> If two sets *A* and *B* are disjoint, then *A* ∩ *B* = ∅

More concretely, let’s say you have the following two impls: ([example from RFC 1023](https://rust-lang.github.io/rfcs/1023-rebalancing-coherence.html#type-locality-and-negative-reasoning))

```rust,ignore
impl<T: Copy> Clone for T { /* ... */ }
impl<U> Clone for MyType<U> { /* ... */ }
```

Then we’ll try to solve the following:

```ignore
not { exists<T, U> { T = MyType<U>, T: Copy } }
```

One way to read this is to say “try to prove that there is no `MyType<U>` for any `U` that implements the `Copy` trait”. The reason we’re trying to prove this is because if there is such an implementation, then the second impl would overlap with the first one. The first impl applies to any type that implements `Copy`.

The issue is that there may very well not be any such impl at this current time. In that case, chalk will conclude that these two impls do not overlap. This is an issue because that is certainly an impl that could be added later, so this conclusion may be too strong.

Why is that we’re only saying that this conclusion *may* be too strong? Well we’re using “may” because it depends on what we want to assume about different crates. The orphan rules make it so that upstream crates can add certain impls to themselves in a semver compatible way. In particular, upstream crates can add impls of upstream traits for their own upstream types without having to worry about breaking downstream code. That means that we can’t just assume that upstream type doesn’t implement an upstream trait. This particular assumption is too strong.

On the other hand, the orphan rules permit the current crate to add certain impls as well. A property of the orphan rules is that the impls it allows are only allowed to be defined in a single crate. So that means that if the impls allowed by the orphan rules in the current crate don’t exist, it is perfectly safe to assume that they are not there.

The conclusion from all of this is that it is perfectly safe to rule out impls that can be defined in the current crate, but we can’t do the same for impls in any other crate. That means that we need to come up with a way to model all possible impls in upstream, downstream and even sibling crates so we can make sure that our overlap check isn’t making assumptions that are too strong.

**Clarification:** One caveat to all of this is that we can’t simply model “all possible impls” because then the current crate wouldn’t be able to add any impls at all for upstream traits. After all, it is *possible* for an upstream crate to add *any* impl for its upstream trait. A more precise version of what we’re looking for is to model impls that an upstream crate could add in a **compatible** way. These are impls that we may not be able to current see, but also cannot ignore since that would be too strong of an assumption.

**We are specifically trying to avoid a situation where a semver compatible upgrade of a dependency breaks the current crate because the current crate was able to add an impl that only the dependency was meant to be able to add.**

**Sibling Crates:** Furthermore, we can immediately rule out sibling crates because by definition they are unable to use each other’s types or traits. If two crates are unable to interact at all, they cannot possibly add a conflicting implementation in any **coherent** world. Proof: Suppose that a sibling crate could add an impl that would conflict with a conclusion drawn by the overlap check in the current crate. Then the sibling crate would have to be able to implement a trait that was available to the current crate for a type that was available for the current crate. Since the sibling crate by definition does not have access to the current crate’s types or traits, the conflicting type and trait must be upstream. By the orphan rules, the sibling crate cannot implement a trait for upstream types and traits. Thus, the conflicting implementation in the sibling crate is impossible and no such implementation can exist.

**Downstream Crates:** Downstream crates come into play because all traits in upstream crates and in the current crate can potentially be implemented by downstream crates using the forms allowed by the orphan rules. In essence, we always need to assume that downstream crates will implement traits in all ways that compile.

## Discussion: Modeling the Overlap Check

[Aaron’s excellent blog post](https://aturon.github.io/blog/2017/04/24/negative-chalk/) talks about this exact problem from the point of view of negative reasoning. It also describes a potential solution which we will apply here to solve our problem.

The **compatible modality** (`compat` in Aaron’s blog post) is necessary because we don’t always want to assume that all compatible impls exist. In particular, there are certain phases of compilation (e.g. trans) where the closed-world assumption is entirely necessary and sufficient.

To start addressing the problem at hand, the question is: what implementations are crates other than the current crate allowed to add in a semver compatible way? 

Since we already ruled out sibling crates, this only leaves upstream crates and downstream crates. Upstream crates only have access to upstream types and traits. That means that the only impls they can add are impls for upstream types or blanket impls over type parameters. Downstream crates have access to all upstream traits and types in addition to all traits and types in the current crate.

**Claim:** No impl containing generic types can be added in a semver compatible way.
**Proof:** If the impl contains only generic types, it is considered a blanket impl and it may already be that a downstream trait implements that trait. So by adding a blanket impl, it now conflicts with the potential downstream implementation and is thus a breaking change. If the impl contains a generic type and also some number of upstream types, then a downstream crate may still have implemented that trait for all of the same values of the type parameters but with the generic types filled with downstream types. Thus, adding such an impl would also be a breaking change that would conflict with that potential downstream impl.

The only situation where an impl containing generic types can be added in a way that is **not** a breaking change is if **in addition to the impl**, a new type is also added to the upstream crate. In that case, downstream crates would not have had an opportunity to implement that trait for those types just yet. All of that being said, from the perspective of the current crate looking at potential upstream impls, this case does not matter at all because the current crate can never query for a type that doesn’t exist yet. That means that this situation doesn’t actually impact the potential impls that we need to account for even though it is a valid example of a situation where a new blanket impl is possible.

Thus, for all intents and purposes, impls containing generic type parameters cannot be added in semver compatible ways. This only leaves a single option: impls containing only upstream types. These are compatible because by the orphan rules, the current crate and any further downstream crates is not allowed to implement upstream traits for all upstream types. Thus, adding these impls cannot possibly break anything.

This significantly narrows down our set of potential impls that we need to account for to only impls of upstream traits for upstream types.

For downstream crates, we need to add rules for all possible impls that they could potentially add using any upstream traits or traits in the current crate. We can do this by enumerating the possibilities generated from the orphan rules specified above:

```ignore
// Given a trait MyTrait<P1...Pn> where WCs

forall<Self, P1...Pn> {
  Implemented(Self: MyTrait<P1...Pn>) :-
    WCs,                  // where clauses
    Compatible,
    DownstreamType(Self), // local to a downstream crate
    CannotProve,
}
forall<Self, P1...Pn> {
  Implemented(Self: MyTrait<P1...Pn>) :-
    WCs,
    Compatible,
    IsFullyVisible(Self),
    DownstreamType(P1),
    CannotProve,
}
...
forall<Self, P1...Pn> {
  Implemented(Self: MyTrait<P1...Pn>) :-
    WCs,
    Compatible,
    IsFullyVisible(Self),
    IsFullyVisible(P1),
    ...,
    IsFullyVisible(Pn-1),
    DownstreamType(Pn),
    CannotProve,
}
```

Perhaps somewhat surprisingly, `IsFullyVisible` works here too. This is because our previous definition of the lowering for `IsFullyVisible` was quite broad. By lowering *all* types in the current crate and in upstream crates with `IsFullyVisible`, that predicate covers the correct set of types here too. The orphan rules only require that there are no types parameters prior to the first local type. Types that are not type parameters and also by definition not downstream types are all of the types in the current crate and in upstream crates. This is exactly what `IsFullyVisible` covers.

Fundamental types in both the current crate and in upstream crates can be considered local in a downstream crate if they are provided with a downstream type. To model this, we can add an additional rule for fundamental types:

```ignore
forall<T> { DownstreamType(MyFundamentalType<T>) :- DownstreamType(T) }
```

**Where clauses:** Traits can have where clauses.

```rust,ignore
#[upstream] trait Foo<T, U, V> where Self: Eq<T> { /* ... */ }
```

**The question is**: do we need to bring these where clauses down into the rule that we generate for the overlap check?
**Answer:** Yes. Since the trait can only be implemented for types that satisfy its where clauses, it makes sense to also limit our assumption of compatible impls to impls that can exist.

**Associated types:** Traits can have associated types. We do not need to worry about them in our discussion because associated types are output types and trait matching is done on input types. This is also why the orphan rules do not mention associated types at all.

## Overlap Check in Chalk

Thus, based on the discussion above, the overlap check with coherence in mind can be modeled in chalk with the following:


- All disjoint queries take place inside of `compatible`

- `compatible { G }` desugars into `forall<T> { (Compatible, DownstreamType(T)) => G }`, thus introducing a `Compatible` predicate using implication

- For each upstream trait `MyTrait<P1…Pn>`, we lower it into the following rule:

  ```ignore
  forall<Self, P1...Pn> {
    Implemented(Self: MyTrait<P1...Pn>) :-
      Compatible,
      IsUpstream(Self),
      IsUpstream(P1),
      ...,
      IsUpstream(Pn),
      CannotProve
  }
  ```

  This will accomplish our goal of returning an ambiguous answer whenever the
overlap check query asks about any impls that an upstream crate may add in a
compatible way. We determined in the discussion above that these are the only
impls in any crate that can be added compatibly.

  **Note:** Trait `where` clauses are lowered into the rule’s conditions as well as a prerequisite to everything else.

- For all traits `MyTrait<P1…Pn> where WCs` in the current crate and in upstream traits,
  ```ignore
  forall<Self, P1...Pn> {
    Implemented(Self: MyTrait<P1...Pn>) :-
      WCs,                  // where clauses
      Compatible,
      DownstreamType(Self), // local to a downstream crate
      CannotProve,
  }
  forall<Self, P1...Pn> {
    Implemented(Self: MyTrait<P1...Pn>) :-
      WCs,
      Compatible,
      IsFullyVisible(Self),
      DownstreamType(P1),
      CannotProve,
  }
  ...
  forall<Self, P1...Pn> {
    Implemented(Self: MyTrait<P1...Pn>) :-
      WCs,
      Compatible,
      IsFullyVisible(Self),
      IsFullyVisible(P1),
      ...,
      IsFullyVisible(Pn-1),
      DownstreamType(Pn),
      CannotProve,
  }
  ```
  
- For fundamental types in both the current crate and in upstream crates,
  ```ignore
  forall<T> { DownstreamType(MyFundamentalType<T>) :- DownstreamType(T) }
  ```

## Alternative Designs

Initially, when Niko and I started working on this, Niko suggested the following implementation:

> For each upstream trait, `MyTrait<P1…Pn>`, we lower it into the following rule:
> ```ignore
> forall<Self, P1...Pn> {
>   Implemented(Self: MyTrait<P1...Pn>) :-
>     Compatible,
>     not { LocalImplAllowed(Self: MyTrait<P1...Pn>) },
>     CannotProve
> }
> ```

This appears to make sense because we need to assume that any impls that the current crate cannot add itself may exist somewhere else. By using `not { LocalImplAllowed(…) }`, we modeled exactly that. The problem is, that this assumption is actually too strong. What we actually need to model is that any **compatible** impls that the current crate cannot add itself may exist somewhere else. This is a **subset** of the impls covered by `not { LocalImplAllowed(…) }`.

Notes to be added somewhere:

- For impls that are definable in the current crate, we assume that the only ones that exist are the ones that are actually present. If the current crate does not define an impl that it could define, for our purposes, that impl does not exist. This is in contrast to how we treat upstream impls. For those, we assume that impls *may* exist even if we don’t *know* that they do.
- Struct/Trait privacy (e.g. `pub`) does not matter. For better or for worse, we always assume that everything is public or is going to be public someday, so we do not consider privacy at all.
- Fundamental traits - tend to be traits that you generally wouldn't implement yourself. The compiler is the one generating implementations for those traits, so it was decided that it was okay to definitively conclude whether or not an impl exists for them

