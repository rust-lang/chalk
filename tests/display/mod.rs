use chalk_integration::{program::Program, query::LoweringDatabase, tls};
use chalk_solve::display::{self, WriterState};
use regex::Regex;
use std::{fmt::Debug, sync::Arc};

mod assoc_ty;
mod built_ins;
mod dyn_;
mod formatting;
mod impl_;
mod lifetimes;
mod opaque_ty;
mod self_;
mod struct_;
mod trait_;
mod where_clauses;

fn write_program(program: &Program) -> String {
    let mut out = String::new();
    let ws = &WriterState::new(program);
    for datum in program.struct_data.values() {
        display::write_top_level(&mut out, ws, &**datum).unwrap();
    }
    for datum in program.trait_data.values() {
        display::write_top_level(&mut out, ws, &**datum).unwrap();
    }
    for datum in program.impl_data.values() {
        display::write_top_level(&mut out, ws, &**datum).unwrap();
    }
    for datum in program.opaque_ty_data.values() {
        display::write_top_level(&mut out, ws, &**datum).unwrap();
    }
    out
}

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

/// Data from performing a reparse test which can be used to make additional
/// assertions.
///
/// Not necessary for use unless additional assertions are necessary.
#[allow(unused)]
struct ReparseTestResult<'a> {
    /// The program text for the original test code
    original_text: &'a str,
    /// The program text for the code the test says should be output
    target_text: &'a str,
    /// The actual reparsed output text
    output_text: String,
    /// Lowered version of `original_text`
    original_program: Arc<Program>,
    /// Lowered version of `target_text`
    target_program: Arc<Program>,
    /// Lowered version of `output_text`
    output_program: Arc<Program>,
}

/// Parses the input, lowers it, prints it, then re-parses and re-lowers,
/// failing if the two lowered programs don't match.
///
/// Note: the comparison here does include IDs, so input order matters. In
/// particular, ProgramWriter always writes traits, then structs, then
/// impls. So all traits must come first, then structs, then all impls, or
/// the reparse will fail.
fn reparse_test(program_text: &str) -> ReparseTestResult<'_> {
    reparse_into_different_test(program_text, program_text)
}

/// [`reparse_test`], but allows a non-convergent test program to be tested
/// a different target.
fn reparse_into_different_test<'a>(
    program_text: &'a str,
    target_text: &'a str,
) -> ReparseTestResult<'a> {
    let original_db = chalk_integration::db::ChalkDatabase::with(program_text, <_>::default());
    let original_program = original_db.program_ir().unwrap_or_else(|e| {
        panic!(
            "unable to lower test program:\n{}\nSource:\n{}\n",
            e, program_text
        )
    });
    let target_db = chalk_integration::db::ChalkDatabase::with(target_text, <_>::default());
    let target_program = target_db.program_ir().unwrap_or_else(|e| {
        panic!(
            "unable to lower test program:\n{}\nSource:\n{}\n",
            e, program_text
        )
    });
    let output_text =
        tls::set_current_program(&original_program, || write_program(&original_program));
    let output_db = chalk_integration::db::ChalkDatabase::with(&output_text, <_>::default());
    let output_program = output_db.program_ir().unwrap_or_else(|e| {
        panic!(
            "error lowering writer output:\n{}\nNew source:\n{}\n",
            e, output_text
        )
    });
    if output_program != target_program {
        panic!(
            "WriteProgram produced different program.\n\
             Diff:\n{}\n\
             Source:\n{}\n{}\
             New Source:\n{}\n",
            program_diff(&target_program, &output_program),
            program_text,
            if target_text != program_text {
                format!(
                    "Test Should Output (different from original):\n{}\n",
                    target_text
                )
            } else {
                String::new()
            },
            output_text
        );
    }
    eprintln!("\nTest Succeeded:\n\n{}\n---", output_text);
    ReparseTestResult {
        original_text: program_text,
        output_text,
        target_text,
        original_program,
        output_program,
        target_program,
    }
}

fn test_formatting(src: &str, acceptable: &str) {
    let result = reparse_test(src);
    let acceptable = Regex::new(acceptable).unwrap();
    if !acceptable.is_match(&result.output_text) {
        panic!(
            "output_text's formatting didn't match the criteria.\
            \noutput_text:\n\"{0}\"\
            \ncriteria:\n\"{1}\"\
            \ndebug output: {0:?}\
            \ndebug criteria: {2:?}\n",
            result.output_text,
            acceptable,
            acceptable.as_str()
        );
    }
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
