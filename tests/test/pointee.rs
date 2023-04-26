use super::*;

#[test]
fn last_field_metadata() {
    test! {
        program {
            #[lang(pointee_trait)]
            trait Pointee {
                type Metadata;
            }

            struct S {
                field1: i32,
                field2: [i32],
            }
        }

        goal {
            Normalize(<(i32, str) as Pointee>::Metadata -> usize)
        } yields {
            expect![["Unique"]]
        }

        goal {
            Normalize(<(u8, i64) as Pointee>::Metadata -> ())
        } yields {
            expect![["Unique"]]
        }

        goal {
            Normalize(<() as Pointee>::Metadata -> ())
        } yields {
            expect![["Unique"]]
        }

        goal {
            Normalize(<S as Pointee>::Metadata -> usize)
        } yields {
            expect![["Unique"]]
        }

        goal {
            Normalize(<((), S) as Pointee>::Metadata -> usize)
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn primitives() {
    test! {
        program {
            #[lang(pointee_trait)]
            trait Pointee {
                type Metadata;
            }
        }

        goal {
            Normalize(<str as Pointee>::Metadata -> usize)
        } yields {
            expect![["Unique"]]
        }

        goal {
            Normalize(<str as Pointee>::Metadata -> ())
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            Normalize(<usize as Pointee>::Metadata -> ())
        } yields {
            expect![["Unique"]]
        }

        goal {
            Normalize(<i128 as Pointee>::Metadata -> ())
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn everything_is_pointee() {
    test! {
        program {
            #[lang(pointee_trait)]
            trait Pointee {
                type Metadata;
            }
        }

        goal {
            forall<T> {
                T: Pointee
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<T> {
                Normalize(<T as Pointee>::Metadata -> usize)
            }
        } yields {
            expect![["No possible solution"]]
        }
    }
}

#[test]
fn slices() {
    test! {
        program {
            #[lang(pointee_trait)]
            trait Pointee {
                type Metadata;
            }

            struct S {}
        }

        goal {
            [S]: Pointee
        } yields {
            expect![["Unique"]]
        }

        goal {
            Normalize(<[S] as Pointee>::Metadata -> usize)
        } yields {
            expect![["Unique"]]
        }

        goal {
            Normalize(<[S] as Pointee>::Metadata -> ())
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            forall<T> {
                Normalize(<[T] as Pointee>::Metadata -> usize)
            }
        } yields {
            expect![["Unique"]]
        }
    }
}
