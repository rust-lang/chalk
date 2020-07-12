#[test]
fn test_trait_impl_assoc_type() {
    // Test printing a single associated type - simplest case.
    reparse_test!(
        program {
            struct Foo { }
            struct Floo { }
            trait Bar {
                type Assoc;
            }
            impl Bar for Foo {
                type Assoc = Floo;
            }
        }
    );
}
#[test]
fn test_trait_with_multiple_assoc_types() {
    // Test multiple associated types per trait
    reparse_test!(
        program {
            struct Foo { }
            struct Floo { }
            trait Bax {
                type Assoc1;
                type Assoc2;
            }
            impl Bax for Foo {
                type Assoc1 = Floo;
                type Assoc2 = Foo;
            }
        }
    );
}

// The four "test_impl_assoc_type_with_generics_*" tests test various
// combinations of generics within associated types in impl blocks in order to
// flush out debrujin index errors (in other words, errors where we name
// generics incorrectly in the output)

#[test]
fn test_impl_assoc_type_with_generics_using_impl_generics() {
    // Test associated type value using generics introduced in impl block.
    reparse_test!(
        program {
            struct Foo { }
            struct Floo<T> { }
            trait Baz<T> {
                type Assoc;
            }
            impl<T> Baz<T> for Foo {
                type Assoc = Floo<T>;
            }
        }
    );
}

#[test]
fn test_impl_assoc_type_with_generics_using_gat_generics() {
    // Test associated type value using generics introduced in associated type.
    reparse_test!(
        program {
            struct Foo { }
            struct Floo<T> { }
            trait Bur {
                type Assoc<A>;
            }
            impl Bur for Foo {
                type Assoc<A> = Floo<A>;
            }
        }
    );
}

#[test]
fn test_impl_assoc_type_with_generics_using_gat_generics_and_impl_block() {
    // Test using both impl block generics and gat generics to ensure we give
    // the first generic introduced in each scope a different name.
    reparse_test!(
        program {
            struct Foo { }
            struct Floo<T, U> { }
            trait Bun<T> {
                type Assoc<A>;
            }
            impl<T, U> Bun<T> for Foo {
                type Assoc<A> = Floo<T, A>;
            }
        }
    );
}

#[test]
fn test_impl_assoc_type_with_generics_multiple_gat_generics_dont_conflict() {
    // Grab bag test using different combinations of impl block and associated
    // type generics in various places - try to flush out bugs the above 3 tests
    // don't catch.
    reparse_test!(
        program {
            struct Foo { }
            struct Floo<T, U> { }
            trait Bun<T, U> {
                type Assoc1<A, N>;
                type Assoc2<B>;
                type Assoc3<C, D>;
            }
            impl<T, U> Bun<T, U> for Foo {
                type Assoc1<A, N> = Floo<N, T>;
                type Assoc2<B> = Floo<U, B>;
                type Assoc3<C, D> = Floo<Floo<T, D>, Floo<U, C>>;
            }
        }
    );
}

#[test]
fn test_simple_assoc_type() {
    // Test we can print a trait with an associated type.
    reparse_test!(
        program {
            trait Foo {
                type Assoc;
            }
        }
    );
}

#[test]
fn test_assoc_type_bounds() {
    // Test we can print associated type bounds.
    reparse_test!(
        program {
            trait Byz {}
            trait Buzz {}
            trait Foo {
                type Assoc: Byz + Buzz;
            }
        }
    );
}

#[test]
fn test_simple_generic_assoc_type() {
    // Test we can render a generic associated type.
    reparse_test!(
        program {
            trait Trait {}
            trait Foo {
                type Assoc<Y>;
            }
        }
    );
}

#[test]
fn test_simple_generic_assoc_type_with_bounds() {
    // Test we render GATs with bounds correctly.
    reparse_test!(
        program {
            trait Trait {}
            trait Foo {
                type Assoc<Y>: Trait;
            }
        }
    );
}

#[test]
fn test_simple_generic_assoc_type_with_where_clause() {
    // Test that generic vars in associated type introduced by an associated
    // render correctly in that associated type's where clause.
    reparse_test!(
        program {
            trait Trait {}
            trait Foo {
                type Assoc<Y> where Y: Trait;
            }
        }
    );
}

#[test]
fn test_assoc_type_in_generic_trait() {
    // Test traits with both generics and associated types render correctly.
    reparse_test!(
        program {
            trait Foo<T> {
                type Assoc;
            }
        }
    );
}

#[test]
fn test_assoc_type_in_trait_with_multiple_generics() {
    // Test traits with multiple generic parameters and an associated type
    // render correctly.
    reparse_test!(
        program {
            trait Fou<T, U, F> {
                type Assoc;
            }
        }
    );
}

#[test]
fn test_assoc_type_where_clause_referencing_trait_generics() {
    // Test generics introduced in trait blocks render correctly when referenced
    // inside an associated type where clause. (looking for debrujin index errors)
    reparse_test!(
        program {
            trait Bax {}
            trait Foo<T> {
                type Assoc where T: Bax;
            }
        }
    );
}

