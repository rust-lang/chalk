use chalk_solve::display::{RenderAsRust, WriterState};

use crate::program::Program;

pub trait WriteProgram {
    fn write(&self) -> String;
}

impl WriteProgram for Program {
    fn write(&self) -> String {
        let mut lines = vec![];
        let ws = &WriterState::new(self);
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
    use std::{fmt::Debug, sync::Arc};

    fn program_diff(original: &impl Debug, produced: &impl Debug) -> String {
        use std::fmt::Write;

        let mut out = String::new();
        let original = format!("{:#?}", original);
        let produced = format!("{:#?}", produced);
        for line in diff::lines(&original, &produced) {
            match line {
                diff::Result::Left(l) => write!(out, "-{}\n", l),
                diff::Result::Both(l, _) => write!(out, " {}\n", l),
                diff::Result::Right(r) => write!(out, "+{}\n", r),
            }
            .expect("writing to string never fails");
        }
        out
    }

    fn reparse_test(program_text: &str) {
        let original_program = match chalk_parse::parse_program(program_text) {
            Ok(v) => v,
            Err(e) => panic!(
                "unable to parse test program:\n{}\nSource:\n{}\n",
                e, program_text
            ),
        };
        let original_program = Arc::new(original_program.lower().unwrap());
        let new_text = tls::set_current_program(&original_program, || original_program.write());
        let new_program = match chalk_parse::parse_program(&new_text) {
            Ok(v) => v,
            Err(e) => panic!(
                "unable to reparse writer output:\n{}\nNew source:\n{}\n",
                e, new_text
            ),
        };
        let new_program = match new_program.lower() {
            Ok(v) => v,
            Err(e) => panic!(
                "error lowering writer output:\n{}\nNew source:\n{}\n",
                e, new_text
            ),
        };
        if new_program != *original_program {
            panic!(
                "WriteProgram produced different program.\n\
                 Diff:\n{}\n\
                 Source:\n{}\n
                 New Source:\n{}\n",
                program_diff(&original_program, &new_program),
                program_text,
                new_text
            );
        }
        eprintln!("{}",new_text);
    }

    #[test]
    fn test_simple_structs_and_bounds() {
        reparse_test("struct Foo {}");
        reparse_test("struct Foo<T> {}");
        // note: the order here matters! Traits must be after structs.
        reparse_test(
            "
            struct Foo<T> where T: Trait {}
            trait Trait {}
            ",
        );
    }

    #[test]
    fn test_simple_traits_and_bounds() {
        reparse_test("trait Foo {}");
        reparse_test("trait Foo<T> {}");
        reparse_test(
            "
            trait Foo<T> where T: Trait {}
            trait Trait {}
            ",
        );
    }

    #[test]
    #[ignore]
    fn test_self_in_where() {
        reparse_test(
            "
            trait Baz<'a> {}
            trait Foo where forall<'a> Self: Baz<'a> {}
            ",
        );
    }

    #[test]
    fn test_forall_in_where() {
        reparse_test(
            "
            trait Bax<T> {}
            trait Foo where forall<T> T: Bax<T> {}
            ",
        );
        reparse_test(
            "
            trait Buz<'a> {}
            trait Foo<T> where forall<'a> T: Buz<'a> {}
            ",
        );
        reparse_test(
            "
            struct Foo where forall<T> T: Biz {}
            trait Biz {}
            ",
        );
        reparse_test(
            "
            struct Foo<T> where forall<'a> T: Bez<'a> {}
            trait Bez<'a> {}
            ",
        );
    }
    #[test]
    fn test_forall_in_dyn() {
        reparse_test(
            "
            trait Foo {}
            trait Bar<'a> {}
            impl Foo for dyn forall<'a> Bar<'a> {}
            ",
        );
        reparse_test(
            "
            struct Foo {
                field: dyn forall<'a> Baz<'a>
            }
            trait Baz<'a> {}
            ",
        );
        reparse_test(
            "
            trait Foo {}
            trait Bax<'a, 'b> {}
            impl Foo for dyn forall<'a, 'b> Bax<'a, 'b> {}
            ",
        );
        reparse_test(
            "
            struct Foo {
                field: dyn forall<'a, 'b> Bix<'a, 'b>
            }
            trait Bix<'a, 'b> {}
            ",
        );
        reparse_test(
            "
            struct Foo {
                field: dyn forall<'a> Bex<'a> + forall<'b> Byx<'b>
            }
            trait Bex<'a> {}
            trait Byx<'a> {}
            ",
        );
        reparse_test(
            "
            struct Foo {
                field: dyn forall<'a, 'b> Bux<'a, 'b> + forall<'b, 'c> Brx<'b, 'c>
            }
            trait Bux<'a, 'b> {}
            trait Brx<'a, 'b> {}
            ",
        );
        reparse_test(
            "
            struct Foo<'a> {
                field: dyn forall<'b> Bpx<'a, 'b>
            }
            trait Bpx<'a, 'b> {}
            ",
        );
    }

