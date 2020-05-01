# Well known traits

Not all traits can be encoded in Rust's type system: special traits
like `Sized`, `Drop` or `Unsize` need additional compiler support in order to
function properly. To address this, chalk introduces a notion of `WellKnownTrait`s:
a subset of rustc's trait lang items that need special handling in trait system logic.

As an example, consider the following two aspects of `Sized` logic:
  1) In order to prove that a struct implements `Sized`, we need to prove 
     that the last field of that struct is `Sized`.  
  2) Structs need all of their fields, except, maybe, the last one to be `Sized`.
    
Neither of those aspects are expressable in Rust, so chalk generates 
special clauses used to encode them. These examples illustrate two main 
places that deal with well known traits: 
1) [`chalk-solve\clauses\builtin_traits`][builtin_traits_mod], which generates 
   requirements for proving that a given type implements a well known trait.
2) [well-formedness](wf.md) checks, some of which need to know about well known traits.

[builtin_traits_mod]: https://github.com/rust-lang/chalk/blob/master/chalk-solve/src/clauses/builtin_traits.rs

# Auto traits

Auto traits are another kind of well known traits.
The idea is that the type implements an auto trait if all data owned by that type implements it,
with an ability to opt-out via special syntax:
```rust,ignore
impl !AutoTrait for Foo {}
```
Common examples of auto trais are `Send` and `Sync`. Since this semantic is not expressable with 
"regular" impls, it needs special support in chalk too.

# Current state 
| Type            | Copy | Clone | Sized | Unsize | Drop | Fn  | Unpin  | Generator | auto traits |
| ---             | ---  | ---   | ---   | ---    | ---  | --- | ---    |  ---      |  ---        |
| tuple types     | ✅    | ✅    | ✅     | ✅     | 🗿    | 🗿  |  🗿      |  🗿       |   ❌         |
| structs         | 🗿    | 🗿    |  ✅    | ✅     | 🗿    | 🗿  |  🗿      |  🗿       |   ✅         |
| scalar types    | 📚    | 📚    | ✅     | 🗿     | 🗿   |  🗿  |  🗿     |  🗿       |    ❌        |
| trait objects   | 🗿    | 🗿    | 🗿     |  ✅    | 🗿    | 🗿   | 🗿      |  🗿       |    🗿        |
| functions       | ✅    | ✅    | ✅     | 🗿     | 🗿    | ❌   | 🗿      |  🗿       |    ❌         |
| arrays❌         | ❌     | ❌    | ❌     | ❌      | 🗿   | 🗿   | 🗿      |  🗿       |    ❌        |
| slices❌         | ❌     | ❌    | 🗿     | ❌      | 🗿   | 🗿   | 🗿      |  🗿       |    ❌       |
| closures❌       | ❌     | ❌    | ❌     | 🗿      | 🗿   | ❌   | 🗿      |  🗿       |    ❌        |
| generators❌     |  🗿    |  🗿  | ❌     |  🗿     | 🗿    | 🗿  | ❌      |   ❌       |    ❌       |
| gen. witness❌   |  🗿    |   🗿  |  🗿   |   🗿    |  🗿   |  🗿 |  🗿    |   🗿       |    ❌       |
| -----------     |       |      |       |        |      |     |        |           |             |
| well-formedness |  ✅   |  🗿   | ✅     | 🗿     | ✅    |  🗿  | 🗿      |  🗿       |   🗿         |

legend:  
🗿 - not applicable  
✅ - implemented  
📚 - implementation provided in libcore  
❌ - not implemented  
❌ in the column type means that type is not yet in chalk

The list of types not yet in chalk is not full, but auto traits/`WellKnownTrait`s
implementation for them is fairly trivial, so it is not listed here.