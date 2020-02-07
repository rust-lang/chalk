use chalk_solve::{ display::{
    WriterState,
    RenderAsRust
} };

use crate::program::Program;

pub trait WriteProgram {
    fn write(&self) -> String;
}

impl WriteProgram for Program {
    fn write(&self) -> String {
        let mut lines = vec![];
        let ws = WriterState::new(self);
        self.struct_data.values().for_each(|datum| {
            lines.push(datum.display(ws).to_string());
        });
        self.trait_data.values().for_each(|datum| {
            lines.push(datum.display(ws).to_string());
        });
        self.impl_data.values().for_each(|datum| {
            lines.push(datum.display(ws).to_string());
        });
        lines.join("\n")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lowering::LowerProgram;
    use chalk_ir::tls;
    use std::sync::Arc;

    #[test]
    fn test_program_writer() {
        let original_program_text = "
            struct Foo { }
            struct Vec<T> { }
            struct Map<_0, _1> { }
            struct Ref<'a, T> { }

            trait Marker { }
            impl<T> Marker for Vec<T> { }

            trait Clone { }
            impl Clone for Foo { }
            impl<T> Clone for Vec<T> where T: Clone { }
            impl<T, U> Clone for Map<T, U> where T: Clone, U: Clone { }

            trait Deref<'a, U> {
                type Assoc: Clone;
            }
            impl<'a, T, U> Deref<'a, T> for Ref<'a, U> {
                type Assoc = Foo;
            }
            trait AssocWithGenerics {
                type Assoc<T>;
            }
            impl AssocWithGenerics for Foo {
                type Assoc<T> = Vec<T>;
            }
            trait AssocTrait3<T> {
                type Assoc<U>;
            }
            impl<T> AssocTrait3<T> for Vec<T> {
                type Assoc<U> = Map<T, U>;
            }
            trait AsRef<T> { }
            trait AssocTraitWithWhere<T> {
                type Assoc<U> where U: AsRef<T>;
            }
            impl<T> AssocTraitWithWhere<T> for Vec<T> {
                type Assoc<U> = Map<T, U>;
            }
        ";

        //trait A { }
        //struct B<T> { }
        //struct C<T> { }
        //impl<T> A for B<C<T>> { }

        let p = Arc::new(
            chalk_parse::parse_program(&original_program_text)
                .unwrap()
                .lower()
                .unwrap(),
        );
        tls::set_current_program(&p, || {
            let written = p.write();
            eprintln!("{}", written);
        })
    }

    #[test]
    fn complicated_bounds() {
        let original_program_text = "
            struct Foo { }
            trait Bar { }
            trait Baz<T> { }
            trait Bax<T> { type BaxT; }
            trait Test {
                type Assoc<T>: Bar + Baz<Foo> + Bax<T, BaxT=T>
                    where
                        Foo: Bax<T, BaxT=T>,
                        Foo: Bar,
                        dyn Bar: Baz<Foo>;
            }
        ";

        let p = Arc::new(
            chalk_parse::parse_program(&original_program_text)
                .unwrap()
                .lower()
                .unwrap(),
        );
        tls::set_current_program(&p, || {
            let written = p.write();
            eprintln!("complicated_bounds:\n{}", written);
        })
    }
    #[test]
    fn function_type() {
        let original_program_text = "
            struct Foo { }
            trait Baz<T> { }
            impl Baz<fn(Foo) -> Foo> for Foo { }
        ";

        let p = Arc::new(
            chalk_parse::parse_program(&original_program_text)
                .unwrap()
                .lower()
                .unwrap(),
        );
        tls::set_current_program(&p, || {
            let written = p.write();
            eprintln!("complicated_bounds:\n{}", written);
        })
    }
    #[test]
    fn where_clauses_galore() {
        let original_program_text = "
            struct Foo<T, U> where T: Baz, U: Bez { }
            trait Baz { }
            trait Bez { }
        ";

        let p = Arc::new(
            chalk_parse::parse_program(&original_program_text)
                .unwrap()
                .lower()
                .unwrap(),
        );
        tls::set_current_program(&p, || {
            let written = p.write();
            eprintln!("complicated_bounds:\n{}", written);
        })
    }
    #[test]
    fn use_as_clause() {
        let original_program_text = "
            struct Foo<T, U> where <U as Bez<T>>::Assoc<dyn Baz>: Baz { }
            trait Baz { }
            trait Bez<T> {
                type Assoc<U>;
            }
        ";

        let p = Arc::new(
            chalk_parse::parse_program(&original_program_text)
                .unwrap()
                .lower()
                .unwrap(),
        );
        tls::set_current_program(&p, || {
            let written = p.write();
            eprintln!("complicated_bounds:\n{}", written);
        })
    }
    #[test]
    fn placeholder_in_different_situations() {
        let original_program_text = "
            trait Baz<'a> {}
            trait Bax<'a> {}
            struct Foo<'b> where forall<'a> Foo<'a>: Baz<'a> { }
            impl<'a> Baz<'a> for for<'b> fn(Foo<'b>) { }
            impl<'a> Bax<'a> for fn(Foo<'a>) { }
        ";
        /*struct Foo where forall<'a> U: Baz<'a> { }
            trait Baz<'a> { }
            trait Bez<T> {
                type Assoc<U: forall<Z> Bez<Z>>;
            }*/

        let p = Arc::new(
            chalk_parse::parse_program(&original_program_text)
                .unwrap()
                .lower()
                .unwrap(),
        );
        tls::set_current_program(&p, || {
            let written = p.write();
            eprintln!("complicated_bounds:\n{}", written);
        })
    }
    #[test]
    fn lifetimes_in_structs() {
        let original_program_text = "
            trait Baz<'a> {}
            struct Foo<'b> { }
            impl<'a> Baz<'a> for Foo<'a> { }
        ";
        /*struct Foo where forall<'a> U: Baz<'a> { }
            trait Baz<'a> { }
            trait Bez<T> {
                type Assoc<U: forall<Z> Bez<Z>>;
            }*/

        let p = Arc::new(
            chalk_parse::parse_program(&original_program_text)
                .unwrap()
                .lower()
                .unwrap(),
        );
        tls::set_current_program(&p, || {
            let written = p.write();
            eprintln!("complicated_bounds:\n{}", written);
        })
    }
}