    #[test]
    fn test_simple_dyn() {
        reparse_test(
            "
            struct Foo {
                field: dyn Bax
            }
            trait Bax {}
            ",
        );
        reparse_test(
            "
            struct Foo<'a> {
                field: dyn Bix<'a>
            }
            trait Bix<'a> {}
            ",
        );
    }

    #[test]
    fn test_simple_assoc_type() {
        reparse_test(
            "
            trait Foo {
                type Assoc;
            }
            ",
        );
        reparse_test(
            "
            trait Byz {}
            trait Buzz {}
            trait Foo {
                type Assoc: Byz + Buzz;
            }
            ",
        );
    }

    #[test]
    fn test_simple_generic_assoc_type() {
        reparse_test(
            "
            trait Trait {}
            trait Foo {
                type Assoc<Y>;
            }
            ",
        );
        reparse_test(
            "
            trait Trait {}
            trait Foo {
                type Assoc<Y>: Trait;
            }
            ",
        );
        reparse_test(
            "
            trait Trait {}
            trait Foo {
                type Assoc<Y> where Y: Trait;
            }
            ",
        );
    }

    #[test]
    fn test_assoc_type_in_generic_trait() {
        reparse_test(
            "
            trait Foo<T> {
                type Assoc;
            }
            ",
        );
        reparse_test(
            "
            trait Fou<T, U, F> {
                type Assoc;
            }
            ",
        );
        reparse_test(
            "
            trait Bax {}
            trait Foo<T> {
                type Assoc where T: Bax;
            }
            ",
        );
        reparse_test(
            "
            trait Bix<T> {}
            trait Foo<T> {
                type Assoc<Y> where Y: Bix<T>;
            }
            ",
        );
        reparse_test(
            "
            trait Bix<_0> {}
            trait Foo<_0_1> {
                type Assoc<_1_1, _1_2> where _1_1: Bix<_1_2>;
            }
            ",
        );
    }


    #[test]
    fn test_struct_fields() {
        reparse_test(
            "
            struct Foo<T> {}
            struct Bar {}
            struct Baz {
                x: Foo<Bar>,
                b: Bar
            }
            ",
        );
    }

    #[test]
    fn test_program_writer() {
        reparse_test(
            "
            struct Foo { }
            struct Vec<T> { }
            struct Map<_0, _1> { }
            struct Ref<'a, T> { }

            trait Marker { }
            trait Clone { }
            trait Deref<'a, U> {
                type Assoc: Clone;
            }
            trait AssocWithGenerics {
                type Assoc<T>;
            }
            trait AssocTrait3<T> {
                type Assoc<U>;
            }
            trait AsRef<T> { }
            
            trait AssocTraitWithWhere<T> {
                type Assoc<U> where U: AsRef<T>;
            }

            impl<T> Marker for Vec<T> { }
            impl Clone for Foo { }
            impl<T> Clone for Vec<T> where T: Clone { }
            impl<T, U> Clone for Map<T, U> where T: Clone, U: Clone { }

            impl<'a, T, U> Deref<'a, T> for Ref<'a, U> {
                type Assoc = Foo;
            }
            impl AssocWithGenerics for Foo {
                type Assoc<T> = Vec<T>;
            }
            impl<T> AssocTrait3<T> for Vec<T> {
                type Assoc<U> = Map<T, U>;
            }
            impl<T> AssocTraitWithWhere<T> for Vec<T> {
                type Assoc<U> = Map<T, U>;
            }
            ",
        );
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
        reparse_test(
            "
            struct Foo { }
            trait Baz<T> { }
            impl Baz<fn(Foo)> for Foo { }
        ",
        );
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
        reparse_test(
            "
            struct Foo<T, U> where <U as Bez<T>>::Assoc<dyn Baz>: Baz { }
            trait Baz { }
            trait Bez<T> {
                type Assoc<U>;
            }
        ",
        );
    }
    #[test]
    fn placeholder_in_different_situations() {
        reparse_test(
            "
            struct Foo<'b> where forall<'a> Foo<'a>: Baz<'a> { }
            trait Baz<'a> {}
            trait Bax<'a> {}
            trait Biz {
                type Bex: forall<'a> Bax<'a>;
            }
            impl<'a> Baz<'a> for for<'b> fn(Foo<'b>) { }
            impl<'a> Bax<'a> for fn(Foo<'a>) { }
            impl<'a> Bax<'a> for dyn forall<'b> Baz<'b> { }
        ",
        );
    }
    #[test]
    fn lifetimes_in_structs() {
        reparse_test(
            "
            struct Foo<'b> { }
            trait Baz<'a> {}
            impl<'a> Baz<'a> for Foo<'a> { }
        ",
        );
    }
}
