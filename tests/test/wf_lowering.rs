use chalk_integration::query::LoweringDatabase;

#[test]
fn well_formed_trait_decl() {
    lowering_success! {
        program {
            trait Clone { }
            trait Copy where Self: Clone { }

            struct Foo { }

            impl Clone for Foo { }
            impl Copy for Foo { }
        }
    }
}

#[test]
fn ill_formed_trait_decl() {
    lowering_error! {
        program {
            trait Clone { }
            trait Copy where Self: Clone { }

            struct Foo { }

            impl Copy for Foo { }
        } error_msg {
            "trait impl for `Copy` does not meet well-formedness requirements"
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
            "trait impl for `B` does not meet well-formedness requirements"
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

            struct MyType { }

            trait Bar {
                type Value;
            }

            impl Bar for MyType {
                // `OnlyFoo<MyType>` is ill-formed because `MyType: Foo` does not hold.
                type Value = OnlyFoo<MyType>;
            }
        } error_msg {
            "trait impl for `Bar` does not meet well-formedness requirements"
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
            "type declaration `MyType` does not meet well-formedness requirements"
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
            "trait impl for `Foo` does not meet well-formedness requirements"
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
fn ill_formed_type_in_header() {
    lowering_error! {
        program {
            trait Foo {
                type Value;
            }

            trait Bar { }

            // Types in where clauses are not assumed to be well-formed,
            // an explicit where clause would be needed (see below).
            impl<T> Bar for T where <T as Foo>::Value: Bar { }
        } error_msg {
            "trait impl for `Bar` does not meet well-formedness requirements"
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
            "trait impl for `Bar` does not meet well-formedness requirements"
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
            "trait impl for `Bar` does not meet well-formedness requirements"
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
            "trait impl for `Bar` does not meet well-formedness requirements"
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
            "type declaration `Foo` does not meet well-formedness requirements"
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
            "trait impl for `PointerFamily` does not meet well-formedness requirements"
        }
    }
}

#[test]
fn higher_ranked_trait_bounds() {
    lowering_error! {
        program {
            trait Foo<'a> { }
            trait Bar where forall<'a> Self: Foo<'a> { }
            struct Baz { }

            impl Bar for Baz { }
        } error_msg {
            "trait impl for `Bar` does not meet well-formedness requirements"
        }
    }

    lowering_success! {
        program {
            trait Foo<'a> { }
            trait Bar where forall<'a> Self: Foo<'a> { }
            struct Baz { }

            impl<'a> Foo<'a> for Baz { }
            impl Bar for Baz { }
        }
    }
}

#[test]
fn higher_ranked_trait_bound_on_gat() {
    lowering_success! {
        program {
            trait Foo<'a> { }
            struct Baz { }

            trait Bar<'a> {
                type Item<V>: Foo<'a> where forall<'b> V: Foo<'b>;
            }

            impl<'a> Bar<'a> for Baz {
                type Item<V> = V;
            }
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
            "trait impl for `Foo` does not meet well-formedness requirements"
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
fn higher_ranked_inline_bound_on_gat() {
    lowering_success! {
        program {
            trait Fn<T> { }
            struct Ref<'a, T> { }
            struct Val {}

            struct fun<T> { }

            impl<'a, T> Fn<Ref<'a, T>> for for<'b> fn(fun<Ref<'b, T>>) { }

            trait Bar {
                type Item<T>: forall<'a> Fn<Ref<'a, T>>;
            }

            impl Bar for Val {
                type Item<T> = for<'a> fn(fun<Ref<'a, T>>);
            }
        }
    }

    lowering_error! {
        program {
            trait Fn<T, U> { }
            struct Val {}

            struct fun<T, U> { }

            impl<T, U> Fn<T, U> for fun<T, U> { }

            trait Bar {
                type Item<T>: forall<U> Fn<T, U>;
            }

            impl Bar for Val {
                type Item<T> = fun<T, Val>;
            }
        } error_msg {
            "trait impl for `Bar` does not meet well-formedness requirements"
        }
    }
}

#[test]
fn assoc_type_recursive_bound() {
    lowering_error! {
        program {
            trait Sized { }
            trait Print {
                // fn print();
            }

            trait Foo {
                type Item: Sized where <Self as Foo>::Item: Sized;
            }

            struct Number { }

            impl Foo for Number {
                // Well-formedness checks require that the following
                // goal is true:
                // ```
                // if (str: Sized) { # if the where clauses hold
                //     str: Sized # then the bound on the associated type hold
                // }
                // ```
                // which it is :)
                type Item = str;
            }

            struct OnlySized<T> where T: Sized { }
            impl<T> Print for OnlySized<T> {
                // fn print() {
                //     println!("{}", std::mem::size_of::<T>());
                // }
            }

            trait Bar {
                type Assoc: Print;
            }

            impl<T> Bar for T where T: Foo {
                type Assoc = OnlySized<<T as Foo>::Item>;
            }

            // Above, we used to incorrectly assume that `OnlySized<<T as Foo>::Item>`
            // is well-formed because of the `FromEnv(T: Foo)`, hence making the `T: Bar`
            // impl pass the well-formedness check. But the following query will
            // (and should) always succeed, as there is no where clauses on `Assoc`:
            // ```
            // forall<T> { if (T: Bar) { WellFormed(<T as Bar>::Assoc) } }
            // ```
            //
            // This may lead to the following code to compile:

            // ```
            // fn foo<T: Print>() {
            //     T::print() // oops, in fact `T = OnlySized<str>` which is ill-formed
            // }

            // fn bar<T: Bar> {
            //     // ok, we have `FromEnv(T: Bar)` hence
            //     // `<T as Bar>::Assoc` is well-formed and
            //     // `Implemented(<T as Bar>::Assoc: Print)` hold
            //     foo<<T as Bar>::Assoc>(
            // }

            // fn main() {
            //     bar::<Number>() // ok, `Implemented(Number: Bar)` hold
            // }
            // ```
        } error_msg {
            "trait impl for `Bar` does not meet well-formedness requirements"
        }
    }
}

#[test]
fn struct_sized_constraints() {
    lowering_error! {
        program {
            #[lang(sized)]
            trait Sized { }

            struct S<T> {
                t1: T,
                t2: T
            }
        } error_msg {
            "type declaration `S` does not meet well-formedness requirements"
        }
    }

    lowering_success! {
        program {
            #[lang(sized)]
            trait Sized { }

            struct Foo { }

            struct S<T> {
                t1: Foo,
                t2: T
            }
        }
    }

    lowering_success! {
        program {
            #[lang(sized)]
            trait Sized { }

            struct S<T> where T: Sized {
                t1: T,
                t2: T
            }
        }
    }

    lowering_success! {
        program {
            #[lang(sized)]
            trait Sized { }

            struct Foo {}

            struct G<T> {
                foo: S<S<Foo>>,
                s: S<S<S<T>>>
            }

            struct S<T> {
                t1: T
            }
        }
    }

    lowering_error! {
        program {
            #[lang(sized)]
            trait Sized { }

            struct Foo {}

            impl Sized for Foo {}
        } error_msg {
            "trait impl for `Sized` does not meet well-formedness requirements"
        }
    }
}

#[test]
fn enum_sized_constraints() {
    // All fields must be sized
    lowering_error! {
        program {
            #[lang(sized)]
            trait Sized { }

            enum E<T> {
                A {
                    t1: T,
                    t2: T,
                },
                B,
                C,
            }
        } error_msg {
            "type declaration `E` does not meet well-formedness requirements"
        }
    }

    // Even the last field must be sized
    lowering_error! {
        program {
            #[lang(sized)]
            trait Sized { }

            struct Foo { }

            enum E<T> {
                A {
                    t1: Foo,
                    t2: T,
                },
                B,
                C,
            }
        } error_msg {
            "type declaration `E` does not meet well-formedness requirements"
        }
    }

    // Sized bound
    lowering_success! {
        program {
            #[lang(sized)]
            trait Sized { }

            enum S<T> where T: Sized {
                A {
                    t1: T,
                    t2: T,
                },
                B,
                C,
            }
        }
    }

    // No manual impls
    lowering_error! {
        program {
            #[lang(sized)]
            trait Sized { }

            enum Foo {}

            impl Sized for Foo {}
        } error_msg {
            "trait impl for `Sized` does not meet well-formedness requirements"
        }
    }
}

#[test]
fn copy_constraints() {
    lowering_success! {
        program {
            #[lang(copy)]
            trait Copy { }

            #[lang(drop)]
            trait Drop { }

            struct S<T1, T2> { t1: T1, t2: T2 }

            impl<T1, T2> Copy for S<T1, T2> where T1: Copy, T2: Copy { }
        }
    }

    lowering_success! {
        program {
            #[lang(copy)]
            trait Copy { }

            #[lang(drop)]
            trait Drop { }

            trait MyTrait where Self: Copy { }

            struct S<T> where T: MyTrait { t: T }

            impl<T> Copy for S<T> { }
        }
    }

    // Copy implementations for a struct with non-copy field
    lowering_error! {
        program {
            #[lang(copy)]
            trait Copy { }

            struct S<T> { t: T }

            impl<T> Copy for S<T> { }
        } error_msg {
           "trait impl for `Copy` does not meet well-formedness requirements"
        }
    }

    lowering_error! {
        program {
            #[lang(copy)]
            trait Copy { }

            struct S<T1, T2> { t1: T1, t2: T2 }

            impl<T1, T2> Copy for S<T1, T2> where T2: Copy { }
        } error_msg {
           "trait impl for `Copy` does not meet well-formedness requirements"
        }
    }

    // Copy implemenation for a Drop type
    lowering_error! {
        program {
            #[lang(copy)]
            trait Copy { }

            #[lang(drop)]
            trait Drop { }

            struct S<T> where T: Copy { t: T }

            impl<T> Copy for S<T> { }

            impl<T> Drop for S<T> { }
        } error_msg {
           "trait impl for `Copy` does not meet well-formedness requirements"
        }
    }

    // Enums

    // Copy types on enum
    lowering_success! {
        program {
            #[lang(copy)]
            trait Copy { }

            #[lang(drop)]
            trait Drop { }

            enum E<T1, T2> { Foo(T1), Bar { t2: T2 } }

            impl<T1, T2> Copy for E<T1, T2> where T1: Copy, T2: Copy { }
        }
    }

    // Types with with copy bound
    lowering_success! {
        program {
            #[lang(copy)]
            trait Copy { }

            #[lang(drop)]
            trait Drop { }

            trait MyTrait where Self: Copy { }

            enum E<T> where T: MyTrait { Foo(T) }

            impl<T> Copy for E<T> { }
        }
    }

    // Copy implementations for a adt with non-copy field
    lowering_error! {
        program {
            #[lang(copy)]
            trait Copy { }

            enum E<T> { Foo(T) }

            impl<T> Copy for E<T> { }
        } error_msg {
           "trait impl for `Copy` does not meet well-formedness requirements"
        }
    }

    // Only one copy field
    lowering_error! {
        program {
            #[lang(copy)]
            trait Copy { }

            enum E<T1, T2> { Foo(T1), Bar { t2: T2 } }

            impl<T1, T2> Copy for E<T1, T2> where T2: Copy { }
        } error_msg {
           "trait impl for `Copy` does not meet well-formedness requirements"
        }
    }

    // Copy implemenation for a Drop type
    lowering_error! {
        program {
            #[lang(copy)]
            trait Copy { }

            #[lang(drop)]
            trait Drop { }

            enum E<T> where T: Copy { Foo { t: T } }

            impl<T> Copy for E<T> { }

            impl<T> Drop for E<T> { }
        } error_msg {
           "trait impl for `Copy` does not meet well-formedness requirements"
        }
    }

    // Tests for Copy impls for builtin types
    lowering_success! {
        program {
            #[lang(copy)]
            trait Copy { }

            #[lang(drop)]
            trait Drop { }

            impl Copy for u8 {}
            impl Copy for f32 {}
            impl Copy for char {}
            impl Copy for bool {}
            impl<T> Copy for *const T {}
            impl<T> Copy for *mut T {}
            impl<'a, T> Copy for &'a T {}
            impl Copy for ! {}
        }
    }

    lowering_error! {
        program {
            #[lang(copy)]
            trait Copy { }

            impl<'a, T> Copy for &'a mut T {}
        } error_msg {
           "trait impl for `Copy` does not meet well-formedness requirements"
        }
    }

    lowering_error! {
        program {
            #[lang(copy)]
            trait Copy { }

            #[object_safe]
            trait Trait {}

            impl<'a> Copy for dyn Trait + 'a {}
        } error_msg {
           "trait impl for `Copy` does not meet well-formedness requirements"
        }
    }

    lowering_error! {
        program {
            #[lang(copy)]
            trait Copy { }

            impl Copy for fn(u32) {}
        } error_msg {
           "trait impl for `Copy` does not meet well-formedness requirements"
        }
    }

    lowering_error! {
        program {
            #[lang(copy)]
            trait Copy { }

            impl Copy for str {}
        } error_msg {
           "trait impl for `Copy` does not meet well-formedness requirements"
        }
    }

    lowering_error! {
        program {
            #[lang(copy)]
            trait Copy { }

            impl Copy for [u32; 4] {}
        } error_msg {
           "trait impl for `Copy` does not meet well-formedness requirements"
        }
    }

    lowering_error! {
        program {
            #[lang(copy)]
            trait Copy { }

            impl Copy for [u32] {}
        } error_msg {
           "trait impl for `Copy` does not meet well-formedness requirements"
        }
    }
}

#[test]
fn drop_constraints() {
    lowering_error! {
        program {
            #[lang(drop)]
            trait Drop { }

            struct Foo { }
            struct S<T> { }

            impl Drop for S<Foo> { }
        } error_msg {
           "trait impl for `Drop` does not meet well-formedness requirements"
        }
    }

    lowering_success! {
        program {
            trait Trait where Self: SuperTrait { }
            trait SuperTrait {}

            #[lang(drop)]
            trait Drop { }

            struct S<T> where T: Trait { }

            impl<T> Drop for S<T> where T: SuperTrait { }
        }
    }

    lowering_success! {
        program {
            #[lang(drop)]
            trait Drop { }

            struct S<T1, T2> { }

            impl<T1, T2> Drop for S<T2, T1> { }
        }
    }

    lowering_error! {
        program {
            trait MyTrait { }

            #[lang(drop)]
            trait Drop { }

            struct S<T>{ }

            impl<T> Drop for S<T> where T: MyTrait { }
        } error_msg {
           "trait impl for `Drop` does not meet well-formedness requirements"
        }
    }
}

#[test]
fn no_unsize_impls() {
    lowering_error! {
        program {
            #[lang(unsize)]
            trait Unsize<T> {}

            impl Unsize<u32> for u32 {}
        } error_msg {
            "trait impl for `Unsize` does not meet well-formedness requirements"
        }
    }
}

#[test]
fn ill_formed_opaque_ty() {
    lowering_error! {
        program {
            trait Foo {}
            struct Bar {}

            opaque type T: Foo = Bar;
        } error_msg {
            "opaque type declaration `T` does not meet well-formedness requirements"
        }
    }

    lowering_error! {
        program {
            trait Foo { }
            struct NotFoo { }
            struct IsFoo { }
            impl Foo for IsFoo { }
            opaque type T: Foo = NotFoo;
        } error_msg {
            "opaque type declaration `T` does not meet well-formedness requirements"
        }
    }
}

#[test]
fn coerce_unsized_pointer() {
    lowering_success! {
        program {
            #[lang(unsize)]
            trait Unsize<T> {}

            #[lang(coerce_unsized)]
            trait CoerceUnsized<T> {}

            impl<'a, T, U> CoerceUnsized<*mut U> for &'a mut T where T: Unsize<U> {}
            impl<T, U> CoerceUnsized<*mut U> for *mut T where T: Unsize<U> {}
        }
    }

    // T: Unsize<U> is not in the environment
    lowering_error! {
        program {
            #[lang(unsize)]
            trait Unsize<T> {}

            #[lang(coerce_unsized)]
            trait CoerceUnsized<T> {}

            impl<'a, T, U> CoerceUnsized<*mut U> for &'a mut T {}
        } error_msg {
            "trait impl for `CoerceUnsized` does not meet well-formedness requirements"
        }
    }

    // Test with builtin Unsize impl
    lowering_success! {
        program {
            #[lang(unsize)]
            trait Unsize<T> {}

            #[lang(coerce_unsized)]
            trait CoerceUnsized<T> {}

            #[object_safe]
            trait Foo {}

            #[auto]
            #[object_safe]
            trait Auto {}

            impl<'a> CoerceUnsized<&'a (dyn Foo + 'a)> for &'a (dyn Foo + Auto + 'a) {}
        }
    }

    // Test with builtin Unsize impl
    lowering_error! {
        program {
            #[lang(unsize)]
            trait Unsize<T> {}

            #[lang(coerce_unsized)]
            trait CoerceUnsized<T> {}

            #[object_safe]
            trait Foo {}

            #[auto]
            #[object_safe]
            trait Auto {}

            impl<'a> CoerceUnsized<&'a (dyn Foo + Auto + 'a)> for &'a (dyn Foo + 'a) {}
        } error_msg {
            "trait impl for `CoerceUnsized` does not meet well-formedness requirements"
        }
    }

    // Test with builtin Unsize impl
    lowering_success! {
        program {
            #[lang(unsize)]
            trait Unsize<T> {}

            #[lang(coerce_unsized)]
            trait CoerceUnsized<T> {}

            impl<'a> CoerceUnsized<&'a [f32]> for &'a [f32; 3] {}
        }
    }

    // Coercing from shared to mut
    lowering_error! {
        program {
            #[lang(unsize)]
            trait Unsize<T> {}

            #[lang(coerce_unsized)]
            trait CoerceUnsized<T> {}

            impl<'a, T, U> CoerceUnsized<*mut U> for &'a T where T: Unsize<U> {}
        } error_msg {
            "trait impl for `CoerceUnsized` does not meet well-formedness requirements"
        }
    }

    // Coercing from shared to mut
    lowering_error! {
        program {
            #[lang(unsize)]
            trait Unsize<T> {}

            #[lang(coerce_unsized)]
            trait CoerceUnsized<T> {}

            impl<'a, T, U> CoerceUnsized<&'a mut U> for &'a T where T: Unsize<U> {}
        } error_msg {
            "trait impl for `CoerceUnsized` does not meet well-formedness requirements"
        }
    }

    // Coercing from shared to mut
    lowering_error! {
        program {
            #[lang(unsize)]
            trait Unsize<T> {}

            #[lang(coerce_unsized)]
            trait CoerceUnsized<T> {}

            impl<T, U> CoerceUnsized<*mut U> for *const T where T: Unsize<U> {}
        } error_msg {
            "trait impl for `CoerceUnsized` does not meet well-formedness requirements"
        }
    }

    // Coercing from raw pointer to ref
    lowering_error! {
        program {
            #[lang(unsize)]
            trait Unsize<T> {}

            #[lang(coerce_unsized)]
            trait CoerceUnsized<T> {}

            impl<'a, T, U> CoerceUnsized<&'a U> for *const T where T: Unsize<U> {}
        } error_msg {
            "trait impl for `CoerceUnsized` does not meet well-formedness requirements"
        }
    }
}

#[test]
fn coerce_unsized_struct() {
    lowering_success! {
        program {
            #[lang(unsize)]
            trait Unsize<T> {}

            #[lang(coerce_unsized)]
            trait CoerceUnsized<T> {}

            struct Foo<'a, T> where T: 'a {
                t: &'a T
            }

            struct Bar<T, U> {
                extra: T,
                ptr: *mut U,
            }

            impl<'a, T, U> CoerceUnsized<&'a U> for &'a T where T: Unsize<U> {}
            impl<T, U> CoerceUnsized<*mut U> for *mut T where T: Unsize<U> {}
            impl<'a> CoerceUnsized<Foo<'a, [u32]>> for Foo<'a, [u32; 3]> {}
            impl<T, U, V> CoerceUnsized<Bar<T, V>> for Bar<T, U> where U: Unsize<V> {}
        }
    }

    // Unsizing different structs
    lowering_error! {
        program {
            #[lang(unsize)]
            trait Unsize<T> {}

            #[lang(coerce_unsized)]
            trait CoerceUnsized<T> {}

            struct S1<T> {
                t: T,
            }

            struct S2<T> {
                t: T,
            }

            impl<T, U> CoerceUnsized<S2<U>> for S1<T> where T: CoerceUnsized<U> {}
        } error_msg {
            "trait impl for `CoerceUnsized` does not meet well-formedness requirements"
        }
    }

    // Unsizing enums
    lowering_error! {
        program {
            #[lang(unsize)]
            trait Unsize<T> {}

            #[lang(coerce_unsized)]
            trait CoerceUnsized<T> {}

            enum Foo<T> {
                A {
                    t: T
                }
            }

            impl<T, U> CoerceUnsized<Foo<U>> for Foo<T> where T: CoerceUnsized<U> {}
        } error_msg {
            "trait impl for `CoerceUnsized` does not meet well-formedness requirements"
        }
    }

    // Unsizing two fields
    lowering_error! {
        program {
            #[lang(unsize)]
            trait Unsize<T> {}

            #[lang(coerce_unsized)]
            trait CoerceUnsized<T> {}

            struct Bar<T, U> {
                ptr1: *mut T,
                ptr2: *mut U,
            }

            impl<T, U> CoerceUnsized<*mut U> for *mut T where T: Unsize<U> {}
            impl<T, S, U, V> CoerceUnsized<Bar<T, V>> for Bar<S, U> where U: Unsize<V>, T: Unsize<S> {}
        } error_msg {
            "trait impl for `CoerceUnsized` does not meet well-formedness requirements"
        }
    }

    // Unsizing no fields
    lowering_error! {
        program {
            #[lang(unsize)]
            trait Unsize<T> {}

            #[lang(coerce_unsized)]
            trait CoerceUnsized<T> {}

            struct Bar<T, U> {
                ptr1: *mut T,
                ptr2: *mut U,
            }

            impl<T, U> CoerceUnsized<*mut U> for *mut T where T: Unsize<U> {}
            impl<T> CoerceUnsized<Bar<T, T>> for Bar<T, T> where T: Unsize<T> {}
        } error_msg {
            "trait impl for `CoerceUnsized` does not meet well-formedness requirements"
        }
    }

    // No unsize in the environment
    lowering_error! {
        program {
            #[lang(unsize)]
            trait Unsize<T> {}

            #[lang(coerce_unsized)]
            trait CoerceUnsized<T> {}

            struct Bar<T, U> {
                extra: T,
                ptr: *mut U,
            }

            impl<T, U> CoerceUnsized<*mut U> for *mut T where T: Unsize<U> {}
            impl<T, U, V> CoerceUnsized<Bar<T, V>> for Bar<T, U> {}
        } error_msg {
            "trait impl for `CoerceUnsized` does not meet well-formedness requirements"
        }
    }

    // Phantom data test & CoerceUnsized in the environment test
    lowering_success! {
        program {
            #[lang(unsize)]
            trait Unsize<T> {}

            #[lang(coerce_unsized)]
            trait CoerceUnsized<T> {}

            #[phantom_data]
            struct PhantomData<T> {}

            struct Foo<T, V> {
                coerce: T,
                phantom: PhantomData<V>,
            }

            struct Bar<T, U, V> {
                extra: T,
                phantom: PhantomData<V>,
                ptr: *mut U,
            }

            impl<T, U> CoerceUnsized<*mut U> for *mut T where T: Unsize<U> {}
            impl<T, U, V, N, M> CoerceUnsized<Bar<T, V, N>> for Bar<T, U, M> where U: Unsize<V> {}
            impl<T, U, V> CoerceUnsized<Foo<U, V>> for Foo<T, V> where T: CoerceUnsized<U> {}
        }
    }
}
