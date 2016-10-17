use ast::*;
use lalrpop_intern::{intern, InternedString};
use std::iter::once;
extern crate lalrpop_util as __lalrpop_util;

mod __parse__Program {
    #![allow(non_snake_case, non_camel_case_types, unused_mut, unused_variables, unused_imports)]

    use ast::*;
    use lalrpop_intern::{intern, InternedString};
    use std::iter::once;
    extern crate lalrpop_util as __lalrpop_util;
    #[allow(dead_code)]
    pub enum __Symbol<'input> {
        Term_22_28_22(&'input str),
        Term_22_29_22(&'input str),
        Term_22_2c_22(&'input str),
        Term_22_2d_3e_22(&'input str),
        Term_22_2e_22(&'input str),
        Term_22_3a_2d_22(&'input str),
        Term_22_3b_22(&'input str),
        Term_22_3d_3e_22(&'input str),
        Term_22___22(&'input str),
        Term_22exists_22(&'input str),
        Term_22forall_22(&'input str),
        Termr_23_22_5c_27_5b_5e_5c_27_5d_2b_5c_27_22_23(&'input str),
        Termr_23_22_5b_2d_7c_21_40_23_24_25_5e_26_2a_3d_2b_2f_3a_3f_7e_3c_3e_5d_2b_22_23(&'input str),
        Termr_23_22_5bA_2dZa_2dz_5d_5bA_2dZa_2dz0_2d9___5d_2a_22_23(&'input str),
        Termr_23_22_5bA_2dZa_2dz_5d_5bA_2dZa_2dz0_2d9___5d_2a_3a_22_23(&'input str),
        Nt_28Operator_20Value_29((Operator, Value)),
        Nt_28Operator_20Value_29_2b(::std::vec::Vec<(Operator, Value)>),
        NtApplication(Application),
        NtApplications(Vec<Application>),
        NtAtom(Atom),
        NtFact(Fact),
        NtFactAnd(Fact),
        NtFactApply(Fact),
        NtFactFunc(Fact),
        NtFactOr(Fact),
        NtIdentifier(InternedString),
        NtItem(Item),
        NtItem_2b(::std::vec::Vec<Item>),
        NtOperator(Operator),
        NtOperator_3f(::std::option::Option<Operator>),
        NtOperatorValue((Operator, Value)),
        NtProgram(Program),
        NtRule(Rule),
        NtValue(Value),
        NtValue_3f(::std::option::Option<Value>),
        NtVariable(Variable),
        Nt____Program(Program),
    }
    const __ACTION: &'static [i32] = &[
        // State 0
        13, // on "(", goto 12
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        14, // on "_", goto 13
        0, // on "exists", error
        0, // on "forall", error
        15, // on r#"\'[^\']+\'"#, goto 14
        16, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 15
        17, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 16
        18, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 17
        // State 1
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -9, // on ".", reduce `Application = (Operator Value)+ => ActionFn(48);`
        -9, // on ":-", reduce `Application = (Operator Value)+ => ActionFn(48);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        16, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 15
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        18, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 17
        // State 2
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        20, // on ".", goto 19
        21, // on ":-", goto 20
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 3
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -37, // on ".", reduce `Value = Atom => ActionFn(25);`
        -37, // on ":-", reduce `Value = Atom => ActionFn(25);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -37, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = Atom => ActionFn(25);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -37, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = Atom => ActionFn(25);`
        // State 4
        22, // on "(", goto 21
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -43, // on ".", reduce `Variable = Identifier => ActionFn(30);`
        -43, // on ":-", reduce `Variable = Identifier => ActionFn(30);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -43, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Variable = Identifier => ActionFn(30);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -43, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Variable = Identifier => ActionFn(30);`
        // State 5
        -28, // on "(", reduce `Item+ = Item => ActionFn(39);`
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        -28, // on "_", reduce `Item+ = Item => ActionFn(39);`
        0, // on "exists", error
        0, // on "forall", error
        -28, // on r#"\'[^\']+\'"#, reduce `Item+ = Item => ActionFn(39);`
        -28, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Item+ = Item => ActionFn(39);`
        -28, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Item+ = Item => ActionFn(39);`
        -28, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Item+ = Item => ActionFn(39);`
        // State 6
        13, // on "(", goto 12
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        14, // on "_", goto 13
        0, // on "exists", error
        0, // on "forall", error
        15, // on r#"\'[^\']+\'"#, goto 14
        16, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 15
        17, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 16
        18, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 17
        // State 7
        13, // on "(", goto 12
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -5, // on ".", reduce `Application = Operator => ActionFn(16);`
        -5, // on ":-", reduce `Application = Operator => ActionFn(16);`
        0, // on ";", error
        0, // on "=>", error
        14, // on "_", goto 13
        0, // on "exists", error
        0, // on "forall", error
        15, // on r#"\'[^\']+\'"#, goto 14
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        26, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 25
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 8
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 9
        -27, // on "(", reduce `Item = Rule => ActionFn(3);`
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        -27, // on "_", reduce `Item = Rule => ActionFn(3);`
        0, // on "exists", error
        0, // on "forall", error
        -27, // on r#"\'[^\']+\'"#, reduce `Item = Rule => ActionFn(3);`
        -27, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Item = Rule => ActionFn(3);`
        -27, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Item = Rule => ActionFn(3);`
        -27, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Item = Rule => ActionFn(3);`
        // State 10
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -4, // on ".", reduce `Application = Value => ActionFn(15);`
        -4, // on ":-", reduce `Application = Value => ActionFn(15);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        29, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 28
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        30, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 29
        // State 11
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -38, // on ".", reduce `Value = Variable => ActionFn(26);`
        -38, // on ":-", reduce `Value = Variable => ActionFn(26);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -38, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = Variable => ActionFn(26);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -38, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = Variable => ActionFn(26);`
        // State 12
        38, // on "(", goto 37
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        39, // on "_", goto 38
        0, // on "exists", error
        0, // on "forall", error
        40, // on r#"\'[^\']+\'"#, goto 39
        41, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 40
        42, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 41
        43, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 42
        // State 13
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -40, // on ".", reduce `Value = "_" => ActionFn(28);`
        -40, // on ":-", reduce `Value = "_" => ActionFn(28);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -40, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = "_" => ActionFn(28);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -40, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = "_" => ActionFn(28);`
        // State 14
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -14, // on ".", reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        -14, // on ":-", reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -14, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -14, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        // State 15
        -31, // on "(", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -31, // on ".", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        -31, // on ":-", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on ";", error
        0, // on "=>", error
        -31, // on "_", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on "exists", error
        0, // on "forall", error
        -31, // on r#"\'[^\']+\'"#, reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        -31, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 16
        -25, // on "(", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -25, // on ".", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        -25, // on ":-", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -25, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -25, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        // State 17
        -30, // on "(", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -30, // on ".", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        -30, // on ":-", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on ";", error
        0, // on "=>", error
        -30, // on "_", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on "exists", error
        0, // on "forall", error
        -30, // on r#"\'[^\']+\'"#, reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        -30, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 18
        13, // on "(", goto 12
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -7, // on ".", reduce `Application = (Operator Value)+, Operator => ActionFn(46);`
        -7, // on ":-", reduce `Application = (Operator Value)+, Operator => ActionFn(46);`
        0, // on ";", error
        0, // on "=>", error
        14, // on "_", goto 13
        0, // on "exists", error
        0, // on "forall", error
        15, // on r#"\'[^\']+\'"#, goto 14
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        26, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 25
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 19
        -26, // on "(", reduce `Item = Application, "." => ActionFn(2);`
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        -26, // on "_", reduce `Item = Application, "." => ActionFn(2);`
        0, // on "exists", error
        0, // on "forall", error
        -26, // on r#"\'[^\']+\'"#, reduce `Item = Application, "." => ActionFn(2);`
        -26, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Item = Application, "." => ActionFn(2);`
        -26, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Item = Application, "." => ActionFn(2);`
        -26, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Item = Application, "." => ActionFn(2);`
        // State 20
        57, // on "(", goto 56
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        58, // on "_", goto 57
        59, // on "exists", goto 58
        60, // on "forall", goto 59
        61, // on r#"\'[^\']+\'"#, goto 60
        62, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 61
        63, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 62
        64, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 63
        // State 21
        73, // on "(", goto 72
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        74, // on "_", goto 73
        0, // on "exists", error
        0, // on "forall", error
        75, // on r#"\'[^\']+\'"#, goto 74
        76, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 75
        77, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 76
        78, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 77
        // State 22
        -29, // on "(", reduce `Item+ = Item+, Item => ActionFn(40);`
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        -29, // on "_", reduce `Item+ = Item+, Item => ActionFn(40);`
        0, // on "exists", error
        0, // on "forall", error
        -29, // on r#"\'[^\']+\'"#, reduce `Item+ = Item+, Item => ActionFn(40);`
        -29, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Item+ = Item+, Item => ActionFn(40);`
        -29, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Item+ = Item+, Item => ActionFn(40);`
        -29, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Item+ = Item+, Item => ActionFn(40);`
        // State 23
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -43, // on ".", reduce `Variable = Identifier => ActionFn(30);`
        -43, // on ":-", reduce `Variable = Identifier => ActionFn(30);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -43, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Variable = Identifier => ActionFn(30);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -43, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Variable = Identifier => ActionFn(30);`
        // State 24
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -2, // on ".", reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        -2, // on ":-", reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -2, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -2, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        // State 25
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -25, // on ".", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        -25, // on ":-", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -25, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -25, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        // State 26
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -8, // on ".", reduce `Application = Value, (Operator Value)+ => ActionFn(47);`
        -8, // on ":-", reduce `Application = Value, (Operator Value)+ => ActionFn(47);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        16, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 15
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        18, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 17
        // State 27
        13, // on "(", goto 12
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        14, // on "_", goto 13
        0, // on "exists", error
        0, // on "forall", error
        15, // on r#"\'[^\']+\'"#, goto 14
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        26, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 25
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 28
        -31, // on "(", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        -31, // on "_", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on "exists", error
        0, // on "forall", error
        -31, // on r#"\'[^\']+\'"#, reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        -31, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 29
        -30, // on "(", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        -30, // on "_", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on "exists", error
        0, // on "forall", error
        -30, // on r#"\'[^\']+\'"#, reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        -30, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 30
        0, // on "(", error
        -9, // on ")", reduce `Application = (Operator Value)+ => ActionFn(48);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        41, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 40
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        43, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 42
        // State 31
        0, // on "(", error
        81, // on ")", goto 80
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 32
        0, // on "(", error
        -37, // on ")", reduce `Value = Atom => ActionFn(25);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -37, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = Atom => ActionFn(25);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -37, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = Atom => ActionFn(25);`
        // State 33
        82, // on "(", goto 81
        -43, // on ")", reduce `Variable = Identifier => ActionFn(30);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -43, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Variable = Identifier => ActionFn(30);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -43, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Variable = Identifier => ActionFn(30);`
        // State 34
        38, // on "(", goto 37
        -5, // on ")", reduce `Application = Operator => ActionFn(16);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        39, // on "_", goto 38
        0, // on "exists", error
        0, // on "forall", error
        40, // on r#"\'[^\']+\'"#, goto 39
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        85, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 84
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 35
        0, // on "(", error
        -4, // on ")", reduce `Application = Value => ActionFn(15);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        29, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 28
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        30, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 29
        // State 36
        0, // on "(", error
        -38, // on ")", reduce `Value = Variable => ActionFn(26);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -38, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = Variable => ActionFn(26);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -38, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = Variable => ActionFn(26);`
        // State 37
        38, // on "(", goto 37
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        39, // on "_", goto 38
        0, // on "exists", error
        0, // on "forall", error
        40, // on r#"\'[^\']+\'"#, goto 39
        41, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 40
        42, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 41
        43, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 42
        // State 38
        0, // on "(", error
        -40, // on ")", reduce `Value = "_" => ActionFn(28);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -40, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = "_" => ActionFn(28);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -40, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = "_" => ActionFn(28);`
        // State 39
        0, // on "(", error
        -14, // on ")", reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -14, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -14, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        // State 40
        -31, // on "(", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        -31, // on ")", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        -31, // on "_", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on "exists", error
        0, // on "forall", error
        -31, // on r#"\'[^\']+\'"#, reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        -31, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 41
        -25, // on "(", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        -25, // on ")", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -25, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -25, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        // State 42
        -30, // on "(", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        -30, // on ")", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        -30, // on "_", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on "exists", error
        0, // on "forall", error
        -30, // on r#"\'[^\']+\'"#, reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        -30, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 43
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -3, // on ".", reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        -3, // on ":-", reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -3, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -3, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        // State 44
        0, // on "(", error
        0, // on ")", error
        -9, // on ",", reduce `Application = (Operator Value)+ => ActionFn(48);`
        0, // on "->", error
        -9, // on ".", reduce `Application = (Operator Value)+ => ActionFn(48);`
        0, // on ":-", error
        -9, // on ";", reduce `Application = (Operator Value)+ => ActionFn(48);`
        -9, // on "=>", reduce `Application = (Operator Value)+ => ActionFn(48);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        62, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 61
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        64, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 63
        // State 45
        0, // on "(", error
        0, // on ")", error
        -18, // on ",", reduce `FactApply = Application => ActionFn(14);`
        0, // on "->", error
        -18, // on ".", reduce `FactApply = Application => ActionFn(14);`
        0, // on ":-", error
        -18, // on ";", reduce `FactApply = Application => ActionFn(14);`
        -18, // on "=>", reduce `FactApply = Application => ActionFn(14);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 46
        0, // on "(", error
        0, // on ")", error
        -37, // on ",", reduce `Value = Atom => ActionFn(25);`
        0, // on "->", error
        -37, // on ".", reduce `Value = Atom => ActionFn(25);`
        0, // on ":-", error
        -37, // on ";", reduce `Value = Atom => ActionFn(25);`
        -37, // on "=>", reduce `Value = Atom => ActionFn(25);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -37, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = Atom => ActionFn(25);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -37, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = Atom => ActionFn(25);`
        // State 47
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        90, // on ".", goto 89
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 48
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -15, // on ".", reduce `Fact = FactAnd => ActionFn(5);`
        0, // on ":-", error
        91, // on ";", goto 90
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 49
        0, // on "(", error
        0, // on ")", error
        -19, // on ",", reduce `FactFunc = FactApply => ActionFn(10);`
        0, // on "->", error
        -19, // on ".", reduce `FactFunc = FactApply => ActionFn(10);`
        0, // on ":-", error
        -19, // on ";", reduce `FactFunc = FactApply => ActionFn(10);`
        92, // on "=>", goto 91
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 50
        0, // on "(", error
        0, // on ")", error
        -23, // on ",", reduce `FactOr = FactFunc => ActionFn(8);`
        0, // on "->", error
        -23, // on ".", reduce `FactOr = FactFunc => ActionFn(8);`
        0, // on ":-", error
        -23, // on ";", reduce `FactOr = FactFunc => ActionFn(8);`
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 51
        0, // on "(", error
        0, // on ")", error
        93, // on ",", goto 92
        0, // on "->", error
        -16, // on ".", reduce `FactAnd = FactOr => ActionFn(6);`
        0, // on ":-", error
        -16, // on ";", reduce `FactAnd = FactOr => ActionFn(6);`
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 52
        94, // on "(", goto 93
        0, // on ")", error
        -43, // on ",", reduce `Variable = Identifier => ActionFn(30);`
        0, // on "->", error
        -43, // on ".", reduce `Variable = Identifier => ActionFn(30);`
        0, // on ":-", error
        -43, // on ";", reduce `Variable = Identifier => ActionFn(30);`
        -43, // on "=>", reduce `Variable = Identifier => ActionFn(30);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -43, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Variable = Identifier => ActionFn(30);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -43, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Variable = Identifier => ActionFn(30);`
        // State 53
        57, // on "(", goto 56
        0, // on ")", error
        -5, // on ",", reduce `Application = Operator => ActionFn(16);`
        0, // on "->", error
        -5, // on ".", reduce `Application = Operator => ActionFn(16);`
        0, // on ":-", error
        -5, // on ";", reduce `Application = Operator => ActionFn(16);`
        -5, // on "=>", reduce `Application = Operator => ActionFn(16);`
        58, // on "_", goto 57
        0, // on "exists", error
        0, // on "forall", error
        61, // on r#"\'[^\']+\'"#, goto 60
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        97, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 96
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 54
        0, // on "(", error
        0, // on ")", error
        -4, // on ",", reduce `Application = Value => ActionFn(15);`
        0, // on "->", error
        -4, // on ".", reduce `Application = Value => ActionFn(15);`
        0, // on ":-", error
        -4, // on ";", reduce `Application = Value => ActionFn(15);`
        -4, // on "=>", reduce `Application = Value => ActionFn(15);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        29, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 28
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        30, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 29
        // State 55
        0, // on "(", error
        0, // on ")", error
        -38, // on ",", reduce `Value = Variable => ActionFn(26);`
        0, // on "->", error
        -38, // on ".", reduce `Value = Variable => ActionFn(26);`
        0, // on ":-", error
        -38, // on ";", reduce `Value = Variable => ActionFn(26);`
        -38, // on "=>", reduce `Value = Variable => ActionFn(26);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -38, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = Variable => ActionFn(26);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -38, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = Variable => ActionFn(26);`
        // State 56
        38, // on "(", goto 37
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        39, // on "_", goto 38
        0, // on "exists", error
        0, // on "forall", error
        40, // on r#"\'[^\']+\'"#, goto 39
        41, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 40
        42, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 41
        43, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 42
        // State 57
        0, // on "(", error
        0, // on ")", error
        -40, // on ",", reduce `Value = "_" => ActionFn(28);`
        0, // on "->", error
        -40, // on ".", reduce `Value = "_" => ActionFn(28);`
        0, // on ":-", error
        -40, // on ";", reduce `Value = "_" => ActionFn(28);`
        -40, // on "=>", reduce `Value = "_" => ActionFn(28);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -40, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = "_" => ActionFn(28);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -40, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = "_" => ActionFn(28);`
        // State 58
        101, // on "(", goto 100
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 59
        102, // on "(", goto 101
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 60
        0, // on "(", error
        0, // on ")", error
        -14, // on ",", reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        0, // on "->", error
        -14, // on ".", reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        0, // on ":-", error
        -14, // on ";", reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        -14, // on "=>", reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -14, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -14, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        // State 61
        -31, // on "(", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on ")", error
        -31, // on ",", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on "->", error
        -31, // on ".", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on ":-", error
        -31, // on ";", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        -31, // on "=>", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        -31, // on "_", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on "exists", error
        0, // on "forall", error
        -31, // on r#"\'[^\']+\'"#, reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        -31, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 62
        -25, // on "(", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on ")", error
        -25, // on ",", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on "->", error
        -25, // on ".", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on ":-", error
        -25, // on ";", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        -25, // on "=>", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -25, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -25, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        // State 63
        -30, // on "(", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on ")", error
        -30, // on ",", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on "->", error
        -30, // on ".", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on ":-", error
        -30, // on ";", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        -30, // on "=>", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        -30, // on "_", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on "exists", error
        0, // on "forall", error
        -30, // on r#"\'[^\']+\'"#, reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        -30, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 64
        0, // on "(", error
        -9, // on ")", reduce `Application = (Operator Value)+ => ActionFn(48);`
        -9, // on ",", reduce `Application = (Operator Value)+ => ActionFn(48);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        76, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 75
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        78, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 77
        // State 65
        0, // on "(", error
        -11, // on ")", reduce `Applications = Application => ActionFn(19);`
        -11, // on ",", reduce `Applications = Application => ActionFn(19);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 66
        0, // on "(", error
        104, // on ")", goto 103
        105, // on ",", goto 104
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 67
        0, // on "(", error
        -37, // on ")", reduce `Value = Atom => ActionFn(25);`
        -37, // on ",", reduce `Value = Atom => ActionFn(25);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -37, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = Atom => ActionFn(25);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -37, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = Atom => ActionFn(25);`
        // State 68
        106, // on "(", goto 105
        -43, // on ")", reduce `Variable = Identifier => ActionFn(30);`
        -43, // on ",", reduce `Variable = Identifier => ActionFn(30);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -43, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Variable = Identifier => ActionFn(30);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -43, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Variable = Identifier => ActionFn(30);`
        // State 69
        73, // on "(", goto 72
        -5, // on ")", reduce `Application = Operator => ActionFn(16);`
        -5, // on ",", reduce `Application = Operator => ActionFn(16);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        74, // on "_", goto 73
        0, // on "exists", error
        0, // on "forall", error
        75, // on r#"\'[^\']+\'"#, goto 74
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        109, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 108
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 70
        0, // on "(", error
        -4, // on ")", reduce `Application = Value => ActionFn(15);`
        -4, // on ",", reduce `Application = Value => ActionFn(15);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        29, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 28
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        30, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 29
        // State 71
        0, // on "(", error
        -38, // on ")", reduce `Value = Variable => ActionFn(26);`
        -38, // on ",", reduce `Value = Variable => ActionFn(26);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -38, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = Variable => ActionFn(26);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -38, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = Variable => ActionFn(26);`
        // State 72
        38, // on "(", goto 37
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        39, // on "_", goto 38
        0, // on "exists", error
        0, // on "forall", error
        40, // on r#"\'[^\']+\'"#, goto 39
        41, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 40
        42, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 41
        43, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 42
        // State 73
        0, // on "(", error
        -40, // on ")", reduce `Value = "_" => ActionFn(28);`
        -40, // on ",", reduce `Value = "_" => ActionFn(28);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -40, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = "_" => ActionFn(28);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -40, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = "_" => ActionFn(28);`
        // State 74
        0, // on "(", error
        -14, // on ")", reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        -14, // on ",", reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -14, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -14, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        // State 75
        -31, // on "(", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        -31, // on ")", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        -31, // on ",", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        -31, // on "_", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on "exists", error
        0, // on "forall", error
        -31, // on r#"\'[^\']+\'"#, reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        -31, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 76
        -25, // on "(", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        -25, // on ")", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        -25, // on ",", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -25, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -25, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        // State 77
        -30, // on "(", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        -30, // on ")", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        -30, // on ",", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        -30, // on "_", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on "exists", error
        0, // on "forall", error
        -30, // on r#"\'[^\']+\'"#, reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        -30, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 78
        13, // on "(", goto 12
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -6, // on ".", reduce `Application = Value, (Operator Value)+, Operator => ActionFn(45);`
        -6, // on ":-", reduce `Application = Value, (Operator Value)+, Operator => ActionFn(45);`
        0, // on ";", error
        0, // on "=>", error
        14, // on "_", goto 13
        0, // on "exists", error
        0, // on "forall", error
        15, // on r#"\'[^\']+\'"#, goto 14
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        26, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 25
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 79
        38, // on "(", goto 37
        -7, // on ")", reduce `Application = (Operator Value)+, Operator => ActionFn(46);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        39, // on "_", goto 38
        0, // on "exists", error
        0, // on "forall", error
        40, // on r#"\'[^\']+\'"#, goto 39
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        85, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 84
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 80
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -39, // on ".", reduce `Value = "(", Application, ")" => ActionFn(27);`
        -39, // on ":-", reduce `Value = "(", Application, ")" => ActionFn(27);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -39, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = "(", Application, ")" => ActionFn(27);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -39, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = "(", Application, ")" => ActionFn(27);`
        // State 81
        73, // on "(", goto 72
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        74, // on "_", goto 73
        0, // on "exists", error
        0, // on "forall", error
        75, // on r#"\'[^\']+\'"#, goto 74
        76, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 75
        77, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 76
        78, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 77
        // State 82
        0, // on "(", error
        -43, // on ")", reduce `Variable = Identifier => ActionFn(30);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -43, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Variable = Identifier => ActionFn(30);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -43, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Variable = Identifier => ActionFn(30);`
        // State 83
        0, // on "(", error
        -2, // on ")", reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -2, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -2, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        // State 84
        0, // on "(", error
        -25, // on ")", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -25, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -25, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        // State 85
        0, // on "(", error
        -8, // on ")", reduce `Application = Value, (Operator Value)+ => ActionFn(47);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        41, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 40
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        43, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 42
        // State 86
        38, // on "(", goto 37
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        39, // on "_", goto 38
        0, // on "exists", error
        0, // on "forall", error
        40, // on r#"\'[^\']+\'"#, goto 39
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        85, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 84
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 87
        0, // on "(", error
        116, // on ")", goto 115
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 88
        57, // on "(", goto 56
        0, // on ")", error
        -7, // on ",", reduce `Application = (Operator Value)+, Operator => ActionFn(46);`
        0, // on "->", error
        -7, // on ".", reduce `Application = (Operator Value)+, Operator => ActionFn(46);`
        0, // on ":-", error
        -7, // on ";", reduce `Application = (Operator Value)+, Operator => ActionFn(46);`
        -7, // on "=>", reduce `Application = (Operator Value)+, Operator => ActionFn(46);`
        58, // on "_", goto 57
        0, // on "exists", error
        0, // on "forall", error
        61, // on r#"\'[^\']+\'"#, goto 60
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        97, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 96
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 89
        -36, // on "(", reduce `Rule = Application, ":-", Fact, "." => ActionFn(4);`
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        -36, // on "_", reduce `Rule = Application, ":-", Fact, "." => ActionFn(4);`
        0, // on "exists", error
        0, // on "forall", error
        -36, // on r#"\'[^\']+\'"#, reduce `Rule = Application, ":-", Fact, "." => ActionFn(4);`
        -36, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Rule = Application, ":-", Fact, "." => ActionFn(4);`
        -36, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Rule = Application, ":-", Fact, "." => ActionFn(4);`
        -36, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Rule = Application, ":-", Fact, "." => ActionFn(4);`
        // State 90
        57, // on "(", goto 56
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        58, // on "_", goto 57
        59, // on "exists", goto 58
        60, // on "forall", goto 59
        61, // on r#"\'[^\']+\'"#, goto 60
        62, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 61
        63, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 62
        64, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 63
        // State 91
        57, // on "(", goto 56
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        58, // on "_", goto 57
        59, // on "exists", goto 58
        60, // on "forall", goto 59
        61, // on r#"\'[^\']+\'"#, goto 60
        62, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 61
        63, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 62
        64, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 63
        // State 92
        57, // on "(", goto 56
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        58, // on "_", goto 57
        59, // on "exists", goto 58
        60, // on "forall", goto 59
        61, // on r#"\'[^\']+\'"#, goto 60
        62, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 61
        63, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 62
        64, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 63
        // State 93
        73, // on "(", goto 72
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        74, // on "_", goto 73
        0, // on "exists", error
        0, // on "forall", error
        75, // on r#"\'[^\']+\'"#, goto 74
        76, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 75
        77, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 76
        78, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 77
        // State 94
        0, // on "(", error
        0, // on ")", error
        -43, // on ",", reduce `Variable = Identifier => ActionFn(30);`
        0, // on "->", error
        -43, // on ".", reduce `Variable = Identifier => ActionFn(30);`
        0, // on ":-", error
        -43, // on ";", reduce `Variable = Identifier => ActionFn(30);`
        -43, // on "=>", reduce `Variable = Identifier => ActionFn(30);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -43, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Variable = Identifier => ActionFn(30);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -43, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Variable = Identifier => ActionFn(30);`
        // State 95
        0, // on "(", error
        0, // on ")", error
        -2, // on ",", reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        0, // on "->", error
        -2, // on ".", reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        0, // on ":-", error
        -2, // on ";", reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        -2, // on "=>", reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -2, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -2, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        // State 96
        0, // on "(", error
        0, // on ")", error
        -25, // on ",", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on "->", error
        -25, // on ".", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on ":-", error
        -25, // on ";", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        -25, // on "=>", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -25, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -25, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        // State 97
        0, // on "(", error
        0, // on ")", error
        -8, // on ",", reduce `Application = Value, (Operator Value)+ => ActionFn(47);`
        0, // on "->", error
        -8, // on ".", reduce `Application = Value, (Operator Value)+ => ActionFn(47);`
        0, // on ":-", error
        -8, // on ";", reduce `Application = Value, (Operator Value)+ => ActionFn(47);`
        -8, // on "=>", reduce `Application = Value, (Operator Value)+ => ActionFn(47);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        62, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 61
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        64, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 63
        // State 98
        57, // on "(", goto 56
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        58, // on "_", goto 57
        0, // on "exists", error
        0, // on "forall", error
        61, // on r#"\'[^\']+\'"#, goto 60
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        97, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 96
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 99
        0, // on "(", error
        123, // on ")", goto 122
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 100
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        126, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 125
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 101
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        126, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 125
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 102
        73, // on "(", goto 72
        -7, // on ")", reduce `Application = (Operator Value)+, Operator => ActionFn(46);`
        -7, // on ",", reduce `Application = (Operator Value)+, Operator => ActionFn(46);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        74, // on "_", goto 73
        0, // on "exists", error
        0, // on "forall", error
        75, // on r#"\'[^\']+\'"#, goto 74
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        109, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 108
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 103
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -10, // on ".", reduce `Application = Identifier, "(", Applications, ")" => ActionFn(18);`
        -10, // on ":-", reduce `Application = Identifier, "(", Applications, ")" => ActionFn(18);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 104
        73, // on "(", goto 72
        -13, // on ")", reduce `Applications = Applications, "," => ActionFn(21);`
        -13, // on ",", reduce `Applications = Applications, "," => ActionFn(21);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        74, // on "_", goto 73
        0, // on "exists", error
        0, // on "forall", error
        75, // on r#"\'[^\']+\'"#, goto 74
        76, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 75
        77, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 76
        78, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 77
        // State 105
        73, // on "(", goto 72
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        74, // on "_", goto 73
        0, // on "exists", error
        0, // on "forall", error
        75, // on r#"\'[^\']+\'"#, goto 74
        76, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 75
        77, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 76
        78, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 77
        // State 106
        0, // on "(", error
        -43, // on ")", reduce `Variable = Identifier => ActionFn(30);`
        -43, // on ",", reduce `Variable = Identifier => ActionFn(30);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -43, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Variable = Identifier => ActionFn(30);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -43, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Variable = Identifier => ActionFn(30);`
        // State 107
        0, // on "(", error
        -2, // on ")", reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        -2, // on ",", reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -2, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -2, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        // State 108
        0, // on "(", error
        -25, // on ")", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        -25, // on ",", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -25, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -25, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        // State 109
        0, // on "(", error
        -8, // on ")", reduce `Application = Value, (Operator Value)+ => ActionFn(47);`
        -8, // on ",", reduce `Application = Value, (Operator Value)+ => ActionFn(47);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        76, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 75
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        78, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 77
        // State 110
        73, // on "(", goto 72
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        74, // on "_", goto 73
        0, // on "exists", error
        0, // on "forall", error
        75, // on r#"\'[^\']+\'"#, goto 74
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        109, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 108
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 111
        0, // on "(", error
        132, // on ")", goto 131
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 112
        0, // on "(", error
        -3, // on ")", reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -3, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -3, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        // State 113
        0, // on "(", error
        133, // on ")", goto 132
        105, // on ",", goto 104
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 114
        38, // on "(", goto 37
        -6, // on ")", reduce `Application = Value, (Operator Value)+, Operator => ActionFn(45);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        39, // on "_", goto 38
        0, // on "exists", error
        0, // on "forall", error
        40, // on r#"\'[^\']+\'"#, goto 39
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        85, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 84
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 115
        0, // on "(", error
        -39, // on ")", reduce `Value = "(", Application, ")" => ActionFn(27);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -39, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = "(", Application, ")" => ActionFn(27);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -39, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = "(", Application, ")" => ActionFn(27);`
        // State 116
        0, // on "(", error
        0, // on ")", error
        -3, // on ",", reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        0, // on "->", error
        -3, // on ".", reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        0, // on ":-", error
        -3, // on ";", reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        -3, // on "=>", reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -3, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -3, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        // State 117
        0, // on "(", error
        0, // on ")", error
        93, // on ",", goto 92
        0, // on "->", error
        -17, // on ".", reduce `FactAnd = FactAnd, ";", FactOr => ActionFn(7);`
        0, // on ":-", error
        -17, // on ";", reduce `FactAnd = FactAnd, ";", FactOr => ActionFn(7);`
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 118
        0, // on "(", error
        0, // on ")", error
        -20, // on ",", reduce `FactFunc = FactApply, "=>", FactFunc => ActionFn(11);`
        0, // on "->", error
        -20, // on ".", reduce `FactFunc = FactApply, "=>", FactFunc => ActionFn(11);`
        0, // on ":-", error
        -20, // on ";", reduce `FactFunc = FactApply, "=>", FactFunc => ActionFn(11);`
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 119
        0, // on "(", error
        0, // on ")", error
        -24, // on ",", reduce `FactOr = FactOr, ",", FactFunc => ActionFn(9);`
        0, // on "->", error
        -24, // on ".", reduce `FactOr = FactOr, ",", FactFunc => ActionFn(9);`
        0, // on ":-", error
        -24, // on ";", reduce `FactOr = FactOr, ",", FactFunc => ActionFn(9);`
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 120
        0, // on "(", error
        134, // on ")", goto 133
        105, // on ",", goto 104
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 121
        57, // on "(", goto 56
        0, // on ")", error
        -6, // on ",", reduce `Application = Value, (Operator Value)+, Operator => ActionFn(45);`
        0, // on "->", error
        -6, // on ".", reduce `Application = Value, (Operator Value)+, Operator => ActionFn(45);`
        0, // on ":-", error
        -6, // on ";", reduce `Application = Value, (Operator Value)+, Operator => ActionFn(45);`
        -6, // on "=>", reduce `Application = Value, (Operator Value)+, Operator => ActionFn(45);`
        58, // on "_", goto 57
        0, // on "exists", error
        0, // on "forall", error
        61, // on r#"\'[^\']+\'"#, goto 60
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        97, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 96
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 122
        0, // on "(", error
        0, // on ")", error
        -39, // on ",", reduce `Value = "(", Application, ")" => ActionFn(27);`
        0, // on "->", error
        -39, // on ".", reduce `Value = "(", Application, ")" => ActionFn(27);`
        0, // on ":-", error
        -39, // on ";", reduce `Value = "(", Application, ")" => ActionFn(27);`
        -39, // on "=>", reduce `Value = "(", Application, ")" => ActionFn(27);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -39, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = "(", Application, ")" => ActionFn(27);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -39, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = "(", Application, ")" => ActionFn(27);`
        // State 123
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        -43, // on "->", reduce `Variable = Identifier => ActionFn(30);`
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 124
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        135, // on "->", goto 134
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 125
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        -25, // on "->", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 126
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        136, // on "->", goto 135
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 127
        0, // on "(", error
        -3, // on ")", reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        -3, // on ",", reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -3, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -3, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        // State 128
        0, // on "(", error
        -12, // on ")", reduce `Applications = Applications, ",", Application => ActionFn(20);`
        -12, // on ",", reduce `Applications = Applications, ",", Application => ActionFn(20);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 129
        0, // on "(", error
        137, // on ")", goto 136
        105, // on ",", goto 104
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 130
        73, // on "(", goto 72
        -6, // on ")", reduce `Application = Value, (Operator Value)+, Operator => ActionFn(45);`
        -6, // on ",", reduce `Application = Value, (Operator Value)+, Operator => ActionFn(45);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        74, // on "_", goto 73
        0, // on "exists", error
        0, // on "forall", error
        75, // on r#"\'[^\']+\'"#, goto 74
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        109, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 108
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 131
        0, // on "(", error
        -39, // on ")", reduce `Value = "(", Application, ")" => ActionFn(27);`
        -39, // on ",", reduce `Value = "(", Application, ")" => ActionFn(27);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -39, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = "(", Application, ")" => ActionFn(27);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -39, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = "(", Application, ")" => ActionFn(27);`
        // State 132
        0, // on "(", error
        -10, // on ")", reduce `Application = Identifier, "(", Applications, ")" => ActionFn(18);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 133
        0, // on "(", error
        0, // on ")", error
        -10, // on ",", reduce `Application = Identifier, "(", Applications, ")" => ActionFn(18);`
        0, // on "->", error
        -10, // on ".", reduce `Application = Identifier, "(", Applications, ")" => ActionFn(18);`
        0, // on ":-", error
        -10, // on ";", reduce `Application = Identifier, "(", Applications, ")" => ActionFn(18);`
        -10, // on "=>", reduce `Application = Identifier, "(", Applications, ")" => ActionFn(18);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 134
        147, // on "(", goto 146
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        148, // on "_", goto 147
        149, // on "exists", goto 148
        150, // on "forall", goto 149
        151, // on r#"\'[^\']+\'"#, goto 150
        152, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 151
        153, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 152
        154, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 153
        // State 135
        147, // on "(", goto 146
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        148, // on "_", goto 147
        149, // on "exists", goto 148
        150, // on "forall", goto 149
        151, // on r#"\'[^\']+\'"#, goto 150
        152, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 151
        153, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 152
        154, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 153
        // State 136
        0, // on "(", error
        -10, // on ")", reduce `Application = Identifier, "(", Applications, ")" => ActionFn(18);`
        -10, // on ",", reduce `Application = Identifier, "(", Applications, ")" => ActionFn(18);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 137
        0, // on "(", error
        -9, // on ")", reduce `Application = (Operator Value)+ => ActionFn(48);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        -9, // on "=>", reduce `Application = (Operator Value)+ => ActionFn(48);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        152, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 151
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        154, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 153
        // State 138
        0, // on "(", error
        -18, // on ")", reduce `FactApply = Application => ActionFn(14);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        -18, // on "=>", reduce `FactApply = Application => ActionFn(14);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 139
        0, // on "(", error
        -37, // on ")", reduce `Value = Atom => ActionFn(25);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        -37, // on "=>", reduce `Value = Atom => ActionFn(25);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -37, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = Atom => ActionFn(25);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -37, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = Atom => ActionFn(25);`
        // State 140
        0, // on "(", error
        -19, // on ")", reduce `FactFunc = FactApply => ActionFn(10);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        157, // on "=>", goto 156
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 141
        0, // on "(", error
        158, // on ")", goto 157
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 142
        159, // on "(", goto 158
        -43, // on ")", reduce `Variable = Identifier => ActionFn(30);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        -43, // on "=>", reduce `Variable = Identifier => ActionFn(30);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -43, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Variable = Identifier => ActionFn(30);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -43, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Variable = Identifier => ActionFn(30);`
        // State 143
        147, // on "(", goto 146
        -5, // on ")", reduce `Application = Operator => ActionFn(16);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        -5, // on "=>", reduce `Application = Operator => ActionFn(16);`
        148, // on "_", goto 147
        0, // on "exists", error
        0, // on "forall", error
        151, // on r#"\'[^\']+\'"#, goto 150
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        162, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 161
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 144
        0, // on "(", error
        -4, // on ")", reduce `Application = Value => ActionFn(15);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        -4, // on "=>", reduce `Application = Value => ActionFn(15);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        29, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 28
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        30, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 29
        // State 145
        0, // on "(", error
        -38, // on ")", reduce `Value = Variable => ActionFn(26);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        -38, // on "=>", reduce `Value = Variable => ActionFn(26);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -38, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = Variable => ActionFn(26);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -38, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = Variable => ActionFn(26);`
        // State 146
        38, // on "(", goto 37
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        39, // on "_", goto 38
        0, // on "exists", error
        0, // on "forall", error
        40, // on r#"\'[^\']+\'"#, goto 39
        41, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 40
        42, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 41
        43, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 42
        // State 147
        0, // on "(", error
        -40, // on ")", reduce `Value = "_" => ActionFn(28);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        -40, // on "=>", reduce `Value = "_" => ActionFn(28);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -40, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = "_" => ActionFn(28);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -40, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = "_" => ActionFn(28);`
        // State 148
        166, // on "(", goto 165
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 149
        167, // on "(", goto 166
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 150
        0, // on "(", error
        -14, // on ")", reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        -14, // on "=>", reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -14, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -14, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        // State 151
        -31, // on "(", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        -31, // on ")", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        -31, // on "=>", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        -31, // on "_", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on "exists", error
        0, // on "forall", error
        -31, // on r#"\'[^\']+\'"#, reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        -31, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 152
        -25, // on "(", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        -25, // on ")", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        -25, // on "=>", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -25, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -25, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        // State 153
        -30, // on "(", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        -30, // on ")", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        -30, // on "=>", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        -30, // on "_", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on "exists", error
        0, // on "forall", error
        -30, // on r#"\'[^\']+\'"#, reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        -30, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 154
        0, // on "(", error
        168, // on ")", goto 167
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 155
        147, // on "(", goto 146
        -7, // on ")", reduce `Application = (Operator Value)+, Operator => ActionFn(46);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        -7, // on "=>", reduce `Application = (Operator Value)+, Operator => ActionFn(46);`
        148, // on "_", goto 147
        0, // on "exists", error
        0, // on "forall", error
        151, // on r#"\'[^\']+\'"#, goto 150
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        162, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 161
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 156
        147, // on "(", goto 146
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        148, // on "_", goto 147
        149, // on "exists", goto 148
        150, // on "forall", goto 149
        151, // on r#"\'[^\']+\'"#, goto 150
        152, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 151
        153, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 152
        154, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 153
        // State 157
        0, // on "(", error
        0, // on ")", error
        -21, // on ",", reduce `FactFunc = "exists", "(", Variable, "->", FactFunc, ")" => ActionFn(12);`
        0, // on "->", error
        -21, // on ".", reduce `FactFunc = "exists", "(", Variable, "->", FactFunc, ")" => ActionFn(12);`
        0, // on ":-", error
        -21, // on ";", reduce `FactFunc = "exists", "(", Variable, "->", FactFunc, ")" => ActionFn(12);`
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 158
        73, // on "(", goto 72
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        74, // on "_", goto 73
        0, // on "exists", error
        0, // on "forall", error
        75, // on r#"\'[^\']+\'"#, goto 74
        76, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 75
        77, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 76
        78, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 77
        // State 159
        0, // on "(", error
        -43, // on ")", reduce `Variable = Identifier => ActionFn(30);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        -43, // on "=>", reduce `Variable = Identifier => ActionFn(30);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -43, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Variable = Identifier => ActionFn(30);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -43, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Variable = Identifier => ActionFn(30);`
        // State 160
        0, // on "(", error
        -2, // on ")", reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        -2, // on "=>", reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -2, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -2, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        // State 161
        0, // on "(", error
        -25, // on ")", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        -25, // on "=>", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -25, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -25, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        // State 162
        0, // on "(", error
        -8, // on ")", reduce `Application = Value, (Operator Value)+ => ActionFn(47);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        -8, // on "=>", reduce `Application = Value, (Operator Value)+ => ActionFn(47);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        152, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 151
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        154, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 153
        // State 163
        147, // on "(", goto 146
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        148, // on "_", goto 147
        0, // on "exists", error
        0, // on "forall", error
        151, // on r#"\'[^\']+\'"#, goto 150
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        162, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 161
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 164
        0, // on "(", error
        173, // on ")", goto 172
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 165
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        126, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 125
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 166
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        126, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 125
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 167
        0, // on "(", error
        0, // on ")", error
        -22, // on ",", reduce `FactFunc = "forall", "(", Variable, "->", FactFunc, ")" => ActionFn(13);`
        0, // on "->", error
        -22, // on ".", reduce `FactFunc = "forall", "(", Variable, "->", FactFunc, ")" => ActionFn(13);`
        0, // on ":-", error
        -22, // on ";", reduce `FactFunc = "forall", "(", Variable, "->", FactFunc, ")" => ActionFn(13);`
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 168
        0, // on "(", error
        -3, // on ")", reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        -3, // on "=>", reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -3, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -3, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        // State 169
        0, // on "(", error
        -20, // on ")", reduce `FactFunc = FactApply, "=>", FactFunc => ActionFn(11);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 170
        0, // on "(", error
        176, // on ")", goto 175
        105, // on ",", goto 104
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 171
        147, // on "(", goto 146
        -6, // on ")", reduce `Application = Value, (Operator Value)+, Operator => ActionFn(45);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        -6, // on "=>", reduce `Application = Value, (Operator Value)+, Operator => ActionFn(45);`
        148, // on "_", goto 147
        0, // on "exists", error
        0, // on "forall", error
        151, // on r#"\'[^\']+\'"#, goto 150
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        162, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 161
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 172
        0, // on "(", error
        -39, // on ")", reduce `Value = "(", Application, ")" => ActionFn(27);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        -39, // on "=>", reduce `Value = "(", Application, ")" => ActionFn(27);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -39, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = "(", Application, ")" => ActionFn(27);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -39, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = "(", Application, ")" => ActionFn(27);`
        // State 173
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        177, // on "->", goto 176
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 174
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        178, // on "->", goto 177
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 175
        0, // on "(", error
        -10, // on ")", reduce `Application = Identifier, "(", Applications, ")" => ActionFn(18);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        -10, // on "=>", reduce `Application = Identifier, "(", Applications, ")" => ActionFn(18);`
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 176
        147, // on "(", goto 146
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        148, // on "_", goto 147
        149, // on "exists", goto 148
        150, // on "forall", goto 149
        151, // on r#"\'[^\']+\'"#, goto 150
        152, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 151
        153, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 152
        154, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 153
        // State 177
        147, // on "(", goto 146
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        148, // on "_", goto 147
        149, // on "exists", goto 148
        150, // on "forall", goto 149
        151, // on r#"\'[^\']+\'"#, goto 150
        152, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 151
        153, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 152
        154, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 153
        // State 178
        0, // on "(", error
        181, // on ")", goto 180
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 179
        0, // on "(", error
        182, // on ")", goto 181
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 180
        0, // on "(", error
        -21, // on ")", reduce `FactFunc = "exists", "(", Variable, "->", FactFunc, ")" => ActionFn(12);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 181
        0, // on "(", error
        -22, // on ")", reduce `FactFunc = "forall", "(", Variable, "->", FactFunc, ")" => ActionFn(13);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "_", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
    ];
    const __EOF_ACTION: &'static [i32] = &[
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        -28, // on EOF, reduce `Item+ = Item => ActionFn(39);`
        -35, // on EOF, reduce `Program = Item+ => ActionFn(1);`
        0, // on EOF, error
        -44, // on EOF, reduce `__Program = Program => ActionFn(0);`
        -27, // on EOF, reduce `Item = Rule => ActionFn(3);`
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        -26, // on EOF, reduce `Item = Application, "." => ActionFn(2);`
        0, // on EOF, error
        0, // on EOF, error
        -29, // on EOF, reduce `Item+ = Item+, Item => ActionFn(40);`
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        -36, // on EOF, reduce `Rule = Application, ":-", Fact, "." => ActionFn(4);`
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
    ];
    const __GOTO: &'static [i32] = &[
        // State 0
        0, // on (Operator Value), error
        2, // on (Operator Value)+, goto 1
        3, // on Application, goto 2
        0, // on Applications, error
        4, // on Atom, goto 3
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        5, // on Identifier, goto 4
        6, // on Item, goto 5
        7, // on Item+, goto 6
        8, // on Operator, goto 7
        0, // on Operator?, error
        0, // on OperatorValue, error
        9, // on Program, goto 8
        10, // on Rule, goto 9
        11, // on Value, goto 10
        0, // on Value?, error
        12, // on Variable, goto 11
        0, // on __Program, error
        // State 1
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        19, // on Operator, goto 18
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 2
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 3
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 4
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 5
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 6
        0, // on (Operator Value), error
        2, // on (Operator Value)+, goto 1
        3, // on Application, goto 2
        0, // on Applications, error
        4, // on Atom, goto 3
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        5, // on Identifier, goto 4
        23, // on Item, goto 22
        0, // on Item+, error
        8, // on Operator, goto 7
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        10, // on Rule, goto 9
        11, // on Value, goto 10
        0, // on Value?, error
        12, // on Variable, goto 11
        0, // on __Program, error
        // State 7
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        4, // on Atom, goto 3
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        24, // on Identifier, goto 23
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        25, // on Value, goto 24
        0, // on Value?, error
        12, // on Variable, goto 11
        0, // on __Program, error
        // State 8
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 9
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 10
        0, // on (Operator Value), error
        27, // on (Operator Value)+, goto 26
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        28, // on Operator, goto 27
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 11
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 12
        0, // on (Operator Value), error
        31, // on (Operator Value)+, goto 30
        32, // on Application, goto 31
        0, // on Applications, error
        33, // on Atom, goto 32
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        34, // on Identifier, goto 33
        0, // on Item, error
        0, // on Item+, error
        35, // on Operator, goto 34
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        36, // on Value, goto 35
        0, // on Value?, error
        37, // on Variable, goto 36
        0, // on __Program, error
        // State 13
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 14
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 15
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 16
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 17
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 18
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        4, // on Atom, goto 3
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        24, // on Identifier, goto 23
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        44, // on Value, goto 43
        0, // on Value?, error
        12, // on Variable, goto 11
        0, // on __Program, error
        // State 19
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 20
        0, // on (Operator Value), error
        45, // on (Operator Value)+, goto 44
        46, // on Application, goto 45
        0, // on Applications, error
        47, // on Atom, goto 46
        48, // on Fact, goto 47
        49, // on FactAnd, goto 48
        50, // on FactApply, goto 49
        51, // on FactFunc, goto 50
        52, // on FactOr, goto 51
        53, // on Identifier, goto 52
        0, // on Item, error
        0, // on Item+, error
        54, // on Operator, goto 53
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        55, // on Value, goto 54
        0, // on Value?, error
        56, // on Variable, goto 55
        0, // on __Program, error
        // State 21
        0, // on (Operator Value), error
        65, // on (Operator Value)+, goto 64
        66, // on Application, goto 65
        67, // on Applications, goto 66
        68, // on Atom, goto 67
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        69, // on Identifier, goto 68
        0, // on Item, error
        0, // on Item+, error
        70, // on Operator, goto 69
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        71, // on Value, goto 70
        0, // on Value?, error
        72, // on Variable, goto 71
        0, // on __Program, error
        // State 22
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 23
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 24
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 25
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 26
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        79, // on Operator, goto 78
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 27
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        4, // on Atom, goto 3
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        24, // on Identifier, goto 23
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        25, // on Value, goto 24
        0, // on Value?, error
        12, // on Variable, goto 11
        0, // on __Program, error
        // State 28
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 29
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 30
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        80, // on Operator, goto 79
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 31
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 32
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 33
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 34
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        33, // on Atom, goto 32
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        83, // on Identifier, goto 82
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        84, // on Value, goto 83
        0, // on Value?, error
        37, // on Variable, goto 36
        0, // on __Program, error
        // State 35
        0, // on (Operator Value), error
        86, // on (Operator Value)+, goto 85
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        87, // on Operator, goto 86
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 36
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 37
        0, // on (Operator Value), error
        31, // on (Operator Value)+, goto 30
        88, // on Application, goto 87
        0, // on Applications, error
        33, // on Atom, goto 32
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        34, // on Identifier, goto 33
        0, // on Item, error
        0, // on Item+, error
        35, // on Operator, goto 34
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        36, // on Value, goto 35
        0, // on Value?, error
        37, // on Variable, goto 36
        0, // on __Program, error
        // State 38
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 39
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 40
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 41
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 42
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 43
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 44
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        89, // on Operator, goto 88
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 45
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 46
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 47
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 48
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 49
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 50
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 51
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 52
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 53
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        47, // on Atom, goto 46
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        95, // on Identifier, goto 94
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        96, // on Value, goto 95
        0, // on Value?, error
        56, // on Variable, goto 55
        0, // on __Program, error
        // State 54
        0, // on (Operator Value), error
        98, // on (Operator Value)+, goto 97
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        99, // on Operator, goto 98
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 55
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 56
        0, // on (Operator Value), error
        31, // on (Operator Value)+, goto 30
        100, // on Application, goto 99
        0, // on Applications, error
        33, // on Atom, goto 32
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        34, // on Identifier, goto 33
        0, // on Item, error
        0, // on Item+, error
        35, // on Operator, goto 34
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        36, // on Value, goto 35
        0, // on Value?, error
        37, // on Variable, goto 36
        0, // on __Program, error
        // State 57
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 58
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 59
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 60
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 61
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 62
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 63
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 64
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        103, // on Operator, goto 102
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 65
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 66
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 67
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 68
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 69
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        68, // on Atom, goto 67
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        107, // on Identifier, goto 106
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        108, // on Value, goto 107
        0, // on Value?, error
        72, // on Variable, goto 71
        0, // on __Program, error
        // State 70
        0, // on (Operator Value), error
        110, // on (Operator Value)+, goto 109
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        111, // on Operator, goto 110
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 71
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 72
        0, // on (Operator Value), error
        31, // on (Operator Value)+, goto 30
        112, // on Application, goto 111
        0, // on Applications, error
        33, // on Atom, goto 32
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        34, // on Identifier, goto 33
        0, // on Item, error
        0, // on Item+, error
        35, // on Operator, goto 34
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        36, // on Value, goto 35
        0, // on Value?, error
        37, // on Variable, goto 36
        0, // on __Program, error
        // State 73
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 74
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 75
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 76
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 77
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 78
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        4, // on Atom, goto 3
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        24, // on Identifier, goto 23
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        44, // on Value, goto 43
        0, // on Value?, error
        12, // on Variable, goto 11
        0, // on __Program, error
        // State 79
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        33, // on Atom, goto 32
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        83, // on Identifier, goto 82
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        113, // on Value, goto 112
        0, // on Value?, error
        37, // on Variable, goto 36
        0, // on __Program, error
        // State 80
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 81
        0, // on (Operator Value), error
        65, // on (Operator Value)+, goto 64
        66, // on Application, goto 65
        114, // on Applications, goto 113
        68, // on Atom, goto 67
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        69, // on Identifier, goto 68
        0, // on Item, error
        0, // on Item+, error
        70, // on Operator, goto 69
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        71, // on Value, goto 70
        0, // on Value?, error
        72, // on Variable, goto 71
        0, // on __Program, error
        // State 82
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 83
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 84
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 85
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        115, // on Operator, goto 114
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 86
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        33, // on Atom, goto 32
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        83, // on Identifier, goto 82
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        84, // on Value, goto 83
        0, // on Value?, error
        37, // on Variable, goto 36
        0, // on __Program, error
        // State 87
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 88
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        47, // on Atom, goto 46
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        95, // on Identifier, goto 94
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        117, // on Value, goto 116
        0, // on Value?, error
        56, // on Variable, goto 55
        0, // on __Program, error
        // State 89
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 90
        0, // on (Operator Value), error
        45, // on (Operator Value)+, goto 44
        46, // on Application, goto 45
        0, // on Applications, error
        47, // on Atom, goto 46
        0, // on Fact, error
        0, // on FactAnd, error
        50, // on FactApply, goto 49
        51, // on FactFunc, goto 50
        118, // on FactOr, goto 117
        53, // on Identifier, goto 52
        0, // on Item, error
        0, // on Item+, error
        54, // on Operator, goto 53
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        55, // on Value, goto 54
        0, // on Value?, error
        56, // on Variable, goto 55
        0, // on __Program, error
        // State 91
        0, // on (Operator Value), error
        45, // on (Operator Value)+, goto 44
        46, // on Application, goto 45
        0, // on Applications, error
        47, // on Atom, goto 46
        0, // on Fact, error
        0, // on FactAnd, error
        50, // on FactApply, goto 49
        119, // on FactFunc, goto 118
        0, // on FactOr, error
        53, // on Identifier, goto 52
        0, // on Item, error
        0, // on Item+, error
        54, // on Operator, goto 53
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        55, // on Value, goto 54
        0, // on Value?, error
        56, // on Variable, goto 55
        0, // on __Program, error
        // State 92
        0, // on (Operator Value), error
        45, // on (Operator Value)+, goto 44
        46, // on Application, goto 45
        0, // on Applications, error
        47, // on Atom, goto 46
        0, // on Fact, error
        0, // on FactAnd, error
        50, // on FactApply, goto 49
        120, // on FactFunc, goto 119
        0, // on FactOr, error
        53, // on Identifier, goto 52
        0, // on Item, error
        0, // on Item+, error
        54, // on Operator, goto 53
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        55, // on Value, goto 54
        0, // on Value?, error
        56, // on Variable, goto 55
        0, // on __Program, error
        // State 93
        0, // on (Operator Value), error
        65, // on (Operator Value)+, goto 64
        66, // on Application, goto 65
        121, // on Applications, goto 120
        68, // on Atom, goto 67
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        69, // on Identifier, goto 68
        0, // on Item, error
        0, // on Item+, error
        70, // on Operator, goto 69
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        71, // on Value, goto 70
        0, // on Value?, error
        72, // on Variable, goto 71
        0, // on __Program, error
        // State 94
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 95
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 96
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 97
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        122, // on Operator, goto 121
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 98
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        47, // on Atom, goto 46
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        95, // on Identifier, goto 94
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        96, // on Value, goto 95
        0, // on Value?, error
        56, // on Variable, goto 55
        0, // on __Program, error
        // State 99
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 100
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        124, // on Identifier, goto 123
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        125, // on Variable, goto 124
        0, // on __Program, error
        // State 101
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        124, // on Identifier, goto 123
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        127, // on Variable, goto 126
        0, // on __Program, error
        // State 102
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        68, // on Atom, goto 67
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        107, // on Identifier, goto 106
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        128, // on Value, goto 127
        0, // on Value?, error
        72, // on Variable, goto 71
        0, // on __Program, error
        // State 103
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 104
        0, // on (Operator Value), error
        65, // on (Operator Value)+, goto 64
        129, // on Application, goto 128
        0, // on Applications, error
        68, // on Atom, goto 67
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        69, // on Identifier, goto 68
        0, // on Item, error
        0, // on Item+, error
        70, // on Operator, goto 69
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        71, // on Value, goto 70
        0, // on Value?, error
        72, // on Variable, goto 71
        0, // on __Program, error
        // State 105
        0, // on (Operator Value), error
        65, // on (Operator Value)+, goto 64
        66, // on Application, goto 65
        130, // on Applications, goto 129
        68, // on Atom, goto 67
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        69, // on Identifier, goto 68
        0, // on Item, error
        0, // on Item+, error
        70, // on Operator, goto 69
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        71, // on Value, goto 70
        0, // on Value?, error
        72, // on Variable, goto 71
        0, // on __Program, error
        // State 106
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 107
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 108
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 109
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        131, // on Operator, goto 130
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 110
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        68, // on Atom, goto 67
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        107, // on Identifier, goto 106
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        108, // on Value, goto 107
        0, // on Value?, error
        72, // on Variable, goto 71
        0, // on __Program, error
        // State 111
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 112
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 113
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 114
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        33, // on Atom, goto 32
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        83, // on Identifier, goto 82
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        113, // on Value, goto 112
        0, // on Value?, error
        37, // on Variable, goto 36
        0, // on __Program, error
        // State 115
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 116
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 117
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 118
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 119
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 120
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 121
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        47, // on Atom, goto 46
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        95, // on Identifier, goto 94
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        117, // on Value, goto 116
        0, // on Value?, error
        56, // on Variable, goto 55
        0, // on __Program, error
        // State 122
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 123
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 124
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 125
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 126
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 127
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 128
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 129
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 130
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        68, // on Atom, goto 67
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        107, // on Identifier, goto 106
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        128, // on Value, goto 127
        0, // on Value?, error
        72, // on Variable, goto 71
        0, // on __Program, error
        // State 131
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 132
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 133
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 134
        0, // on (Operator Value), error
        138, // on (Operator Value)+, goto 137
        139, // on Application, goto 138
        0, // on Applications, error
        140, // on Atom, goto 139
        0, // on Fact, error
        0, // on FactAnd, error
        141, // on FactApply, goto 140
        142, // on FactFunc, goto 141
        0, // on FactOr, error
        143, // on Identifier, goto 142
        0, // on Item, error
        0, // on Item+, error
        144, // on Operator, goto 143
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        145, // on Value, goto 144
        0, // on Value?, error
        146, // on Variable, goto 145
        0, // on __Program, error
        // State 135
        0, // on (Operator Value), error
        138, // on (Operator Value)+, goto 137
        139, // on Application, goto 138
        0, // on Applications, error
        140, // on Atom, goto 139
        0, // on Fact, error
        0, // on FactAnd, error
        141, // on FactApply, goto 140
        155, // on FactFunc, goto 154
        0, // on FactOr, error
        143, // on Identifier, goto 142
        0, // on Item, error
        0, // on Item+, error
        144, // on Operator, goto 143
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        145, // on Value, goto 144
        0, // on Value?, error
        146, // on Variable, goto 145
        0, // on __Program, error
        // State 136
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 137
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        156, // on Operator, goto 155
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 138
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 139
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 140
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 141
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 142
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 143
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        140, // on Atom, goto 139
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        160, // on Identifier, goto 159
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        161, // on Value, goto 160
        0, // on Value?, error
        146, // on Variable, goto 145
        0, // on __Program, error
        // State 144
        0, // on (Operator Value), error
        163, // on (Operator Value)+, goto 162
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        164, // on Operator, goto 163
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 145
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 146
        0, // on (Operator Value), error
        31, // on (Operator Value)+, goto 30
        165, // on Application, goto 164
        0, // on Applications, error
        33, // on Atom, goto 32
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        34, // on Identifier, goto 33
        0, // on Item, error
        0, // on Item+, error
        35, // on Operator, goto 34
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        36, // on Value, goto 35
        0, // on Value?, error
        37, // on Variable, goto 36
        0, // on __Program, error
        // State 147
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 148
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 149
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 150
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 151
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 152
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 153
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 154
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 155
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        140, // on Atom, goto 139
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        160, // on Identifier, goto 159
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        169, // on Value, goto 168
        0, // on Value?, error
        146, // on Variable, goto 145
        0, // on __Program, error
        // State 156
        0, // on (Operator Value), error
        138, // on (Operator Value)+, goto 137
        139, // on Application, goto 138
        0, // on Applications, error
        140, // on Atom, goto 139
        0, // on Fact, error
        0, // on FactAnd, error
        141, // on FactApply, goto 140
        170, // on FactFunc, goto 169
        0, // on FactOr, error
        143, // on Identifier, goto 142
        0, // on Item, error
        0, // on Item+, error
        144, // on Operator, goto 143
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        145, // on Value, goto 144
        0, // on Value?, error
        146, // on Variable, goto 145
        0, // on __Program, error
        // State 157
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 158
        0, // on (Operator Value), error
        65, // on (Operator Value)+, goto 64
        66, // on Application, goto 65
        171, // on Applications, goto 170
        68, // on Atom, goto 67
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        69, // on Identifier, goto 68
        0, // on Item, error
        0, // on Item+, error
        70, // on Operator, goto 69
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        71, // on Value, goto 70
        0, // on Value?, error
        72, // on Variable, goto 71
        0, // on __Program, error
        // State 159
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 160
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 161
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 162
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        172, // on Operator, goto 171
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 163
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        140, // on Atom, goto 139
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        160, // on Identifier, goto 159
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        161, // on Value, goto 160
        0, // on Value?, error
        146, // on Variable, goto 145
        0, // on __Program, error
        // State 164
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 165
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        124, // on Identifier, goto 123
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        174, // on Variable, goto 173
        0, // on __Program, error
        // State 166
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        124, // on Identifier, goto 123
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        175, // on Variable, goto 174
        0, // on __Program, error
        // State 167
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 168
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 169
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 170
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 171
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        140, // on Atom, goto 139
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        160, // on Identifier, goto 159
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        169, // on Value, goto 168
        0, // on Value?, error
        146, // on Variable, goto 145
        0, // on __Program, error
        // State 172
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 173
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 174
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 175
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 176
        0, // on (Operator Value), error
        138, // on (Operator Value)+, goto 137
        139, // on Application, goto 138
        0, // on Applications, error
        140, // on Atom, goto 139
        0, // on Fact, error
        0, // on FactAnd, error
        141, // on FactApply, goto 140
        179, // on FactFunc, goto 178
        0, // on FactOr, error
        143, // on Identifier, goto 142
        0, // on Item, error
        0, // on Item+, error
        144, // on Operator, goto 143
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        145, // on Value, goto 144
        0, // on Value?, error
        146, // on Variable, goto 145
        0, // on __Program, error
        // State 177
        0, // on (Operator Value), error
        138, // on (Operator Value)+, goto 137
        139, // on Application, goto 138
        0, // on Applications, error
        140, // on Atom, goto 139
        0, // on Fact, error
        0, // on FactAnd, error
        141, // on FactApply, goto 140
        180, // on FactFunc, goto 179
        0, // on FactOr, error
        143, // on Identifier, goto 142
        0, // on Item, error
        0, // on Item+, error
        144, // on Operator, goto 143
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        145, // on Value, goto 144
        0, // on Value?, error
        146, // on Variable, goto 145
        0, // on __Program, error
        // State 178
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 179
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 180
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 181
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        0, // on Atom, error
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        0, // on Identifier, error
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
    ];
    pub fn parse_Program<
        'input,
    >(
        input: &'input str,
    ) -> Result<Program, __lalrpop_util::ParseError<usize,(usize, &'input str),()>>
    {
        let mut __tokens = super::__intern_token::__Matcher::new(input);
        let mut __states = vec![0_i32];
        let mut __symbols = vec![];
        '__shift: loop {
            let __lookahead = match __tokens.next() {
                Some(Ok(v)) => v,
                None => break '__shift,
                Some(Err(e)) => return Err(e),
            };
            let __integer = match __lookahead {
                (_, (0, _), _) if true => 0,
                (_, (1, _), _) if true => 1,
                (_, (2, _), _) if true => 2,
                (_, (3, _), _) if true => 3,
                (_, (4, _), _) if true => 4,
                (_, (5, _), _) if true => 5,
                (_, (6, _), _) if true => 6,
                (_, (7, _), _) if true => 7,
                (_, (8, _), _) if true => 8,
                (_, (9, _), _) if true => 9,
                (_, (10, _), _) if true => 10,
                (_, (11, _), _) if true => 11,
                (_, (12, _), _) if true => 12,
                (_, (13, _), _) if true => 13,
                (_, (14, _), _) if true => 14,
                _ => {
                    return Err(__lalrpop_util::ParseError::UnrecognizedToken {
                        token: Some(__lookahead),
                        expected: vec![],
                    });
                }
            };
            loop {
                let __state = *__states.last().unwrap() as usize;
                let __action = __ACTION[__state * 15 + __integer];
                if __action > 0 {
                    let __symbol = match __integer {
                        0 => match __lookahead.1 {
                            (0, __tok0) => __Symbol::Term_22_28_22(__tok0),
                            _ => unreachable!(),
                        },
                        1 => match __lookahead.1 {
                            (1, __tok0) => __Symbol::Term_22_29_22(__tok0),
                            _ => unreachable!(),
                        },
                        2 => match __lookahead.1 {
                            (2, __tok0) => __Symbol::Term_22_2c_22(__tok0),
                            _ => unreachable!(),
                        },
                        3 => match __lookahead.1 {
                            (3, __tok0) => __Symbol::Term_22_2d_3e_22(__tok0),
                            _ => unreachable!(),
                        },
                        4 => match __lookahead.1 {
                            (4, __tok0) => __Symbol::Term_22_2e_22(__tok0),
                            _ => unreachable!(),
                        },
                        5 => match __lookahead.1 {
                            (5, __tok0) => __Symbol::Term_22_3a_2d_22(__tok0),
                            _ => unreachable!(),
                        },
                        6 => match __lookahead.1 {
                            (6, __tok0) => __Symbol::Term_22_3b_22(__tok0),
                            _ => unreachable!(),
                        },
                        7 => match __lookahead.1 {
                            (7, __tok0) => __Symbol::Term_22_3d_3e_22(__tok0),
                            _ => unreachable!(),
                        },
                        8 => match __lookahead.1 {
                            (8, __tok0) => __Symbol::Term_22___22(__tok0),
                            _ => unreachable!(),
                        },
                        9 => match __lookahead.1 {
                            (9, __tok0) => __Symbol::Term_22exists_22(__tok0),
                            _ => unreachable!(),
                        },
                        10 => match __lookahead.1 {
                            (10, __tok0) => __Symbol::Term_22forall_22(__tok0),
                            _ => unreachable!(),
                        },
                        11 => match __lookahead.1 {
                            (11, __tok0) => __Symbol::Termr_23_22_5c_27_5b_5e_5c_27_5d_2b_5c_27_22_23(__tok0),
                            _ => unreachable!(),
                        },
                        12 => match __lookahead.1 {
                            (12, __tok0) => __Symbol::Termr_23_22_5b_2d_7c_21_40_23_24_25_5e_26_2a_3d_2b_2f_3a_3f_7e_3c_3e_5d_2b_22_23(__tok0),
                            _ => unreachable!(),
                        },
                        13 => match __lookahead.1 {
                            (13, __tok0) => __Symbol::Termr_23_22_5bA_2dZa_2dz_5d_5bA_2dZa_2dz0_2d9___5d_2a_22_23(__tok0),
                            _ => unreachable!(),
                        },
                        14 => match __lookahead.1 {
                            (14, __tok0) => __Symbol::Termr_23_22_5bA_2dZa_2dz_5d_5bA_2dZa_2dz0_2d9___5d_2a_3a_22_23(__tok0),
                            _ => unreachable!(),
                        },
                        _ => unreachable!(),
                    };
                    __states.push(__action - 1);
                    __symbols.push((__lookahead.0, __symbol, __lookahead.2));
                    continue '__shift;
                } else if __action < 0 {
                    if let Some(r) = __reduce(input, __action, Some(&__lookahead.0), &mut __states, &mut __symbols, ::std::marker::PhantomData::<()>) {
                        return r;
                    }
                } else {
                    return Err(__lalrpop_util::ParseError::UnrecognizedToken {
                        token: Some(__lookahead),
                        expected: vec![],
                    });
                }
            }
        }
        loop {
            let __state = *__states.last().unwrap() as usize;
            let __action = __EOF_ACTION[__state];
            if __action < 0 {
                if let Some(r) = __reduce(input, __action, None, &mut __states, &mut __symbols, ::std::marker::PhantomData::<()>) {
                    return r;
                }
            } else {
                return Err(__lalrpop_util::ParseError::UnrecognizedToken {
                    token: None,
                    expected: vec![],
                });
            }
        }
    }
    pub fn __reduce<
        'input,
    >(
        input: &'input str,
        __action: i32,
        __lookahead_start: Option<&usize>,
        __states: &mut ::std::vec::Vec<i32>,
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: ::std::marker::PhantomData<()>,
    ) -> Option<Result<Program,__lalrpop_util::ParseError<usize,(usize, &'input str),()>>>
    {
        let __nonterminal = match -__action {
            1 => {
                // (Operator Value) = Operator, Value => ActionFn(36);
                let __sym1 = __pop_NtValue(__symbols);
                let __sym0 = __pop_NtOperator(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = super::__action36::<>(input, __sym0, __sym1);
                let __states_len = __states.len();
                __states.truncate(__states_len - 2);
                __symbols.push((__start, __Symbol::Nt_28Operator_20Value_29(__nt), __end));
                0
            }
            2 => {
                // (Operator Value)+ = Operator, Value => ActionFn(41);
                let __sym1 = __pop_NtValue(__symbols);
                let __sym0 = __pop_NtOperator(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = super::__action41::<>(input, __sym0, __sym1);
                let __states_len = __states.len();
                __states.truncate(__states_len - 2);
                __symbols.push((__start, __Symbol::Nt_28Operator_20Value_29_2b(__nt), __end));
                1
            }
            3 => {
                // (Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);
                let __sym2 = __pop_NtValue(__symbols);
                let __sym1 = __pop_NtOperator(__symbols);
                let __sym0 = __pop_Nt_28Operator_20Value_29_2b(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = super::__action42::<>(input, __sym0, __sym1, __sym2);
                let __states_len = __states.len();
                __states.truncate(__states_len - 3);
                __symbols.push((__start, __Symbol::Nt_28Operator_20Value_29_2b(__nt), __end));
                1
            }
            4 => {
                // Application = Value => ActionFn(15);
                let __sym0 = __pop_NtValue(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action15::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtApplication(__nt), __end));
                2
            }
            5 => {
                // Application = Operator => ActionFn(16);
                let __sym0 = __pop_NtOperator(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action16::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtApplication(__nt), __end));
                2
            }
            6 => {
                // Application = Value, (Operator Value)+, Operator => ActionFn(45);
                let __sym2 = __pop_NtOperator(__symbols);
                let __sym1 = __pop_Nt_28Operator_20Value_29_2b(__symbols);
                let __sym0 = __pop_NtValue(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = super::__action45::<>(input, __sym0, __sym1, __sym2);
                let __states_len = __states.len();
                __states.truncate(__states_len - 3);
                __symbols.push((__start, __Symbol::NtApplication(__nt), __end));
                2
            }
            7 => {
                // Application = (Operator Value)+, Operator => ActionFn(46);
                let __sym1 = __pop_NtOperator(__symbols);
                let __sym0 = __pop_Nt_28Operator_20Value_29_2b(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = super::__action46::<>(input, __sym0, __sym1);
                let __states_len = __states.len();
                __states.truncate(__states_len - 2);
                __symbols.push((__start, __Symbol::NtApplication(__nt), __end));
                2
            }
            8 => {
                // Application = Value, (Operator Value)+ => ActionFn(47);
                let __sym1 = __pop_Nt_28Operator_20Value_29_2b(__symbols);
                let __sym0 = __pop_NtValue(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = super::__action47::<>(input, __sym0, __sym1);
                let __states_len = __states.len();
                __states.truncate(__states_len - 2);
                __symbols.push((__start, __Symbol::NtApplication(__nt), __end));
                2
            }
            9 => {
                // Application = (Operator Value)+ => ActionFn(48);
                let __sym0 = __pop_Nt_28Operator_20Value_29_2b(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action48::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtApplication(__nt), __end));
                2
            }
            10 => {
                // Application = Identifier, "(", Applications, ")" => ActionFn(18);
                let __sym3 = __pop_Term_22_29_22(__symbols);
                let __sym2 = __pop_NtApplications(__symbols);
                let __sym1 = __pop_Term_22_28_22(__symbols);
                let __sym0 = __pop_NtIdentifier(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = super::__action18::<>(input, __sym0, __sym1, __sym2, __sym3);
                let __states_len = __states.len();
                __states.truncate(__states_len - 4);
                __symbols.push((__start, __Symbol::NtApplication(__nt), __end));
                2
            }
            11 => {
                // Applications = Application => ActionFn(19);
                let __sym0 = __pop_NtApplication(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action19::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtApplications(__nt), __end));
                3
            }
            12 => {
                // Applications = Applications, ",", Application => ActionFn(20);
                let __sym2 = __pop_NtApplication(__symbols);
                let __sym1 = __pop_Term_22_2c_22(__symbols);
                let __sym0 = __pop_NtApplications(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = super::__action20::<>(input, __sym0, __sym1, __sym2);
                let __states_len = __states.len();
                __states.truncate(__states_len - 3);
                __symbols.push((__start, __Symbol::NtApplications(__nt), __end));
                3
            }
            13 => {
                // Applications = Applications, "," => ActionFn(21);
                let __sym1 = __pop_Term_22_2c_22(__symbols);
                let __sym0 = __pop_NtApplications(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = super::__action21::<>(input, __sym0, __sym1);
                let __states_len = __states.len();
                __states.truncate(__states_len - 2);
                __symbols.push((__start, __Symbol::NtApplications(__nt), __end));
                3
            }
            14 => {
                // Atom = r#"\'[^\']+\'"# => ActionFn(29);
                let __sym0 = __pop_Termr_23_22_5c_27_5b_5e_5c_27_5d_2b_5c_27_22_23(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action29::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtAtom(__nt), __end));
                4
            }
            15 => {
                // Fact = FactAnd => ActionFn(5);
                let __sym0 = __pop_NtFactAnd(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action5::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtFact(__nt), __end));
                5
            }
            16 => {
                // FactAnd = FactOr => ActionFn(6);
                let __sym0 = __pop_NtFactOr(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action6::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtFactAnd(__nt), __end));
                6
            }
            17 => {
                // FactAnd = FactAnd, ";", FactOr => ActionFn(7);
                let __sym2 = __pop_NtFactOr(__symbols);
                let __sym1 = __pop_Term_22_3b_22(__symbols);
                let __sym0 = __pop_NtFactAnd(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = super::__action7::<>(input, __sym0, __sym1, __sym2);
                let __states_len = __states.len();
                __states.truncate(__states_len - 3);
                __symbols.push((__start, __Symbol::NtFactAnd(__nt), __end));
                6
            }
            18 => {
                // FactApply = Application => ActionFn(14);
                let __sym0 = __pop_NtApplication(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action14::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtFactApply(__nt), __end));
                7
            }
            19 => {
                // FactFunc = FactApply => ActionFn(10);
                let __sym0 = __pop_NtFactApply(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action10::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtFactFunc(__nt), __end));
                8
            }
            20 => {
                // FactFunc = FactApply, "=>", FactFunc => ActionFn(11);
                let __sym2 = __pop_NtFactFunc(__symbols);
                let __sym1 = __pop_Term_22_3d_3e_22(__symbols);
                let __sym0 = __pop_NtFactApply(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = super::__action11::<>(input, __sym0, __sym1, __sym2);
                let __states_len = __states.len();
                __states.truncate(__states_len - 3);
                __symbols.push((__start, __Symbol::NtFactFunc(__nt), __end));
                8
            }
            21 => {
                // FactFunc = "exists", "(", Variable, "->", FactFunc, ")" => ActionFn(12);
                let __sym5 = __pop_Term_22_29_22(__symbols);
                let __sym4 = __pop_NtFactFunc(__symbols);
                let __sym3 = __pop_Term_22_2d_3e_22(__symbols);
                let __sym2 = __pop_NtVariable(__symbols);
                let __sym1 = __pop_Term_22_28_22(__symbols);
                let __sym0 = __pop_Term_22exists_22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = super::__action12::<>(input, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5);
                let __states_len = __states.len();
                __states.truncate(__states_len - 6);
                __symbols.push((__start, __Symbol::NtFactFunc(__nt), __end));
                8
            }
            22 => {
                // FactFunc = "forall", "(", Variable, "->", FactFunc, ")" => ActionFn(13);
                let __sym5 = __pop_Term_22_29_22(__symbols);
                let __sym4 = __pop_NtFactFunc(__symbols);
                let __sym3 = __pop_Term_22_2d_3e_22(__symbols);
                let __sym2 = __pop_NtVariable(__symbols);
                let __sym1 = __pop_Term_22_28_22(__symbols);
                let __sym0 = __pop_Term_22forall_22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym5.2.clone();
                let __nt = super::__action13::<>(input, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5);
                let __states_len = __states.len();
                __states.truncate(__states_len - 6);
                __symbols.push((__start, __Symbol::NtFactFunc(__nt), __end));
                8
            }
            23 => {
                // FactOr = FactFunc => ActionFn(8);
                let __sym0 = __pop_NtFactFunc(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action8::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtFactOr(__nt), __end));
                9
            }
            24 => {
                // FactOr = FactOr, ",", FactFunc => ActionFn(9);
                let __sym2 = __pop_NtFactFunc(__symbols);
                let __sym1 = __pop_Term_22_2c_22(__symbols);
                let __sym0 = __pop_NtFactOr(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = super::__action9::<>(input, __sym0, __sym1, __sym2);
                let __states_len = __states.len();
                __states.truncate(__states_len - 3);
                __symbols.push((__start, __Symbol::NtFactOr(__nt), __end));
                9
            }
            25 => {
                // Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);
                let __sym0 = __pop_Termr_23_22_5bA_2dZa_2dz_5d_5bA_2dZa_2dz0_2d9___5d_2a_22_23(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action31::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtIdentifier(__nt), __end));
                10
            }
            26 => {
                // Item = Application, "." => ActionFn(2);
                let __sym1 = __pop_Term_22_2e_22(__symbols);
                let __sym0 = __pop_NtApplication(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = super::__action2::<>(input, __sym0, __sym1);
                let __states_len = __states.len();
                __states.truncate(__states_len - 2);
                __symbols.push((__start, __Symbol::NtItem(__nt), __end));
                11
            }
            27 => {
                // Item = Rule => ActionFn(3);
                let __sym0 = __pop_NtRule(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action3::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtItem(__nt), __end));
                11
            }
            28 => {
                // Item+ = Item => ActionFn(39);
                let __sym0 = __pop_NtItem(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action39::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtItem_2b(__nt), __end));
                12
            }
            29 => {
                // Item+ = Item+, Item => ActionFn(40);
                let __sym1 = __pop_NtItem(__symbols);
                let __sym0 = __pop_NtItem_2b(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = super::__action40::<>(input, __sym0, __sym1);
                let __states_len = __states.len();
                __states.truncate(__states_len - 2);
                __symbols.push((__start, __Symbol::NtItem_2b(__nt), __end));
                12
            }
            30 => {
                // Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(23);
                let __sym0 = __pop_Termr_23_22_5bA_2dZa_2dz_5d_5bA_2dZa_2dz0_2d9___5d_2a_3a_22_23(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action23::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtOperator(__nt), __end));
                13
            }
            31 => {
                // Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(24);
                let __sym0 = __pop_Termr_23_22_5b_2d_7c_21_40_23_24_25_5e_26_2a_3d_2b_2f_3a_3f_7e_3c_3e_5d_2b_22_23(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action24::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtOperator(__nt), __end));
                13
            }
            32 => {
                // Operator? = Operator => ActionFn(32);
                let __sym0 = __pop_NtOperator(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action32::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtOperator_3f(__nt), __end));
                14
            }
            33 => {
                // Operator? =  => ActionFn(33);
                let __start = __symbols.last().map(|s| s.2.clone()).unwrap_or_default();
                let __end = __lookahead_start.cloned().unwrap_or_else(|| __start.clone());
                let __nt = super::__action33::<>(input, &__start, &__end);
                let __states_len = __states.len();
                __states.truncate(__states_len - 0);
                __symbols.push((__start, __Symbol::NtOperator_3f(__nt), __end));
                14
            }
            34 => {
                // OperatorValue = Operator, Value => ActionFn(22);
                let __sym1 = __pop_NtValue(__symbols);
                let __sym0 = __pop_NtOperator(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = super::__action22::<>(input, __sym0, __sym1);
                let __states_len = __states.len();
                __states.truncate(__states_len - 2);
                __symbols.push((__start, __Symbol::NtOperatorValue(__nt), __end));
                15
            }
            35 => {
                // Program = Item+ => ActionFn(1);
                let __sym0 = __pop_NtItem_2b(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action1::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtProgram(__nt), __end));
                16
            }
            36 => {
                // Rule = Application, ":-", Fact, "." => ActionFn(4);
                let __sym3 = __pop_Term_22_2e_22(__symbols);
                let __sym2 = __pop_NtFact(__symbols);
                let __sym1 = __pop_Term_22_3a_2d_22(__symbols);
                let __sym0 = __pop_NtApplication(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = super::__action4::<>(input, __sym0, __sym1, __sym2, __sym3);
                let __states_len = __states.len();
                __states.truncate(__states_len - 4);
                __symbols.push((__start, __Symbol::NtRule(__nt), __end));
                17
            }
            37 => {
                // Value = Atom => ActionFn(25);
                let __sym0 = __pop_NtAtom(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action25::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtValue(__nt), __end));
                18
            }
            38 => {
                // Value = Variable => ActionFn(26);
                let __sym0 = __pop_NtVariable(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action26::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtValue(__nt), __end));
                18
            }
            39 => {
                // Value = "(", Application, ")" => ActionFn(27);
                let __sym2 = __pop_Term_22_29_22(__symbols);
                let __sym1 = __pop_NtApplication(__symbols);
                let __sym0 = __pop_Term_22_28_22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = super::__action27::<>(input, __sym0, __sym1, __sym2);
                let __states_len = __states.len();
                __states.truncate(__states_len - 3);
                __symbols.push((__start, __Symbol::NtValue(__nt), __end));
                18
            }
            40 => {
                // Value = "_" => ActionFn(28);
                let __sym0 = __pop_Term_22___22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action28::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtValue(__nt), __end));
                18
            }
            41 => {
                // Value? = Value => ActionFn(37);
                let __sym0 = __pop_NtValue(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action37::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtValue_3f(__nt), __end));
                19
            }
            42 => {
                // Value? =  => ActionFn(38);
                let __start = __symbols.last().map(|s| s.2.clone()).unwrap_or_default();
                let __end = __lookahead_start.cloned().unwrap_or_else(|| __start.clone());
                let __nt = super::__action38::<>(input, &__start, &__end);
                let __states_len = __states.len();
                __states.truncate(__states_len - 0);
                __symbols.push((__start, __Symbol::NtValue_3f(__nt), __end));
                19
            }
            43 => {
                // Variable = Identifier => ActionFn(30);
                let __sym0 = __pop_NtIdentifier(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action30::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtVariable(__nt), __end));
                20
            }
            44 => {
                // __Program = Program => ActionFn(0);
                let __sym0 = __pop_NtProgram(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action0::<>(input, __sym0);
                return Some(Ok(__nt));
            }
            _ => panic!("invalid action code {}", __action)
        };
        let __state = *__states.last().unwrap() as usize;
        let __next_state = __GOTO[__state * 22 + __nonterminal] - 1;
        __states.push(__next_state);
        None
    }
    fn __pop_Term_22_28_22<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, &'input str, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Term_22_28_22(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Term_22_29_22<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, &'input str, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Term_22_29_22(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Term_22_2c_22<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, &'input str, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Term_22_2c_22(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Term_22_2d_3e_22<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, &'input str, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Term_22_2d_3e_22(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Term_22_2e_22<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, &'input str, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Term_22_2e_22(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Term_22_3a_2d_22<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, &'input str, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Term_22_3a_2d_22(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Term_22_3b_22<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, &'input str, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Term_22_3b_22(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Term_22_3d_3e_22<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, &'input str, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Term_22_3d_3e_22(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Term_22___22<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, &'input str, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Term_22___22(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Term_22exists_22<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, &'input str, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Term_22exists_22(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Term_22forall_22<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, &'input str, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Term_22forall_22(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Termr_23_22_5c_27_5b_5e_5c_27_5d_2b_5c_27_22_23<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, &'input str, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Termr_23_22_5c_27_5b_5e_5c_27_5d_2b_5c_27_22_23(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Termr_23_22_5b_2d_7c_21_40_23_24_25_5e_26_2a_3d_2b_2f_3a_3f_7e_3c_3e_5d_2b_22_23<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, &'input str, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Termr_23_22_5b_2d_7c_21_40_23_24_25_5e_26_2a_3d_2b_2f_3a_3f_7e_3c_3e_5d_2b_22_23(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Termr_23_22_5bA_2dZa_2dz_5d_5bA_2dZa_2dz0_2d9___5d_2a_22_23<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, &'input str, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Termr_23_22_5bA_2dZa_2dz_5d_5bA_2dZa_2dz0_2d9___5d_2a_22_23(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Termr_23_22_5bA_2dZa_2dz_5d_5bA_2dZa_2dz0_2d9___5d_2a_3a_22_23<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, &'input str, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Termr_23_22_5bA_2dZa_2dz_5d_5bA_2dZa_2dz0_2d9___5d_2a_3a_22_23(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Nt_28Operator_20Value_29<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, (Operator, Value), usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Nt_28Operator_20Value_29(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Nt_28Operator_20Value_29_2b<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, ::std::vec::Vec<(Operator, Value)>, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Nt_28Operator_20Value_29_2b(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtApplication<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Application, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtApplication(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtApplications<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Vec<Application>, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtApplications(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtAtom<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Atom, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtAtom(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtFact<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Fact, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtFact(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtFactAnd<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Fact, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtFactAnd(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtFactApply<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Fact, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtFactApply(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtFactFunc<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Fact, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtFactFunc(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtFactOr<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Fact, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtFactOr(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtIdentifier<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, InternedString, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtIdentifier(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtItem<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Item, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtItem(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtItem_2b<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, ::std::vec::Vec<Item>, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtItem_2b(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtOperator<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Operator, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtOperator(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtOperator_3f<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, ::std::option::Option<Operator>, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtOperator_3f(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtOperatorValue<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, (Operator, Value), usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtOperatorValue(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtProgram<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Program, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtProgram(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtRule<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Rule, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtRule(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtValue<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Value, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtValue(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtValue_3f<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, ::std::option::Option<Value>, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtValue_3f(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtVariable<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Variable, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtVariable(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Nt____Program<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Program, usize) {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Nt____Program(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
}
pub use self::__parse__Program::parse_Program;
mod __intern_token {
    extern crate lalrpop_util as __lalrpop_util;
    pub struct __Matcher<'input> {
        text: &'input str,
        consumed: usize,
    }

    fn __tokenize(text: &str) -> Option<(usize, usize)> {
        let mut __chars = text.char_indices();
        let mut __current_match: Option<(usize, usize)> = None;
        let mut __current_state: usize = 0;
        loop {
            match __current_state {
                0 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        33 => /* '!' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        35 ... 38 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 1;
                            continue;
                        }
                        39 => /* '\'' */ {
                            __current_state = 2;
                            continue;
                        }
                        40 => /* '(' */ {
                            __current_match = Some((0, __index + 1));
                            __current_state = 3;
                            continue;
                        }
                        41 => /* ')' */ {
                            __current_match = Some((1, __index + 1));
                            __current_state = 4;
                            continue;
                        }
                        42 ... 43 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 1;
                            continue;
                        }
                        44 => /* ',' */ {
                            __current_match = Some((2, __index + 1));
                            __current_state = 5;
                            continue;
                        }
                        45 => /* '-' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 6;
                            continue;
                        }
                        46 => /* '.' */ {
                            __current_match = Some((4, __index + 1));
                            __current_state = 7;
                            continue;
                        }
                        47 => /* '/' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 8;
                            continue;
                        }
                        59 => /* ';' */ {
                            __current_match = Some((6, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        60 => /* '<' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        61 => /* '=' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 10;
                            continue;
                        }
                        62 ... 64 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 1;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 11;
                            continue;
                        }
                        94 => /* '^' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((8, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        97 ... 100 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 11;
                            continue;
                        }
                        101 => /* 'e' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 13;
                            continue;
                        }
                        102 => /* 'f' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 14;
                            continue;
                        }
                        103 ... 122 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 11;
                            continue;
                        }
                        124 => /* '|' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        126 => /* '~' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                1 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        33 => /* '!' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        35 ... 38 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 16;
                            continue;
                        }
                        42 ... 43 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 16;
                            continue;
                        }
                        45 => /* '-' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        47 => /* '/' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        60 ... 64 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 16;
                            continue;
                        }
                        94 => /* '^' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        124 => /* '|' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        126 => /* '~' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                2 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        0 ... 38 => {
                            __current_state = 17;
                            continue;
                        }
                        40 ... 1114111 => {
                            __current_state = 17;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                3 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        _ => {
                            return __current_match;
                        }
                    }
                }
                4 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        _ => {
                            return __current_match;
                        }
                    }
                }
                5 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        _ => {
                            return __current_match;
                        }
                    }
                }
                6 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        33 => /* '!' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        35 ... 38 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 16;
                            continue;
                        }
                        42 ... 43 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 16;
                            continue;
                        }
                        45 => /* '-' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        47 => /* '/' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        60 ... 61 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 16;
                            continue;
                        }
                        62 => /* '>' */ {
                            __current_match = Some((3, __index + 1));
                            __current_state = 18;
                            continue;
                        }
                        63 ... 64 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 16;
                            continue;
                        }
                        94 => /* '^' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        124 => /* '|' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        126 => /* '~' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                7 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        _ => {
                            return __current_match;
                        }
                    }
                }
                8 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        33 => /* '!' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        35 ... 38 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 16;
                            continue;
                        }
                        42 ... 43 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 16;
                            continue;
                        }
                        45 => /* '-' */ {
                            __current_match = Some((5, __index + 1));
                            __current_state = 19;
                            continue;
                        }
                        47 => /* '/' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        60 ... 64 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 16;
                            continue;
                        }
                        94 => /* '^' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        124 => /* '|' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        126 => /* '~' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                9 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        _ => {
                            return __current_match;
                        }
                    }
                }
                10 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        33 => /* '!' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        35 ... 38 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 16;
                            continue;
                        }
                        42 ... 43 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 16;
                            continue;
                        }
                        45 => /* '-' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        47 => /* '/' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        60 ... 61 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 16;
                            continue;
                        }
                        62 => /* '>' */ {
                            __current_match = Some((7, __index + 1));
                            __current_state = 20;
                            continue;
                        }
                        63 ... 64 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 16;
                            continue;
                        }
                        94 => /* '^' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        124 => /* '|' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        126 => /* '~' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                11 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        48 ... 57 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((14, __index + 1));
                            __current_state = 22;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        97 ... 122 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                12 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        _ => {
                            return __current_match;
                        }
                    }
                }
                13 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        48 ... 57 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((14, __index + 1));
                            __current_state = 22;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        97 ... 119 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        120 => /* 'x' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 23;
                            continue;
                        }
                        121 ... 122 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                14 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        48 ... 57 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((14, __index + 1));
                            __current_state = 22;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        97 ... 110 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        111 => /* 'o' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 24;
                            continue;
                        }
                        112 ... 122 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                15 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        _ => {
                            return __current_match;
                        }
                    }
                }
                16 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        33 => /* '!' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        35 ... 38 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 16;
                            continue;
                        }
                        42 ... 43 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 16;
                            continue;
                        }
                        45 => /* '-' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        47 => /* '/' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        60 ... 64 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 16;
                            continue;
                        }
                        94 => /* '^' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        124 => /* '|' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        126 => /* '~' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                17 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        0 ... 38 => {
                            __current_state = 25;
                            continue;
                        }
                        39 => /* '\'' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 26;
                            continue;
                        }
                        40 ... 1114111 => {
                            __current_state = 25;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                18 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        33 => /* '!' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        35 ... 38 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 16;
                            continue;
                        }
                        42 ... 43 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 16;
                            continue;
                        }
                        45 => /* '-' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        47 => /* '/' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        60 ... 64 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 16;
                            continue;
                        }
                        94 => /* '^' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        124 => /* '|' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        126 => /* '~' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                19 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        33 => /* '!' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        35 ... 38 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 16;
                            continue;
                        }
                        42 ... 43 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 16;
                            continue;
                        }
                        45 => /* '-' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        47 => /* '/' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        60 ... 64 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 16;
                            continue;
                        }
                        94 => /* '^' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        124 => /* '|' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        126 => /* '~' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                20 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        33 => /* '!' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        35 ... 38 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 16;
                            continue;
                        }
                        42 ... 43 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 16;
                            continue;
                        }
                        45 => /* '-' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        47 => /* '/' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        60 ... 64 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 16;
                            continue;
                        }
                        94 => /* '^' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        124 => /* '|' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        126 => /* '~' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                21 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        48 ... 57 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((14, __index + 1));
                            __current_state = 22;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        97 ... 122 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                22 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        _ => {
                            return __current_match;
                        }
                    }
                }
                23 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        48 ... 57 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((14, __index + 1));
                            __current_state = 22;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        97 ... 104 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        105 => /* 'i' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 27;
                            continue;
                        }
                        106 ... 122 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                24 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        48 ... 57 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((14, __index + 1));
                            __current_state = 22;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        97 ... 113 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        114 => /* 'r' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 28;
                            continue;
                        }
                        115 ... 122 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                25 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        0 ... 38 => {
                            __current_state = 25;
                            continue;
                        }
                        39 => /* '\'' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 26;
                            continue;
                        }
                        40 ... 1114111 => {
                            __current_state = 25;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                26 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        _ => {
                            return __current_match;
                        }
                    }
                }
                27 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        48 ... 57 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((14, __index + 1));
                            __current_state = 22;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        97 ... 114 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        115 => /* 's' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 29;
                            continue;
                        }
                        116 ... 122 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                28 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        48 ... 57 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((14, __index + 1));
                            __current_state = 22;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        97 => /* 'a' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 30;
                            continue;
                        }
                        98 ... 122 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                29 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        48 ... 57 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((14, __index + 1));
                            __current_state = 22;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        97 ... 115 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        116 => /* 't' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 31;
                            continue;
                        }
                        117 ... 122 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                30 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        48 ... 57 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((14, __index + 1));
                            __current_state = 22;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        97 ... 107 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        108 => /* 'l' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 32;
                            continue;
                        }
                        109 ... 122 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                31 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        48 ... 57 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((14, __index + 1));
                            __current_state = 22;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        97 ... 114 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        115 => /* 's' */ {
                            __current_match = Some((9, __index + 1));
                            __current_state = 33;
                            continue;
                        }
                        116 ... 122 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                32 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        48 ... 57 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((14, __index + 1));
                            __current_state = 22;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        97 ... 107 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        108 => /* 'l' */ {
                            __current_match = Some((10, __index + 1));
                            __current_state = 34;
                            continue;
                        }
                        109 ... 122 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                33 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        48 ... 57 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((14, __index + 1));
                            __current_state = 22;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        97 ... 122 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                34 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        48 ... 57 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((14, __index + 1));
                            __current_state = 22;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        97 ... 122 => {
                            __current_match = Some((13, __index + __ch.len_utf8()));
                            __current_state = 21;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                _ => { panic!("invalid state {}", __current_state); }
            }
        }
    }

    impl<'input> __Matcher<'input> {
        pub fn new(s: &'input str) -> __Matcher<'input> {
            __Matcher { text: s, consumed: 0 }
        }
    }

    impl<'input> Iterator for __Matcher<'input> {
        type Item = Result<(usize, (usize, &'input str), usize), __lalrpop_util::ParseError<usize,(usize, &'input str),()>>;

        fn next(&mut self) -> Option<Self::Item> {
            let __text = self.text.trim_left();
            let __whitespace = self.text.len() - __text.len();
            let __start_offset = self.consumed + __whitespace;
            if __text.is_empty() {
                self.text = __text;
                self.consumed = __start_offset;
                None
            } else {
                match __tokenize(__text) {
                    Some((__index, __length)) => {
                        let __result = &__text[..__length];
                        let __remaining = &__text[__length..];
                        let __end_offset = __start_offset + __length;
                        self.text = __remaining;
                        self.consumed = __end_offset;
                        Some(Ok((__start_offset, (__index, __result), __end_offset)))
                    }
                    None => {
                        Some(Err(__lalrpop_util::ParseError::InvalidToken { location: __start_offset }))
                    }
                }
            }
        }
    }
}

#[allow(unused_variables)]
pub fn __action0<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, Program, usize),
) -> Program
{
    (__0)
}

#[allow(unused_variables)]
pub fn __action1<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, ::std::vec::Vec<Item>, usize),
) -> Program
{
    Program { items: __0 }
}

#[allow(unused_variables)]
pub fn __action2<
    'input,
>(
    input: &'input str,
    (_, a, _): (usize, Application, usize),
    (_, _, _): (usize, &'input str, usize),
) -> Item
{
    Item::Fact(a)
}

#[allow(unused_variables)]
pub fn __action3<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, Rule, usize),
) -> Item
{
    Item::Rule(__0)
}

#[allow(unused_variables)]
pub fn __action4<
    'input,
>(
    input: &'input str,
    (_, a, _): (usize, Application, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, f, _): (usize, Fact, usize),
    (_, _, _): (usize, &'input str, usize),
) -> Rule
{
    Rule {
        consequence: a,
        condition: f
    }
}

#[allow(unused_variables)]
pub fn __action5<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, Fact, usize),
) -> Fact
{
    (__0)
}

#[allow(unused_variables)]
pub fn __action6<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, Fact, usize),
) -> Fact
{
    (__0)
}

#[allow(unused_variables)]
pub fn __action7<
    'input,
>(
    input: &'input str,
    (_, l, _): (usize, Fact, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, r, _): (usize, Fact, usize),
) -> Fact
{
    Fact { data: Box::new(FactData::And(l, r)) }
}

#[allow(unused_variables)]
pub fn __action8<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, Fact, usize),
) -> Fact
{
    (__0)
}

#[allow(unused_variables)]
pub fn __action9<
    'input,
>(
    input: &'input str,
    (_, l, _): (usize, Fact, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, r, _): (usize, Fact, usize),
) -> Fact
{
    Fact { data: Box::new(FactData::Or(l, r)) }
}

#[allow(unused_variables)]
pub fn __action10<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, Fact, usize),
) -> Fact
{
    (__0)
}

#[allow(unused_variables)]
pub fn __action11<
    'input,
>(
    input: &'input str,
    (_, l, _): (usize, Fact, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, r, _): (usize, Fact, usize),
) -> Fact
{
    Fact { data: Box::new(FactData::Implication(l, r)) }
}

#[allow(unused_variables)]
pub fn __action12<
    'input,
>(
    input: &'input str,
    (_, _, _): (usize, &'input str, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, v, _): (usize, Variable, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, b, _): (usize, Fact, usize),
    (_, _, _): (usize, &'input str, usize),
) -> Fact
{
    Fact { data: Box::new(FactData::Exists(v, b)) }
}

#[allow(unused_variables)]
pub fn __action13<
    'input,
>(
    input: &'input str,
    (_, _, _): (usize, &'input str, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, v, _): (usize, Variable, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, b, _): (usize, Fact, usize),
    (_, _, _): (usize, &'input str, usize),
) -> Fact
{
    Fact { data: Box::new(FactData::ForAll(v, b)) }
}

#[allow(unused_variables)]
pub fn __action14<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, Application, usize),
) -> Fact
{
    Fact { data: Box::new(FactData::Apply(__0)) }
}

#[allow(unused_variables)]
pub fn __action15<
    'input,
>(
    input: &'input str,
    (_, head, _): (usize, Value, usize),
) -> Application
{
    {
        Application {
            bits: vec![Bit::Value(head)]
        }
    }
}

#[allow(unused_variables)]
pub fn __action16<
    'input,
>(
    input: &'input str,
    (_, tail, _): (usize, Operator, usize),
) -> Application
{
    {
        Application {
            bits: vec![Bit::Operator(tail)]
        }
    }
}

#[allow(unused_variables)]
pub fn __action17<
    'input,
>(
    input: &'input str,
    (_, head, _): (usize, ::std::option::Option<Value>, usize),
    (_, body, _): (usize, ::std::vec::Vec<(Operator, Value)>, usize),
    (_, tail, _): (usize, ::std::option::Option<Operator>, usize),
) -> Application
{
    {
        Application {
            bits: head.into_iter().map(Bit::Value)
                                  .chain(body.into_iter().flat_map(|(o, v)| {
                                      once(Bit::Operator(o)).chain(once(Bit::Value(v)))
                                  }))
                                  .chain(tail.into_iter().map(Bit::Operator))
                                  .collect()
        }
    }
}

#[allow(unused_variables)]
pub fn __action18<
    'input,
>(
    input: &'input str,
    (_, id, _): (usize, InternedString, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, appls, _): (usize, Vec<Application>, usize),
    (_, _, _): (usize, &'input str, usize),
) -> Application
{
    {
        let oper_bit = Bit::Operator(Operator::Parens(id));
        let appl_bits = appls.into_iter().map(Value::Application).map(Bit::Value);
        Application {
            bits: Some(oper_bit).into_iter().chain(appl_bits).collect()
        }
    }
}

#[allow(unused_variables)]
pub fn __action19<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, Application, usize),
) -> Vec<Application>
{
    vec![__0]
}

#[allow(unused_variables)]
pub fn __action20<
    'input,
>(
    input: &'input str,
    (_, appls, _): (usize, Vec<Application>, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, appl, _): (usize, Application, usize),
) -> Vec<Application>
{
    {
        let mut appls = appls;
        appls.push(appl);
        appls
    }
}

#[allow(unused_variables)]
pub fn __action21<
    'input,
>(
    input: &'input str,
    (_, appls, _): (usize, Vec<Application>, usize),
    (_, _, _): (usize, &'input str, usize),
) -> Vec<Application>
{
    appls
}

#[allow(unused_variables)]
pub fn __action22<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, Operator, usize),
    (_, __1, _): (usize, Value, usize),
) -> (Operator, Value)
{
    (__0, __1)
}

#[allow(unused_variables)]
pub fn __action23<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> Operator
{
    Operator::Colon(intern(__0))
}

#[allow(unused_variables)]
pub fn __action24<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> Operator
{
    Operator::Symbols(intern(__0))
}

#[allow(unused_variables)]
pub fn __action25<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, Atom, usize),
) -> Value
{
    Value::Atom(__0)
}

#[allow(unused_variables)]
pub fn __action26<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, Variable, usize),
) -> Value
{
    Value::Variable(__0)
}

