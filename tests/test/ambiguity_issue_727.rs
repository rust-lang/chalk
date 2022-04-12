use super::*;

#[test]
fn issue_727_1() {
    test!(
        program {
            #[upstream] #[non_enumerable] #[lang(sized)]
            trait Sized {}

            #[non_enumerable] #[object_safe]
            trait Database {}

            #[non_enumerable]
            trait QueryGroup
            where
                Self: Sized,
            {
                type DynDb: Database + HasQueryGroup<Self>;
            }

            #[non_enumerable] #[object_safe]
            trait HasQueryGroup<G>
            where
                Self: Database,
                G: QueryGroup,
                G: Sized,
            { }

            #[non_enumerable] #[object_safe]
            trait HelloWorld
            where
                Self: HasQueryGroup<HelloWorldStorage>,
            { }

            struct HelloWorldStorage {}

            impl QueryGroup for HelloWorldStorage {
                type DynDb = dyn HelloWorld + 'static;
            }
            impl<DB> HelloWorld for DB
            where
                DB: Database,
                DB: HasQueryGroup<HelloWorldStorage>,
                DB: Sized,
            { }
        }

        goal {
            forall<T> {
                if (FromEnv(T: Database); FromEnv(T: HasQueryGroup<HelloWorldStorage>); FromEnv(T: Sized)) {
                    T: HelloWorld
                }
            }
        } yields[SolverChoice::slg_default()] { // ok
            expect![["Unique"]]
        } yields[SolverChoice::recursive_default()] { // fails: "Ambiguous; no inference guidance"
            expect![["Unique"]]
        }
    );
}

#[test]
fn issue_727_2() {
    test!(
        program {
            #[non_enumerable] #[object_safe]
            trait Database {}

            #[non_enumerable]
            trait QueryGroup
            {
                type DynDb: Database + HasQueryGroup<Self>;
            }

            #[non_enumerable] #[object_safe]
            trait HasQueryGroup<G>
            where
                Self: Database,
                G: QueryGroup,
            { }

            struct HelloWorldStorage {}

            impl QueryGroup for HelloWorldStorage {
                type DynDb = dyn HasQueryGroup<HelloWorldStorage> + 'static;
            }
        }

        goal {
            forall<T> {
                if (FromEnv(T: HasQueryGroup<HelloWorldStorage>)) {
                    T: Database
                }
            }
        } yields[SolverChoice::slg_default()] {
            expect![["Unique"]]
        } yields[SolverChoice::recursive_default()] {
            expect![[r#"Ambiguous; no inference guidance"#]] // FIXME rust-lang/chalk#727
        }
    );
}

#[test]
fn issue_727_3() {
    test!(
        program {
            #[non_enumerable]
            trait Database {}

            #[non_enumerable]
            trait HasQueryGroup<G>
            where
                Self: Database,
            { }

            struct HelloWorldStorage {}

            impl Database for HelloWorldStorage { }
        }

        goal {
            forall<T, S> {
                if (FromEnv(HelloWorldStorage: HasQueryGroup<T>); FromEnv(HelloWorldStorage: HasQueryGroup<S>)) {
                    HelloWorldStorage: Database
                }
            }
        } yields[SolverChoice::slg_default()] {
            expect![["Unique"]]
        } yields[SolverChoice::recursive_default()] {
            expect![["Unique"]]
        }
    );
}