#[test]
fn test_assoc_type_and_trait_generics_coexist() {
    // Test that we give associated type generics and trait generics different
    // names. (looking for debrujin index errors)
    reparse_test!(
        program {
            trait Bix<T> {}
            trait Foo<T> {
                type Assoc<Y> where Y: Bix<T>;
            }
        }
    );
}

#[test]
fn test_impl_assoc_ty() {
    // Test we can print associated type values in impl blocks.
    reparse_test!(
        program {
            struct Fuu {}
            trait Bhz {
                type Assoc;
            }
            impl Bhz for Fuu {
                type Assoc = Fuu;
            }
        }
    );
}

#[test]
fn test_impl_assoc_ty_in_generic_block() {
    // Test we can print associated type values in generic impl blocks.
    reparse_test!(
        program {
            struct Fou {}
            trait Bax<T> {
                type Assoc;
            }
            impl<T> Bax<T> for Fou {
                type Assoc = Fou;
            }
        }
    );
}

#[test]
fn test_impl_assoc_ty_value_referencing_block_generic() {
    // Test we can print generics introduced in impl blocks inside associated
    // type values.
    reparse_test!(
        program {
            struct Fuu {}
            trait Bmx<T> {
                type Assoc;
            }
            impl<T> Bmx<T> for Fuu {
                type Assoc = T;
            }
        }
    );
}

#[test]
fn test_impl_assoc_ty_value_referencing_block_generic_nested() {
    // Test we can print generics introduced in impl blocks inside bigger
    // expressions in an associated type value.
    reparse_test!(
        program {
            struct Fuu {}
            struct Guu<T> {}
            trait Bmx<T> {
                type Assoc;
            }
            impl<T> Bmx<T> for Fuu {
                type Assoc = Guu<T>;
            }
        }
    );
}

#[test]
fn test_impl_generics_and_assoc_ty_generics_coexist() {
    // Test we can print both generics introduced in an impl block and for an
    // associated type in the same ty expression, and they aren't conflated with
    // the same name. (looking for debrujin index errors)
    reparse_test!(
        program {
            struct Fuu {}
            struct Guu<T, U> {}
            trait Bmx<T> {
                type Assoc<U>;
            }
            impl<T> Bmx<T> for Fuu {
                type Assoc<U> = Guu<T, U>;
            }
        }
    );
}

#[test]
fn test_impl_assoc_ty_alias() {
    // Test printing `AliasTy` associated type bounds. In other words, test
    // bounds which have `Assoc=Value` print correctly on associated types.
    reparse_test!(
        program {
            struct Fow {}
            struct Qac {}
            trait Bow<T> {}
            trait Baq<T> {
                type Assoc<G>: Boo<G, Item=Fow>;
            }
            trait Boo<T> {
                type Item;
            }
            impl<T> Boo<T> for Qac {
                type Item = Fow;
            }
            impl<T> Baq<T> for Fow {
                type Assoc<U> = Qac;
            }
        }
    );
}

// The "alias_ty" tests all use a separate `produces` clause as
// `Foo: Bax<BaxT=T>` bounds are lowered into two bounds, `Bax` and
// `Bax<BaxT=T>`, and the formatter does not coalesce those bounds.

#[test]
fn test_alias_ty_bound_in_assoc_ty_where_clauses() {
    // Test bounds which have `Assoc=Value` print correctly in associated type
    // where clauses.
    reparse_test!(
        program {
            struct Foo { }
            trait Bax { type BaxT; }
            trait Test {
                type Assoc<T>
                    where
                        Foo: Bax<BaxT=T>;
            }
        }
        produces {
            struct Foo { }
            trait Bax { type BaxT; }
            trait Test {
                type Assoc<T>
                    where
                        Foo: Bax<BaxT=T>,
                        Foo: Bax;
            }
        }
    );
}

#[test]
fn test_alias_ty_bound_in_struct_where_clauses() {
    // Test bounds which have `Assoc=Value` print correctly in struct where
    // clauses.
    reparse_test!(
        program {
            struct Foo<T> where T: Baux<Assoc=T> { }
            trait Baux { type Assoc; }
        }
        produces {
            struct Foo<T> where T: Baux<Assoc=T>, T: Baux { }
            trait Baux { type Assoc; }
        }
    );
}

#[test]
fn test_alias_ty_bound_in_impl_where_clauses() {
    // Test bounds which have `Assoc=Value` print correctly in impl where clauses.
    reparse_test!(
        program {
            struct Foo<T> {}
            trait Boz { type Assoc; }
            impl<T> Boz for Foo<T> where T: Boz<Assoc=Foo<T>> {
                type Assoc = Foo<T>;
            }
        }
        produces
        {
            struct Foo<T> {}
            trait Boz { type Assoc; }
            impl<T> Boz for Foo<T> where T: Boz<Assoc=Foo<T>>, T: Boz {
                type Assoc = Foo<T>;
            }
        }
    );
}