#[allow(unused_variables)]
pub fn __action27<
    'input,
>(
    input: &'input str,
    (_, _, _): (usize, &'input str, usize),
    (_, __0, _): (usize, Application, usize),
    (_, _, _): (usize, &'input str, usize),
) -> Value
{
    Value::Application(__0)
}

#[allow(unused_variables)]
pub fn __action28<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> Value
{
    Value::Wildcard
}

#[allow(unused_variables)]
pub fn __action29<
    'input,
>(
    input: &'input str,
    (_, s, _): (usize, &'input str, usize),
) -> Atom
{
    Atom { id: intern(&s[1..s.len() - 1]) }
}

#[allow(unused_variables)]
pub fn __action30<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, InternedString, usize),
) -> Variable
{
    Variable { id: __0 }
}

#[allow(unused_variables)]
pub fn __action31<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> InternedString
{
    intern(__0)
}

#[allow(unused_variables)]
pub fn __action32<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, Operator, usize),
) -> ::std::option::Option<Operator>
{
    Some(__0)
}

#[allow(unused_variables)]
pub fn __action33<
    'input,
>(
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> ::std::option::Option<Operator>
{
    None
}

#[allow(unused_variables)]
pub fn __action34<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, (Operator, Value), usize),
) -> ::std::vec::Vec<(Operator, Value)>
{
    vec![__0]
}

