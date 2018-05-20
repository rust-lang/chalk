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
            impl<T> Bar for T where <T as Foo>::Value: Bar { }
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

            impl<T> Bar for T where T: Foo, <T as Foo>::Value: Bar { }
        }
    }
}

#[test]
fn bound_in_header_from_env() {
    lowering_success! {
        program {
            trait Foo { }

            trait Bar {
                type Item: Foo;
            }

            struct Stuff<T> { }

            impl<T> Bar for Stuff<T> where T: Foo {
                // Should have FromEnv(T: Foo) here.
                type Item = T;
            }
        }
    }

    lowering_error! {
        program {
            trait Foo { }
            trait Baz { }

            trait Bar {
                type Item: Baz;
            }

            struct Stuff<T> { }

            impl<T> Bar for Stuff<T> where T: Foo {
                // No T: Baz here.
                type Item = T;
            }
        } error_msg {
            "trait impl for \"Bar\" does not meet well-formedness requirements"
        }
    }
}

#[test]
fn mixed_indices_check_projection_bounds() {
    lowering_success! {
        program {
            trait Foo<T> { }

            trait Bar<T> {
                type Item: Foo<T>;
            }

            struct Stuff<T, U> { }

            impl<T, U> Bar<T> for Stuff<T, U> where U: Foo<T> {
                type Item = U;
            }
        }
    }

    lowering_error! {
        program {
            trait Foo<T> { }
            trait Baz<T> { }

            trait Bar<T> {
                type Item: Baz<T>;
            }

            struct Stuff<T, U> { }

            impl<T, U> Bar<T> for Stuff<T, U> where U: Foo<T> {
                type Item = U;
            }
        } error_msg {
            "trait impl for \"Bar\" does not meet well-formedness requirements"
        }
    }
}

#[test]
fn mixed_indices_check_generic_projection_bounds() {
    lowering_success! {
        program {
            struct Stuff<T, U> { }

            trait Foo<T> { }

            // A type that impls Foo<T> as long as U: Foo<T>.
            struct Fooey<U, V> { }
            impl<T, U, V> Foo<T> for Fooey<U, V> where U: Foo<T> { }

            trait Bar<T> {
                type Item<V>: Foo<T> where V: Foo<T>;
            }

            impl<T, U> Bar<T> for Stuff<T, U> where U: Foo<T> {
                type Item<V> = Fooey<U, V>;
            }
        }
    }

    lowering_error! {
        program {
            struct Stuff<T, U> { }

            trait Foo<T> { }
            trait Baz<T> { }

            // A type that impls Foo<T> as long as U: Foo<T>.
            struct Fooey<U, V> { }
            impl<T, U, V> Foo<T> for Fooey<U, V> where U: Foo<T> { }

            trait Bar<T> {
                type Item<V>: Baz<T> where V: Foo<T>;
            }

            impl<T, U> Bar<T> for Stuff<T, U> where U: Foo<T> {
                type Item<V> = Fooey<U, V>;
            }
        } error_msg {
            "trait impl for \"Bar\" does not meet well-formedness requirements"
        }
    }
}

#[test]
fn generic_projection_where_clause() {
    lowering_success! {
        program {
            trait PointerFamily { type Pointer<T>; }

            struct Cow<T> { }
            struct CowFamily { }
            impl PointerFamily for CowFamily { type Pointer<T> = Cow<T>; }

            struct String { }
            struct Foo<P> where P: PointerFamily {
                bar: <P as PointerFamily>::Pointer<String>
            }
        }
    }

    lowering_error! {
        program {
            trait Copy { }
            trait PointerFamily { type Pointer<T> where T: Copy; }

            struct Cow<T> { }
            struct CowFamily { }
            impl PointerFamily for CowFamily { type Pointer<T> = Cow<T>; }

            struct String { }
            struct Foo<P> where P: PointerFamily {
                // No impl Copy for String, so this will fail.
                bar: <P as PointerFamily>::Pointer<String>
            }
        } error_msg {
            "type declaration \"Foo\" does not meet well-formedness requirements"
        }
    }
}

#[test]
fn generic_projection_bound() {
    lowering_success! {
        program {
            trait Clone { }
            trait PointerFamily { type Pointer<T>: Clone where T: Clone; }

            struct Cow<T> { }
            impl<T> Clone for Cow<T> where T: Clone { }

            struct CowFamily { }

            // impl is WF due because of:
            // - `where T: Clone` clause on PointerFamily::Pointer<T>
            // - impl<T> Clone for Cow<T> where T: Clone
            impl PointerFamily for CowFamily { type Pointer<T> = Cow<T>; }

            struct String { }
            impl Clone for String { }
            struct Foo<P> where P: PointerFamily {
                bar: <P as PointerFamily>::Pointer<String>
            }
        }
    }

    lowering_error! {
        program {
            trait Clone { }
            trait PointerFamily { type Pointer<T>: Clone where T: Clone; }

            struct Cow<T> { }
            struct CowFamily { }

            // No impl Clone for Cow<T>, so this will fail.
            impl PointerFamily for CowFamily { type Pointer<T> = Cow<T>; }
        } error_msg {
            "trait impl for \"PointerFamily\" does not meet well-formedness requirements"
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
