//! Utilities and macros for use in display tests.
//!
//! This can't live as a submodule of `test_util.rs`, as then it would conflict
//! with `display/mod.rs` for the name `mod display` when `test_util.rs` is
//! compiled as a standalone test (rather than from `lib.rs`).
use chalk_integration::{interner::ChalkIr, program::Program, query::LoweringDatabase, tls};
use chalk_solve::{
    display::{write_items, WriterState},
    logging_db::RecordedItemId,
};
use regex::Regex;
use std::{fmt::Debug, sync::Arc};

pub fn strip_leading_trailing_braces(input: &str) -> &str {
    assert!(input.starts_with('{'));
    assert!(input.ends_with('}'));

    &input[1..input.len() - 1]
}

/// Performs a test on chalk's `display` code to render programs as `.chalk` files.
macro_rules! reparse_test {
    // Test that a program, when rendered and then reparsed, results in a
    // program identical to the input.
    (program $program:tt) => {
        crate::display::util::reparse_test(crate::display::util::strip_leading_trailing_braces(
            stringify!($program),
        ))
    };
    // Tests that a program, when rendered and then reparsed, results in a
    // second, different program. Good for cases where this process is non-convergent.
    (program $program:tt produces $diff:tt) => {
        crate::display::util::reparse_into_different_test(
            crate::display::util::strip_leading_trailing_braces(stringify!($program)),
            crate::display::util::strip_leading_trailing_braces(stringify!($diff)),
        )
    };
    // Tests that a program, when rendered, results in a string which matches the
    // given regex.
    (program $program:tt formatting matches $res:literal) => {
        crate::display::util::test_formatting(
            crate::display::util::strip_leading_trailing_braces(stringify!($program)),
            $res,
        )
    };
}

/// Retrieves all item ids from a given `Program` necessary to print the entire
/// program.
pub fn program_item_ids(program: &Program) -> impl Iterator<Item = RecordedItemId<ChalkIr>> + '_ {
    macro_rules! grab_ids {
        ($map:expr) => {
            $map.keys()
                .copied()
                .map(|id| (id.0, RecordedItemId::from(id)))
        };
    }
    let mut ids = std::iter::empty()
        .chain(grab_ids!(program.adt_data))
        .chain(grab_ids!(program.trait_data))
        .chain(grab_ids!(program.impl_data))
        .chain(grab_ids!(program.opaque_ty_data))
        .chain(grab_ids!(program.fn_def_data))
        .collect::<Vec<_>>();

    // sort by the RawIds so we maintain exact program input order (note: this
    // is here rather than in logging_db.rs as we can't in general maintain this
    // - only for `chalk_integration`'s RawIds).
    //
    // We need to maintain exact order because we abuse Program's Eq
    // implementation to check the results of our tests, and which structs have
    // which ids is part of that data.
    ids.sort_by_key(|(raw_id, _)| *raw_id);

    // then discard the RawId since the RecordedItemId has the same information,
    // and is what we actually want to consume.
    ids.into_iter().map(|(_, id)| id)
}

/// Sends all items in a `chalk_integration::Program` through `display` code and
/// returns the string representing the program.
pub fn write_program(program: &Program) -> String {
    let mut out = String::new();
    let ids = program_item_ids(program);
    write_items::<_, _, Program, _, _>(&mut out, &WriterState::new(program), ids).unwrap();
    out
}

/// Diffs two `Program`s. This diffs the verbose debug output of `Program`, so
/// that you can see exactly what parts have changed in case a test fails.
///
/// Will produces something akin to the following:
///
/// ```diff
///  Program {
/// -    adt_ids: {
/// -        Atom('Foo' type=inline): AdtId(#0),
/// -    },
/// -    adt_kinds: {
/// -        AdtId(#0): TypeKind {
/// -            sort: Struct,
/// -            name: Atom('Foo' type=inline),
/// -            binders: for[] Unit,
/// -        },
/// -    },
/// +    adt_ids: {},
/// +    adt_kinds: {},
///      fn_def_ids: {},
///      fn_def_kinds: {},
///      trait_ids: {},
///      trait_kinds: {},
/// -    adt_data: {
/// -        AdtId(#0): AdtDatum {
/// -            binders: for[] AdtDatumBound {
/// -                fields: [],
/// -                where_clauses: [],
/// -            },
/// -            id: AdtId(#0),
/// -            flags: AdtFlags {
/// -                upstream: false,
/// -                fundamental: false,
/// -            },
/// -        },
/// -    },
/// +    adt_data: {},
///      fn_def_data: {},
///      impl_data: {},
///      associated_ty_values: {},
///      opaque_ty_ids: {},
///      opaque_ty_kinds: {},
///      opaque_ty_data: {},
///      trait_data: {},
///      well_known_traits: {},
///      associated_ty_data: {},
///      custom_clauses: [],
///      object_safe_traits: {},
///  }
/// ```
fn program_diff(original: &impl Debug, produced: &impl Debug) -> String {
    use std::fmt::Write;

    let mut out = String::new();
    let original = format!("{:#?}", original);
    let produced = format!("{:#?}", produced);
    for line in diff::lines(&original, &produced) {
        match line {
            diff::Result::Left(l) => writeln!(out, "-{}", l),
            diff::Result::Both(l, _) => writeln!(out, " {}", l),
            diff::Result::Right(r) => writeln!(out, "+{}", r),
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
pub struct ReparseTestResult<'a> {
    /// The program text for the original test code
    pub original_text: &'a str,
    /// The program text for the code the test says should be output
    pub target_text: &'a str,
    /// The actual reparsed output text
    pub output_text: String,
    /// Lowered version of `original_text`
    pub original_program: Arc<Program>,
    /// Lowered version of `target_text`
    pub target_program: Arc<Program>,
    /// Lowered version of `output_text`
    pub output_program: Arc<Program>,
}

/// Parses the input, lowers it, prints it, then re-parses and re-lowers,
/// failing if the two lowered programs don't match.
pub fn reparse_test(program_text: &str) -> ReparseTestResult<'_> {
    reparse_into_different_test(program_text, program_text)
}

/// [`reparse_test`], but allows a non-convergent test program to be tested
/// against a different target.
pub fn reparse_into_different_test<'a>(
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

/// Tests that a string matches a given regex pattern, erroring out if it
/// doesn't.
///
/// This is used for exact formatting tests, for testing things like indentation.
pub fn test_formatting(src: &str, acceptable: &str) {
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