#[allow(unused_variables)]
pub fn __action35<
    'input,
>(
    input: &'input str,
    (_, v, _): (usize, ::std::vec::Vec<(Operator, Value)>, usize),
    (_, e, _): (usize, (Operator, Value), usize),
) -> ::std::vec::Vec<(Operator, Value)>
{
    { let mut v = v; v.push(e); v }
}

#[allow(unused_variables)]
pub fn __action36<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, Operator, usize),
    (_, __1, _): (usize, Value, usize),
) -> (Operator, Value)
{
    (__0, __1)
}

#[allow(unused_variables)]
pub fn __action37<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, Value, usize),
) -> ::std::option::Option<Value>
{
    Some(__0)
}

#[allow(unused_variables)]
pub fn __action38<
    'input,
>(
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> ::std::option::Option<Value>
{
    None
}

#[allow(unused_variables)]
pub fn __action39<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, Item, usize),
) -> ::std::vec::Vec<Item>
{
    vec![__0]
}

#[allow(unused_variables)]
pub fn __action40<
    'input,
>(
    input: &'input str,
    (_, v, _): (usize, ::std::vec::Vec<Item>, usize),
    (_, e, _): (usize, Item, usize),
) -> ::std::vec::Vec<Item>
{
    { let mut v = v; v.push(e); v }
}

