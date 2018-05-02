#![cfg(test)]

use test_util::*;

#[test]
fn well_formed_trait_decl() {
    lowering_success! {
        program {
            trait Clone { }
            trait Copy where Self: Clone { }

            struct i32 { }

            impl Clone for i32 { }
            impl Copy for i32 { }
        }
    }
}

#[test]
fn ill_formed_trait_decl() {
    lowering_error! {
        program {
            trait Clone { }
            trait Copy where Self: Clone { }

            struct i32 { }

            impl Copy for i32 { }
        } error_msg {
            "trait impl for \"Copy\" does not meet well-formedness requirements"
        }
    }
}
#[test]
fn cyclic_traits() {
    lowering_success! {
        program {
            trait A where Self: B { }
            trait B where Self: A { }

            impl<T> B for T { }
            impl<T> A for T { }
        }
    }

    lowering_error! {
        program {
            trait Copy { }

            trait A where Self: B, Self: Copy {}
            trait B where Self: A { }

            // This impl won't be able to prove that `T: Copy` holds.
            impl<T> B for T {}

            impl<T> A for T where T: B {}
        } error_msg {
            "trait impl for \"B\" does not meet well-formedness requirements"
        }
    }

    lowering_success! {
        program {
            trait Copy { }

            trait A where Self: B, Self: Copy {}
            trait B where Self: A { }

            impl<T> B for T where T: Copy {}
            impl<T> A for T where T: B {}
        }
    }
}

#[test]
fn cyclic_wf_requirements() {
    lowering_success! {
        program {
            trait Foo where <Self as Foo>::Value: Foo {
                type Value;
            }

            struct Unit { }
            impl Foo for Unit {
                type Value = Unit;
            }
        }
    }
}

#[test]
fn ill_formed_assoc_ty() {
    lowering_error! {
        program {
            trait Foo { }
            struct OnlyFoo<T> where T: Foo { }

            struct i32 { }

            trait Bar {
                type Value;
            }

            impl Bar for i32 {
                // `OnlyFoo<i32>` is ill-formed because `i32: Foo` does not hold.
                type Value = OnlyFoo<i32>;
            }
        } error_msg {
            "trait impl for \"Bar\" does not meet well-formedness requirements"
        }
    }
}

#[test]
fn implied_bounds() {
    lowering_success! {
        program {
            trait Eq { }
            trait Hash where Self: Eq { }

            struct Set<K> where K: Hash { }

            struct OnlyEq<T> where T: Eq { }

            trait Foo {
                type Value;
            }

            impl<K> Foo for Set<K> {
                // Here, `WF(Set<K>)` implies `K: Hash` and hence `OnlyEq<K>` is WF.
                type Value = OnlyEq<K>;
            }
        }
    }
}

#[test]
fn ill_formed_ty_decl() {
    lowering_error! {
        program {
            trait Hash { }
            struct Set<K> where K: Hash { }

            struct MyType<K> {
                value: Set<K>
            }
        } error_msg {
            "type declaration \"MyType\" does not meet well-formedness requirements"
        }
    }
}

#[test]
fn implied_bounds_on_ty_decl() {
    lowering_success! {
        program {
            trait Eq { }
            trait Hash where Self: Eq { }
            struct OnlyEq<T> where T: Eq { }

            struct MyType<K> where K: Hash {
                value: OnlyEq<K>
            }
        }
    }
}

#[test]
fn wf_requiremements_for_projection() {
    lowering_error! {
        program {
            trait Foo {
                type Value;
            }

            trait Iterator {
                type Item;
            }

            impl<T> Foo for T {
                // The projection is well-formed if `T: Iterator` holds, which cannot
                // be proved here.
                type Value = <T as Iterator>::Item;
            }
        } error_msg {
            "trait impl for \"Foo\" does not meet well-formedness requirements"
        }
    }

    lowering_success! {
        program {
            trait Foo {
                type Value;
            }

            trait Iterator {
                type Item;
            }

            impl<T> Foo for T where T: Iterator {
                type Value = <T as Iterator>::Item;
            }
        }
    }
}

#[test]
fn projection_type_in_header() {
    lowering_error! {
        program {
            trait Foo {
                type Value;
            }

            trait Bar { }

            // Projection types in an impl header are not assumed to be well-formed,
            // an explicit where clause is needed (see below).
            impl<T> Bar for <T as Foo>::Value { }
        } error_msg {
            "trait impl for \"Bar\" does not meet well-formedness requirements"
        }
    }

    lowering_success! {
        program {
            trait Foo {
                type Value;
            }

            trait Bar { }

            impl<T> Bar for <T as Foo>::Value where T: Foo { }
        }
    }
}

#[test]
fn external_items() {
    lowering_success! {
        program {
            extern trait Send { }
            extern struct Vec<T> { }
        }
    }
}

#[test]
fn higher_ranked_trait_bounds() {
    lowering_error! {
        program {
            trait Foo<'a> { }
            trait Bar where forall<'a> Self: Foo<'a> { }
            struct i32 { }

            impl Bar for i32 { }
        } error_msg {
            "trait impl for \"Bar\" does not meet well-formedness requirements"
        }
    }

    lowering_success! {
        program {
            trait Foo<'a> { }
            trait Bar where forall<'a> Self: Foo<'a> { }
            struct i32 { }

            impl<'a> Foo<'a> for i32 { }
            impl Bar for i32 { }
        }
    }
}

// See `cyclic_traits`, this is essentially the same but with higher-ranked co-inductive WF goals.
#[test]
fn higher_ranked_cyclic_requirements() {
    lowering_success! {
        program {
            trait Foo<T> where forall<U> Self: Bar<U> { }
            trait Bar<T> where forall<U> Self: Foo<T> { }

            impl<T, U> Foo<T> for U { }
            impl<T, U> Bar<T> for U { }
        }
    }

    lowering_error! {
        program {
            trait Copy { }
            trait Foo<T> where forall<U> Self: Bar<U>, Self: Copy { }
            trait Bar<T> where forall<U> Self: Foo<T> { }

            impl<T, U> Foo<T> for U { }
            impl<T, U> Bar<T> for U where U: Foo<T> { }
        } error_msg {
            "trait impl for \"Foo\" does not meet well-formedness requirements"
        }
    }

    lowering_success! {
        program {
            trait Copy { }
            trait Foo<T> where forall<U> Self: Bar<U>, Self: Copy { }
            trait Bar<T> where forall<U> Self: Foo<T> { }

            impl<T, U> Foo<T> for U where U: Copy { }
            impl<T, U> Bar<T> for U where U: Foo<T> { }
        }
    }
}

#[test]
fn deref_trait() {
    lowering_success! {
        program {
            #[lang_deref] trait Deref { type Target; }
        }
    }

    lowering_error! {
        program {
            #[lang_deref] trait Deref { }
            #[lang_deref] trait DerefDupe { }
        } error_msg {
            "Duplicate lang item `DerefTrait`"
        }
    }
}