#[allow(unused_variables)]
pub fn __action41<
    'input,
>(
    input: &'input str,
    __0: (usize, Operator, usize),
    __1: (usize, Value, usize),
) -> ::std::vec::Vec<(Operator, Value)>
{
    let __start0 = __0.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action36(
        input,
        __0,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action34(
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
pub fn __action42<
    'input,
>(
    input: &'input str,
    __0: (usize, ::std::vec::Vec<(Operator, Value)>, usize),
    __1: (usize, Operator, usize),
    __2: (usize, Value, usize),
) -> ::std::vec::Vec<(Operator, Value)>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action36(
        input,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action35(
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
pub fn __action43<
    'input,
>(
    input: &'input str,
    __0: (usize, ::std::option::Option<Value>, usize),
    __1: (usize, ::std::vec::Vec<(Operator, Value)>, usize),
    __2: (usize, Operator, usize),
) -> Application
{
    let __start0 = __2.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action32(
        input,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action17(
        input,
        __0,
        __1,
        __temp0,
    )
}

#[allow(unused_variables)]
pub fn __action44<
    'input,
>(
    input: &'input str,
    __0: (usize, ::std::option::Option<Value>, usize),
    __1: (usize, ::std::vec::Vec<(Operator, Value)>, usize),
) -> Application
{
    let __start0 = __1.2.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action33(
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action17(
        input,
        __0,
        __1,
        __temp0,
    )
}

#[allow(unused_variables)]
pub fn __action45<
    'input,
>(
    input: &'input str,
    __0: (usize, Value, usize),
    __1: (usize, ::std::vec::Vec<(Operator, Value)>, usize),
    __2: (usize, Operator, usize),
) -> Application
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action37(
        input,
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action43(
        input,
        __temp0,
        __1,
        __2,
    )
}

#[allow(unused_variables)]
pub fn __action46<
    'input,
>(
    input: &'input str,
    __0: (usize, ::std::vec::Vec<(Operator, Value)>, usize),
    __1: (usize, Operator, usize),
) -> Application
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action38(
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action43(
        input,
        __temp0,
        __0,
        __1,
    )
}

#[allow(unused_variables)]
pub fn __action47<
    'input,
>(
    input: &'input str,
    __0: (usize, Value, usize),
    __1: (usize, ::std::vec::Vec<(Operator, Value)>, usize),
) -> Application
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action37(
        input,
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action44(
        input,
        __temp0,
        __1,
    )
}

#[allow(unused_variables)]
pub fn __action48<
    'input,
>(
    input: &'input str,
    __0: (usize, ::std::vec::Vec<(Operator, Value)>, usize),
) -> Application
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action38(
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action44(
        input,
        __temp0,
        __0,
    )
}

pub trait __ToTriple<'input, > {
    type Error;
    fn to_triple(value: Self) -> Result<(usize,(usize, &'input str),usize),Self::Error>;
}

impl<'input, > __ToTriple<'input, > for (usize, (usize, &'input str), usize) {
    type Error = ();
    fn to_triple(value: Self) -> Result<(usize,(usize, &'input str),usize),()> {
        Ok(value)
    }
}
impl<'input, > __ToTriple<'input, > for Result<(usize, (usize, &'input str), usize),()> {
    type Error = ();
    fn to_triple(value: Self) -> Result<(usize,(usize, &'input str),usize),()> {
        value
    }
}
