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
        0, // on "exists", error
        0, // on "forall", error
        14, // on r#"\'[^\']+\'"#, goto 13
        15, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 14
        16, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 15
        17, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 16
        // State 1
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -9, // on ".", reduce `Application = (Operator Value)+ => ActionFn(48);`
        -9, // on ":-", reduce `Application = (Operator Value)+ => ActionFn(48);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        15, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 14
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        17, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 16
        // State 2
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        19, // on ".", goto 18
        20, // on ":-", goto 19
        0, // on ";", error
        0, // on "=>", error
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
        -38, // on ".", reduce `Value = Atom => ActionFn(26);`
        -38, // on ":-", reduce `Value = Atom => ActionFn(26);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -38, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = Atom => ActionFn(26);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -38, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = Atom => ActionFn(26);`
        // State 4
        21, // on "(", goto 20
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -43, // on ".", reduce `Variable = Identifier => ActionFn(30);`
        -43, // on ":-", reduce `Variable = Identifier => ActionFn(30);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -43, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Variable = Identifier => ActionFn(30);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -43, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Variable = Identifier => ActionFn(30);`
        // State 5
        -29, // on "(", reduce `Item+ = Item => ActionFn(39);`
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        -29, // on r#"\'[^\']+\'"#, reduce `Item+ = Item => ActionFn(39);`
        -29, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Item+ = Item => ActionFn(39);`
        -29, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Item+ = Item => ActionFn(39);`
        -29, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Item+ = Item => ActionFn(39);`
        // State 6
        13, // on "(", goto 12
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        14, // on r#"\'[^\']+\'"#, goto 13
        15, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 14
        16, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 15
        17, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 16
        // State 7
        13, // on "(", goto 12
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -5, // on ".", reduce `Application = Operator => ActionFn(17);`
        -5, // on ":-", reduce `Application = Operator => ActionFn(17);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        14, // on r#"\'[^\']+\'"#, goto 13
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        25, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 24
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
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 9
        -28, // on "(", reduce `Item = Rule => ActionFn(3);`
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        -28, // on r#"\'[^\']+\'"#, reduce `Item = Rule => ActionFn(3);`
        -28, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Item = Rule => ActionFn(3);`
        -28, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Item = Rule => ActionFn(3);`
        -28, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Item = Rule => ActionFn(3);`
        // State 10
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -4, // on ".", reduce `Application = Value => ActionFn(16);`
        -4, // on ":-", reduce `Application = Value => ActionFn(16);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        28, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 27
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        29, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 28
        // State 11
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -39, // on ".", reduce `Value = Variable => ActionFn(27);`
        -39, // on ":-", reduce `Value = Variable => ActionFn(27);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -39, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = Variable => ActionFn(27);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -39, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = Variable => ActionFn(27);`
        // State 12
        37, // on "(", goto 36
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        38, // on r#"\'[^\']+\'"#, goto 37
        39, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 38
        40, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 39
        41, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 40
        // State 13
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -14, // on ".", reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        -14, // on ":-", reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -14, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -14, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        // State 14
        -32, // on "(", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(25);`
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -32, // on ".", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(25);`
        -32, // on ":-", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(25);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        -32, // on r#"\'[^\']+\'"#, reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(25);`
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        -32, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(25);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 15
        -26, // on "(", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -26, // on ".", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        -26, // on ":-", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -26, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -26, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        // State 16
        -31, // on "(", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(24);`
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -31, // on ".", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(24);`
        -31, // on ":-", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(24);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        -31, // on r#"\'[^\']+\'"#, reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(24);`
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        -31, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(24);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 17
        13, // on "(", goto 12
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -7, // on ".", reduce `Application = (Operator Value)+, Operator => ActionFn(46);`
        -7, // on ":-", reduce `Application = (Operator Value)+, Operator => ActionFn(46);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        14, // on r#"\'[^\']+\'"#, goto 13
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        25, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 24
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 18
        -27, // on "(", reduce `Item = Application, "." => ActionFn(2);`
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        -27, // on r#"\'[^\']+\'"#, reduce `Item = Application, "." => ActionFn(2);`
        -27, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Item = Application, "." => ActionFn(2);`
        -27, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Item = Application, "." => ActionFn(2);`
        -27, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Item = Application, "." => ActionFn(2);`
        // State 19
        55, // on "(", goto 54
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        56, // on "exists", goto 55
        57, // on "forall", goto 56
        58, // on r#"\'[^\']+\'"#, goto 57
        59, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 58
        60, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 59
        61, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 60
        // State 20
        70, // on "(", goto 69
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        71, // on r#"\'[^\']+\'"#, goto 70
        72, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 71
        73, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 72
        74, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 73
        // State 21
        -30, // on "(", reduce `Item+ = Item+, Item => ActionFn(40);`
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        -30, // on r#"\'[^\']+\'"#, reduce `Item+ = Item+, Item => ActionFn(40);`
        -30, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Item+ = Item+, Item => ActionFn(40);`
        -30, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Item+ = Item+, Item => ActionFn(40);`
        -30, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Item+ = Item+, Item => ActionFn(40);`
        // State 22
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -43, // on ".", reduce `Variable = Identifier => ActionFn(30);`
        -43, // on ":-", reduce `Variable = Identifier => ActionFn(30);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -43, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Variable = Identifier => ActionFn(30);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -43, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Variable = Identifier => ActionFn(30);`
        // State 23
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -2, // on ".", reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        -2, // on ":-", reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -2, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -2, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        // State 24
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -26, // on ".", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        -26, // on ":-", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -26, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -26, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        // State 25
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -8, // on ".", reduce `Application = Value, (Operator Value)+ => ActionFn(47);`
        -8, // on ":-", reduce `Application = Value, (Operator Value)+ => ActionFn(47);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        15, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 14
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        17, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 16
        // State 26
        13, // on "(", goto 12
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        14, // on r#"\'[^\']+\'"#, goto 13
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        25, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 24
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 27
        -32, // on "(", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(25);`
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        -32, // on r#"\'[^\']+\'"#, reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(25);`
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        -32, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(25);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 28
        -31, // on "(", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(24);`
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        -31, // on r#"\'[^\']+\'"#, reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(24);`
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        -31, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(24);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 29
        0, // on "(", error
        -9, // on ")", reduce `Application = (Operator Value)+ => ActionFn(48);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        39, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 38
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        41, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 40
        // State 30
        0, // on "(", error
        77, // on ")", goto 76
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 31
        0, // on "(", error
        -38, // on ")", reduce `Value = Atom => ActionFn(26);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -38, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = Atom => ActionFn(26);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -38, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = Atom => ActionFn(26);`
        // State 32
        78, // on "(", goto 77
        -43, // on ")", reduce `Variable = Identifier => ActionFn(30);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -43, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Variable = Identifier => ActionFn(30);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -43, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Variable = Identifier => ActionFn(30);`
        // State 33
        37, // on "(", goto 36
        -5, // on ")", reduce `Application = Operator => ActionFn(17);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        38, // on r#"\'[^\']+\'"#, goto 37
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        81, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 80
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 34
        0, // on "(", error
        -4, // on ")", reduce `Application = Value => ActionFn(16);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        28, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 27
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        29, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 28
        // State 35
        0, // on "(", error
        -39, // on ")", reduce `Value = Variable => ActionFn(27);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -39, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = Variable => ActionFn(27);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -39, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = Variable => ActionFn(27);`
        // State 36
        37, // on "(", goto 36
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        38, // on r#"\'[^\']+\'"#, goto 37
        39, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 38
        40, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 39
        41, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 40
        // State 37
        0, // on "(", error
        -14, // on ")", reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -14, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -14, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        // State 38
        -32, // on "(", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(25);`
        -32, // on ")", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(25);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        -32, // on r#"\'[^\']+\'"#, reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(25);`
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        -32, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(25);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 39
        -26, // on "(", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        -26, // on ")", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -26, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -26, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        // State 40
        -31, // on "(", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(24);`
        -31, // on ")", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(24);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        -31, // on r#"\'[^\']+\'"#, reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(24);`
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        -31, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(24);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 41
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -3, // on ".", reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        -3, // on ":-", reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -3, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -3, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        // State 42
        0, // on "(", error
        0, // on ")", error
        -9, // on ",", reduce `Application = (Operator Value)+ => ActionFn(48);`
        0, // on "->", error
        -9, // on ".", reduce `Application = (Operator Value)+ => ActionFn(48);`
        0, // on ":-", error
        -9, // on ";", reduce `Application = (Operator Value)+ => ActionFn(48);`
        -9, // on "=>", reduce `Application = (Operator Value)+ => ActionFn(48);`
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        59, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 58
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        61, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 60
        // State 43
        0, // on "(", error
        0, // on ")", error
        -18, // on ",", reduce `FactApply = Application => ActionFn(15);`
        0, // on "->", error
        -18, // on ".", reduce `FactApply = Application => ActionFn(15);`
        0, // on ":-", error
        -18, // on ";", reduce `FactApply = Application => ActionFn(15);`
        -18, // on "=>", reduce `FactApply = Application => ActionFn(15);`
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 44
        0, // on "(", error
        0, // on ")", error
        -38, // on ",", reduce `Value = Atom => ActionFn(26);`
        0, // on "->", error
        -38, // on ".", reduce `Value = Atom => ActionFn(26);`
        0, // on ":-", error
        -38, // on ";", reduce `Value = Atom => ActionFn(26);`
        -38, // on "=>", reduce `Value = Atom => ActionFn(26);`
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -38, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = Atom => ActionFn(26);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -38, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = Atom => ActionFn(26);`
        // State 45
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        86, // on ".", goto 85
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 46
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -15, // on ".", reduce `Fact = FactAnd => ActionFn(5);`
        0, // on ":-", error
        87, // on ";", goto 86
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 47
        0, // on "(", error
        0, // on ")", error
        -19, // on ",", reduce `FactFunc = FactApply => ActionFn(10);`
        0, // on "->", error
        -19, // on ".", reduce `FactFunc = FactApply => ActionFn(10);`
        0, // on ":-", error
        -19, // on ";", reduce `FactFunc = FactApply => ActionFn(10);`
        88, // on "=>", goto 87
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 48
        0, // on "(", error
        0, // on ")", error
        -24, // on ",", reduce `FactOr = FactFunc => ActionFn(8);`
        0, // on "->", error
        -24, // on ".", reduce `FactOr = FactFunc => ActionFn(8);`
        0, // on ":-", error
        -24, // on ";", reduce `FactOr = FactFunc => ActionFn(8);`
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 49
        0, // on "(", error
        0, // on ")", error
        89, // on ",", goto 88
        0, // on "->", error
        -16, // on ".", reduce `FactAnd = FactOr => ActionFn(6);`
        0, // on ":-", error
        -16, // on ";", reduce `FactAnd = FactOr => ActionFn(6);`
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 50
        90, // on "(", goto 89
        0, // on ")", error
        -43, // on ",", reduce `Variable = Identifier => ActionFn(30);`
        -43, // on "->", reduce `Variable = Identifier => ActionFn(30);`
        -43, // on ".", reduce `Variable = Identifier => ActionFn(30);`
        0, // on ":-", error
        -43, // on ";", reduce `Variable = Identifier => ActionFn(30);`
        -43, // on "=>", reduce `Variable = Identifier => ActionFn(30);`
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -43, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Variable = Identifier => ActionFn(30);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -43, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Variable = Identifier => ActionFn(30);`
        // State 51
        55, // on "(", goto 54
        0, // on ")", error
        -5, // on ",", reduce `Application = Operator => ActionFn(17);`
        0, // on "->", error
        -5, // on ".", reduce `Application = Operator => ActionFn(17);`
        0, // on ":-", error
        -5, // on ";", reduce `Application = Operator => ActionFn(17);`
        -5, // on "=>", reduce `Application = Operator => ActionFn(17);`
        0, // on "exists", error
        0, // on "forall", error
        58, // on r#"\'[^\']+\'"#, goto 57
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        94, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 93
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 52
        0, // on "(", error
        0, // on ")", error
        -4, // on ",", reduce `Application = Value => ActionFn(16);`
        0, // on "->", error
        -4, // on ".", reduce `Application = Value => ActionFn(16);`
        0, // on ":-", error
        -4, // on ";", reduce `Application = Value => ActionFn(16);`
        -4, // on "=>", reduce `Application = Value => ActionFn(16);`
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        28, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 27
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        29, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 28
        // State 53
        0, // on "(", error
        0, // on ")", error
        -39, // on ",", reduce `Value = Variable => ActionFn(27);`
        97, // on "->", goto 96
        -39, // on ".", reduce `Value = Variable => ActionFn(27);`
        0, // on ":-", error
        -39, // on ";", reduce `Value = Variable => ActionFn(27);`
        -39, // on "=>", reduce `Value = Variable => ActionFn(27);`
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -39, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = Variable => ActionFn(27);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -39, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = Variable => ActionFn(27);`
        // State 54
        37, // on "(", goto 36
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        38, // on r#"\'[^\']+\'"#, goto 37
        39, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 38
        40, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 39
        41, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 40
        // State 55
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        101, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 100
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 56
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        101, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 100
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 57
        0, // on "(", error
        0, // on ")", error
        -14, // on ",", reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        0, // on "->", error
        -14, // on ".", reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        0, // on ":-", error
        -14, // on ";", reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        -14, // on "=>", reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -14, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -14, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        // State 58
        -32, // on "(", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(25);`
        0, // on ")", error
        -32, // on ",", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(25);`
        0, // on "->", error
        -32, // on ".", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(25);`
        0, // on ":-", error
        -32, // on ";", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(25);`
        -32, // on "=>", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(25);`
        0, // on "exists", error
        0, // on "forall", error
        -32, // on r#"\'[^\']+\'"#, reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(25);`
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        -32, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(25);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 59
        -26, // on "(", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on ")", error
        -26, // on ",", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        -26, // on "->", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        -26, // on ".", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on ":-", error
        -26, // on ";", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        -26, // on "=>", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -26, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -26, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        // State 60
        -31, // on "(", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(24);`
        0, // on ")", error
        -31, // on ",", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(24);`
        0, // on "->", error
        -31, // on ".", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(24);`
        0, // on ":-", error
        -31, // on ";", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(24);`
        -31, // on "=>", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(24);`
        0, // on "exists", error
        0, // on "forall", error
        -31, // on r#"\'[^\']+\'"#, reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(24);`
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        -31, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(24);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 61
        0, // on "(", error
        -9, // on ")", reduce `Application = (Operator Value)+ => ActionFn(48);`
        -9, // on ",", reduce `Application = (Operator Value)+ => ActionFn(48);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        72, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 71
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        74, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 73
        // State 62
        0, // on "(", error
        -11, // on ")", reduce `Applications = Application => ActionFn(20);`
        -11, // on ",", reduce `Applications = Application => ActionFn(20);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 63
        0, // on "(", error
        104, // on ")", goto 103
        105, // on ",", goto 104
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 64
        0, // on "(", error
        -38, // on ")", reduce `Value = Atom => ActionFn(26);`
        -38, // on ",", reduce `Value = Atom => ActionFn(26);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -38, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = Atom => ActionFn(26);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -38, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = Atom => ActionFn(26);`
        // State 65
        106, // on "(", goto 105
        -43, // on ")", reduce `Variable = Identifier => ActionFn(30);`
        -43, // on ",", reduce `Variable = Identifier => ActionFn(30);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -43, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Variable = Identifier => ActionFn(30);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -43, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Variable = Identifier => ActionFn(30);`
        // State 66
        70, // on "(", goto 69
        -5, // on ")", reduce `Application = Operator => ActionFn(17);`
        -5, // on ",", reduce `Application = Operator => ActionFn(17);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        71, // on r#"\'[^\']+\'"#, goto 70
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        109, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 108
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 67
        0, // on "(", error
        -4, // on ")", reduce `Application = Value => ActionFn(16);`
        -4, // on ",", reduce `Application = Value => ActionFn(16);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        28, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 27
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        29, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 28
        // State 68
        0, // on "(", error
        -39, // on ")", reduce `Value = Variable => ActionFn(27);`
        -39, // on ",", reduce `Value = Variable => ActionFn(27);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -39, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = Variable => ActionFn(27);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -39, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = Variable => ActionFn(27);`
        // State 69
        37, // on "(", goto 36
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        38, // on r#"\'[^\']+\'"#, goto 37
        39, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 38
        40, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 39
        41, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 40
        // State 70
        0, // on "(", error
        -14, // on ")", reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        -14, // on ",", reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -14, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -14, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Atom = r#"\'[^\']+\'"# => ActionFn(29);`
        // State 71
        -32, // on "(", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(25);`
        -32, // on ")", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(25);`
        -32, // on ",", reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(25);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        -32, // on r#"\'[^\']+\'"#, reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(25);`
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        -32, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(25);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 72
        -26, // on "(", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        -26, // on ")", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        -26, // on ",", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -26, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -26, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        // State 73
        -31, // on "(", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(24);`
        -31, // on ")", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(24);`
        -31, // on ",", reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(24);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        -31, // on r#"\'[^\']+\'"#, reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(24);`
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        -31, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(24);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 74
        13, // on "(", goto 12
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -6, // on ".", reduce `Application = Value, (Operator Value)+, Operator => ActionFn(45);`
        -6, // on ":-", reduce `Application = Value, (Operator Value)+, Operator => ActionFn(45);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        14, // on r#"\'[^\']+\'"#, goto 13
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        25, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 24
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 75
        37, // on "(", goto 36
        -7, // on ")", reduce `Application = (Operator Value)+, Operator => ActionFn(46);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        38, // on r#"\'[^\']+\'"#, goto 37
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        81, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 80
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 76
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -40, // on ".", reduce `Value = "(", Application, ")" => ActionFn(28);`
        -40, // on ":-", reduce `Value = "(", Application, ")" => ActionFn(28);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -40, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = "(", Application, ")" => ActionFn(28);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -40, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = "(", Application, ")" => ActionFn(28);`
        // State 77
        70, // on "(", goto 69
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        71, // on r#"\'[^\']+\'"#, goto 70
        72, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 71
        73, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 72
        74, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 73
        // State 78
        0, // on "(", error
        -43, // on ")", reduce `Variable = Identifier => ActionFn(30);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -43, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Variable = Identifier => ActionFn(30);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -43, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Variable = Identifier => ActionFn(30);`
        // State 79
        0, // on "(", error
        -2, // on ")", reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -2, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -2, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        // State 80
        0, // on "(", error
        -26, // on ")", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -26, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -26, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        // State 81
        0, // on "(", error
        -8, // on ")", reduce `Application = Value, (Operator Value)+ => ActionFn(47);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        39, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 38
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        41, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 40
        // State 82
        37, // on "(", goto 36
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        38, // on r#"\'[^\']+\'"#, goto 37
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        81, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 80
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 83
        0, // on "(", error
        116, // on ")", goto 115
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 84
        55, // on "(", goto 54
        0, // on ")", error
        -7, // on ",", reduce `Application = (Operator Value)+, Operator => ActionFn(46);`
        0, // on "->", error
        -7, // on ".", reduce `Application = (Operator Value)+, Operator => ActionFn(46);`
        0, // on ":-", error
        -7, // on ";", reduce `Application = (Operator Value)+, Operator => ActionFn(46);`
        -7, // on "=>", reduce `Application = (Operator Value)+, Operator => ActionFn(46);`
        0, // on "exists", error
        0, // on "forall", error
        58, // on r#"\'[^\']+\'"#, goto 57
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        94, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 93
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 85
        -37, // on "(", reduce `Rule = Application, ":-", Fact, "." => ActionFn(4);`
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        -37, // on r#"\'[^\']+\'"#, reduce `Rule = Application, ":-", Fact, "." => ActionFn(4);`
        -37, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Rule = Application, ":-", Fact, "." => ActionFn(4);`
        -37, // on r#"[A-Za-z][A-Za-z0-9_]*"#, reduce `Rule = Application, ":-", Fact, "." => ActionFn(4);`
        -37, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Rule = Application, ":-", Fact, "." => ActionFn(4);`
        // State 86
        55, // on "(", goto 54
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        56, // on "exists", goto 55
        57, // on "forall", goto 56
        58, // on r#"\'[^\']+\'"#, goto 57
        59, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 58
        60, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 59
        61, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 60
        // State 87
        55, // on "(", goto 54
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        56, // on "exists", goto 55
        57, // on "forall", goto 56
        58, // on r#"\'[^\']+\'"#, goto 57
        59, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 58
        60, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 59
        61, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 60
        // State 88
        55, // on "(", goto 54
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        56, // on "exists", goto 55
        57, // on "forall", goto 56
        58, // on r#"\'[^\']+\'"#, goto 57
        59, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 58
        60, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 59
        61, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 60
        // State 89
        70, // on "(", goto 69
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        71, // on r#"\'[^\']+\'"#, goto 70
        72, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 71
        73, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 72
        74, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 73
        // State 90
        0, // on "(", error
        0, // on ")", error
        -43, // on ",", reduce `Variable = Identifier => ActionFn(30);`
        0, // on "->", error
        -43, // on ".", reduce `Variable = Identifier => ActionFn(30);`
        0, // on ":-", error
        -43, // on ";", reduce `Variable = Identifier => ActionFn(30);`
        -43, // on "=>", reduce `Variable = Identifier => ActionFn(30);`
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -43, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Variable = Identifier => ActionFn(30);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -43, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Variable = Identifier => ActionFn(30);`
        // State 91
        0, // on "(", error
        0, // on ")", error
        -2, // on ",", reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        0, // on "->", error
        -2, // on ".", reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        0, // on ":-", error
        -2, // on ";", reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        -2, // on "=>", reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -2, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -2, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        // State 92
        0, // on "(", error
        0, // on ")", error
        -39, // on ",", reduce `Value = Variable => ActionFn(27);`
        0, // on "->", error
        -39, // on ".", reduce `Value = Variable => ActionFn(27);`
        0, // on ":-", error
        -39, // on ";", reduce `Value = Variable => ActionFn(27);`
        -39, // on "=>", reduce `Value = Variable => ActionFn(27);`
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -39, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = Variable => ActionFn(27);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -39, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = Variable => ActionFn(27);`
        // State 93
        0, // on "(", error
        0, // on ")", error
        -26, // on ",", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on "->", error
        -26, // on ".", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on ":-", error
        -26, // on ";", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        -26, // on "=>", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -26, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -26, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        // State 94
        0, // on "(", error
        0, // on ")", error
        -8, // on ",", reduce `Application = Value, (Operator Value)+ => ActionFn(47);`
        0, // on "->", error
        -8, // on ".", reduce `Application = Value, (Operator Value)+ => ActionFn(47);`
        0, // on ":-", error
        -8, // on ";", reduce `Application = Value, (Operator Value)+ => ActionFn(47);`
        -8, // on "=>", reduce `Application = Value, (Operator Value)+ => ActionFn(47);`
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        59, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 58
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        61, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 60
        // State 95
        55, // on "(", goto 54
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        58, // on r#"\'[^\']+\'"#, goto 57
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        94, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 93
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 96
        55, // on "(", goto 54
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        56, // on "exists", goto 55
        57, // on "forall", goto 56
        58, // on r#"\'[^\']+\'"#, goto 57
        59, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 58
        60, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 59
        61, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 60
        // State 97
        0, // on "(", error
        124, // on ")", goto 123
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 98
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        -43, // on "->", reduce `Variable = Identifier => ActionFn(30);`
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 99
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        125, // on "->", goto 124
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
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
        -26, // on "->", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 101
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        126, // on "->", goto 125
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 102
        70, // on "(", goto 69
        -7, // on ")", reduce `Application = (Operator Value)+, Operator => ActionFn(46);`
        -7, // on ",", reduce `Application = (Operator Value)+, Operator => ActionFn(46);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        71, // on r#"\'[^\']+\'"#, goto 70
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        109, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 108
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 103
        0, // on "(", error
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        -10, // on ".", reduce `Application = Identifier, "(", Applications, ")" => ActionFn(19);`
        -10, // on ":-", reduce `Application = Identifier, "(", Applications, ")" => ActionFn(19);`
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 104
        70, // on "(", goto 69
        -13, // on ")", reduce `Applications = Applications, "," => ActionFn(22);`
        -13, // on ",", reduce `Applications = Applications, "," => ActionFn(22);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        71, // on r#"\'[^\']+\'"#, goto 70
        72, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 71
        73, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 72
        74, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 73
        // State 105
        70, // on "(", goto 69
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        71, // on r#"\'[^\']+\'"#, goto 70
        72, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 71
        73, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 72
        74, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 73
        // State 106
        0, // on "(", error
        -43, // on ")", reduce `Variable = Identifier => ActionFn(30);`
        -43, // on ",", reduce `Variable = Identifier => ActionFn(30);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
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
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -2, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -2, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `(Operator Value)+ = Operator, Value => ActionFn(41);`
        // State 108
        0, // on "(", error
        -26, // on ")", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        -26, // on ",", reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -26, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -26, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Identifier = r#"[A-Za-z][A-Za-z0-9_]*"# => ActionFn(31);`
        // State 109
        0, // on "(", error
        -8, // on ")", reduce `Application = Value, (Operator Value)+ => ActionFn(47);`
        -8, // on ",", reduce `Application = Value, (Operator Value)+ => ActionFn(47);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        72, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 71
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        74, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 73
        // State 110
        70, // on "(", goto 69
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        71, // on r#"\'[^\']+\'"#, goto 70
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        109, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 108
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 111
        0, // on "(", error
        131, // on ")", goto 130
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
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
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -3, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -3, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        // State 113
        0, // on "(", error
        132, // on ")", goto 131
        105, // on ",", goto 104
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 114
        37, // on "(", goto 36
        -6, // on ")", reduce `Application = Value, (Operator Value)+, Operator => ActionFn(45);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        38, // on r#"\'[^\']+\'"#, goto 37
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        81, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 80
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 115
        0, // on "(", error
        -40, // on ")", reduce `Value = "(", Application, ")" => ActionFn(28);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -40, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = "(", Application, ")" => ActionFn(28);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -40, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = "(", Application, ")" => ActionFn(28);`
        // State 116
        0, // on "(", error
        0, // on ")", error
        -3, // on ",", reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        0, // on "->", error
        -3, // on ".", reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        0, // on ":-", error
        -3, // on ";", reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        -3, // on "=>", reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -3, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -3, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        // State 117
        0, // on "(", error
        0, // on ")", error
        89, // on ",", goto 88
        0, // on "->", error
        -17, // on ".", reduce `FactAnd = FactAnd, ";", FactOr => ActionFn(7);`
        0, // on ":-", error
        -17, // on ";", reduce `FactAnd = FactAnd, ";", FactOr => ActionFn(7);`
        0, // on "=>", error
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
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 119
        0, // on "(", error
        0, // on ")", error
        -25, // on ",", reduce `FactOr = FactOr, ",", FactFunc => ActionFn(9);`
        0, // on "->", error
        -25, // on ".", reduce `FactOr = FactOr, ",", FactFunc => ActionFn(9);`
        0, // on ":-", error
        -25, // on ";", reduce `FactOr = FactOr, ",", FactFunc => ActionFn(9);`
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 120
        0, // on "(", error
        133, // on ")", goto 132
        105, // on ",", goto 104
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 121
        55, // on "(", goto 54
        0, // on ")", error
        -6, // on ",", reduce `Application = Value, (Operator Value)+, Operator => ActionFn(45);`
        0, // on "->", error
        -6, // on ".", reduce `Application = Value, (Operator Value)+, Operator => ActionFn(45);`
        0, // on ":-", error
        -6, // on ";", reduce `Application = Value, (Operator Value)+, Operator => ActionFn(45);`
        -6, // on "=>", reduce `Application = Value, (Operator Value)+, Operator => ActionFn(45);`
        0, // on "exists", error
        0, // on "forall", error
        58, // on r#"\'[^\']+\'"#, goto 57
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        94, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 93
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 122
        0, // on "(", error
        0, // on ")", error
        -23, // on ",", reduce `FactFunc = Variable, "->", FactFunc => ActionFn(14);`
        0, // on "->", error
        -23, // on ".", reduce `FactFunc = Variable, "->", FactFunc => ActionFn(14);`
        0, // on ":-", error
        -23, // on ";", reduce `FactFunc = Variable, "->", FactFunc => ActionFn(14);`
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 123
        0, // on "(", error
        0, // on ")", error
        -40, // on ",", reduce `Value = "(", Application, ")" => ActionFn(28);`
        0, // on "->", error
        -40, // on ".", reduce `Value = "(", Application, ")" => ActionFn(28);`
        0, // on ":-", error
        -40, // on ";", reduce `Value = "(", Application, ")" => ActionFn(28);`
        -40, // on "=>", reduce `Value = "(", Application, ")" => ActionFn(28);`
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -40, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = "(", Application, ")" => ActionFn(28);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -40, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = "(", Application, ")" => ActionFn(28);`
        // State 124
        55, // on "(", goto 54
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        56, // on "exists", goto 55
        57, // on "forall", goto 56
        58, // on r#"\'[^\']+\'"#, goto 57
        59, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 58
        60, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 59
        61, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 60
        // State 125
        55, // on "(", goto 54
        0, // on ")", error
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        56, // on "exists", goto 55
        57, // on "forall", goto 56
        58, // on r#"\'[^\']+\'"#, goto 57
        59, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, goto 58
        60, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 59
        61, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, goto 60
        // State 126
        0, // on "(", error
        -3, // on ")", reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        -3, // on ",", reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -3, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -3, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `(Operator Value)+ = (Operator Value)+, Operator, Value => ActionFn(42);`
        // State 127
        0, // on "(", error
        -12, // on ")", reduce `Applications = Applications, ",", Application => ActionFn(21);`
        -12, // on ",", reduce `Applications = Applications, ",", Application => ActionFn(21);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 128
        0, // on "(", error
        136, // on ")", goto 135
        105, // on ",", goto 104
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 129
        70, // on "(", goto 69
        -6, // on ")", reduce `Application = Value, (Operator Value)+, Operator => ActionFn(45);`
        -6, // on ",", reduce `Application = Value, (Operator Value)+, Operator => ActionFn(45);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        71, // on r#"\'[^\']+\'"#, goto 70
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        109, // on r#"[A-Za-z][A-Za-z0-9_]*"#, goto 108
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 130
        0, // on "(", error
        -40, // on ")", reduce `Value = "(", Application, ")" => ActionFn(28);`
        -40, // on ",", reduce `Value = "(", Application, ")" => ActionFn(28);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        -40, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, reduce `Value = "(", Application, ")" => ActionFn(28);`
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        -40, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, reduce `Value = "(", Application, ")" => ActionFn(28);`
        // State 131
        0, // on "(", error
        -10, // on ")", reduce `Application = Identifier, "(", Applications, ")" => ActionFn(19);`
        0, // on ",", error
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 132
        0, // on "(", error
        0, // on ")", error
        -10, // on ",", reduce `Application = Identifier, "(", Applications, ")" => ActionFn(19);`
        0, // on "->", error
        -10, // on ".", reduce `Application = Identifier, "(", Applications, ")" => ActionFn(19);`
        0, // on ":-", error
        -10, // on ";", reduce `Application = Identifier, "(", Applications, ")" => ActionFn(19);`
        -10, // on "=>", reduce `Application = Identifier, "(", Applications, ")" => ActionFn(19);`
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 133
        0, // on "(", error
        0, // on ")", error
        -21, // on ",", reduce `FactFunc = "exists", Variable, "->", FactFunc => ActionFn(12);`
        0, // on "->", error
        -21, // on ".", reduce `FactFunc = "exists", Variable, "->", FactFunc => ActionFn(12);`
        0, // on ":-", error
        -21, // on ";", reduce `FactFunc = "exists", Variable, "->", FactFunc => ActionFn(12);`
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 134
        0, // on "(", error
        0, // on ")", error
        -22, // on ",", reduce `FactFunc = "forall", Variable, "->", FactFunc => ActionFn(13);`
        0, // on "->", error
        -22, // on ".", reduce `FactFunc = "forall", Variable, "->", FactFunc => ActionFn(13);`
        0, // on ":-", error
        -22, // on ";", reduce `FactFunc = "forall", Variable, "->", FactFunc => ActionFn(13);`
        0, // on "=>", error
        0, // on "exists", error
        0, // on "forall", error
        0, // on r#"\'[^\']+\'"#, error
        0, // on r#"[-|!@#$%^&*=+/:?~<>]+"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*"#, error
        0, // on r#"[A-Za-z][A-Za-z0-9_]*:"#, error
        // State 135
        0, // on "(", error
        -10, // on ")", reduce `Application = Identifier, "(", Applications, ")" => ActionFn(19);`
        -10, // on ",", reduce `Application = Identifier, "(", Applications, ")" => ActionFn(19);`
        0, // on "->", error
        0, // on ".", error
        0, // on ":-", error
        0, // on ";", error
        0, // on "=>", error
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
        -29, // on EOF, reduce `Item+ = Item => ActionFn(39);`
        -36, // on EOF, reduce `Program = Item+ => ActionFn(1);`
        0, // on EOF, error
        -44, // on EOF, reduce `__Program = Program => ActionFn(0);`
        -28, // on EOF, reduce `Item = Rule => ActionFn(3);`
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        0, // on EOF, error
        -27, // on EOF, reduce `Item = Application, "." => ActionFn(2);`
        0, // on EOF, error
        0, // on EOF, error
        -30, // on EOF, reduce `Item+ = Item+, Item => ActionFn(40);`
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
        -37, // on EOF, reduce `Rule = Application, ":-", Fact, "." => ActionFn(4);`
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
        18, // on Operator, goto 17
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
        22, // on Item, goto 21
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
        23, // on Identifier, goto 22
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        24, // on Value, goto 23
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
        26, // on (Operator Value)+, goto 25
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
        27, // on Operator, goto 26
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
        30, // on (Operator Value)+, goto 29
        31, // on Application, goto 30
        0, // on Applications, error
        32, // on Atom, goto 31
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        33, // on Identifier, goto 32
        0, // on Item, error
        0, // on Item+, error
        34, // on Operator, goto 33
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        35, // on Value, goto 34
        0, // on Value?, error
        36, // on Variable, goto 35
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
        4, // on Atom, goto 3
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        23, // on Identifier, goto 22
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        42, // on Value, goto 41
        0, // on Value?, error
        12, // on Variable, goto 11
        0, // on __Program, error
        // State 18
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
        // State 19
        0, // on (Operator Value), error
        43, // on (Operator Value)+, goto 42
        44, // on Application, goto 43
        0, // on Applications, error
        45, // on Atom, goto 44
        46, // on Fact, goto 45
        47, // on FactAnd, goto 46
        48, // on FactApply, goto 47
        49, // on FactFunc, goto 48
        50, // on FactOr, goto 49
        51, // on Identifier, goto 50
        0, // on Item, error
        0, // on Item+, error
        52, // on Operator, goto 51
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        53, // on Value, goto 52
        0, // on Value?, error
        54, // on Variable, goto 53
        0, // on __Program, error
        // State 20
        0, // on (Operator Value), error
        62, // on (Operator Value)+, goto 61
        63, // on Application, goto 62
        64, // on Applications, goto 63
        65, // on Atom, goto 64
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        66, // on Identifier, goto 65
        0, // on Item, error
        0, // on Item+, error
        67, // on Operator, goto 66
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        68, // on Value, goto 67
        0, // on Value?, error
        69, // on Variable, goto 68
        0, // on __Program, error
        // State 21
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
        75, // on Operator, goto 74
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
        4, // on Atom, goto 3
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        23, // on Identifier, goto 22
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        24, // on Value, goto 23
        0, // on Value?, error
        12, // on Variable, goto 11
        0, // on __Program, error
        // State 27
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
        76, // on Operator, goto 75
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
        0, // on Operator, error
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
        32, // on Atom, goto 31
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        79, // on Identifier, goto 78
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        80, // on Value, goto 79
        0, // on Value?, error
        36, // on Variable, goto 35
        0, // on __Program, error
        // State 34
        0, // on (Operator Value), error
        82, // on (Operator Value)+, goto 81
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
        83, // on Operator, goto 82
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        0, // on Variable, error
        0, // on __Program, error
        // State 35
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
        // State 36
        0, // on (Operator Value), error
        30, // on (Operator Value)+, goto 29
        84, // on Application, goto 83
        0, // on Applications, error
        32, // on Atom, goto 31
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        33, // on Identifier, goto 32
        0, // on Item, error
        0, // on Item+, error
        34, // on Operator, goto 33
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        35, // on Value, goto 34
        0, // on Value?, error
        36, // on Variable, goto 35
        0, // on __Program, error
        // State 37
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
        85, // on Operator, goto 84
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
        0, // on Operator, error
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
        45, // on Atom, goto 44
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        91, // on Identifier, goto 90
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        92, // on Value, goto 91
        0, // on Value?, error
        93, // on Variable, goto 92
        0, // on __Program, error
        // State 52
        0, // on (Operator Value), error
        95, // on (Operator Value)+, goto 94
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
        96, // on Operator, goto 95
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
        // State 54
        0, // on (Operator Value), error
        30, // on (Operator Value)+, goto 29
        98, // on Application, goto 97
        0, // on Applications, error
        32, // on Atom, goto 31
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        33, // on Identifier, goto 32
        0, // on Item, error
        0, // on Item+, error
        34, // on Operator, goto 33
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        35, // on Value, goto 34
        0, // on Value?, error
        36, // on Variable, goto 35
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
        99, // on Identifier, goto 98
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        100, // on Variable, goto 99
        0, // on __Program, error
        // State 56
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
        99, // on Identifier, goto 98
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        0, // on Value, error
        0, // on Value?, error
        102, // on Variable, goto 101
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
        103, // on Operator, goto 102
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
        0, // on Operator, error
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
        65, // on Atom, goto 64
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
        69, // on Variable, goto 68
        0, // on __Program, error
        // State 67
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
        30, // on (Operator Value)+, goto 29
        112, // on Application, goto 111
        0, // on Applications, error
        32, // on Atom, goto 31
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        33, // on Identifier, goto 32
        0, // on Item, error
        0, // on Item+, error
        34, // on Operator, goto 33
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        35, // on Value, goto 34
        0, // on Value?, error
        36, // on Variable, goto 35
        0, // on __Program, error
        // State 70
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
        4, // on Atom, goto 3
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        23, // on Identifier, goto 22
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        42, // on Value, goto 41
        0, // on Value?, error
        12, // on Variable, goto 11
        0, // on __Program, error
        // State 75
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        32, // on Atom, goto 31
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        79, // on Identifier, goto 78
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        113, // on Value, goto 112
        0, // on Value?, error
        36, // on Variable, goto 35
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
        62, // on (Operator Value)+, goto 61
        63, // on Application, goto 62
        114, // on Applications, goto 113
        65, // on Atom, goto 64
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        66, // on Identifier, goto 65
        0, // on Item, error
        0, // on Item+, error
        67, // on Operator, goto 66
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        68, // on Value, goto 67
        0, // on Value?, error
        69, // on Variable, goto 68
        0, // on __Program, error
        // State 78
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
        // State 79
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
        // State 82
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        32, // on Atom, goto 31
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        79, // on Identifier, goto 78
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        80, // on Value, goto 79
        0, // on Value?, error
        36, // on Variable, goto 35
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
        45, // on Atom, goto 44
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        91, // on Identifier, goto 90
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        117, // on Value, goto 116
        0, // on Value?, error
        93, // on Variable, goto 92
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
        0, // on Operator, error
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
        43, // on (Operator Value)+, goto 42
        44, // on Application, goto 43
        0, // on Applications, error
        45, // on Atom, goto 44
        0, // on Fact, error
        0, // on FactAnd, error
        48, // on FactApply, goto 47
        49, // on FactFunc, goto 48
        118, // on FactOr, goto 117
        51, // on Identifier, goto 50
        0, // on Item, error
        0, // on Item+, error
        52, // on Operator, goto 51
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        53, // on Value, goto 52
        0, // on Value?, error
        54, // on Variable, goto 53
        0, // on __Program, error
        // State 87
        0, // on (Operator Value), error
        43, // on (Operator Value)+, goto 42
        44, // on Application, goto 43
        0, // on Applications, error
        45, // on Atom, goto 44
        0, // on Fact, error
        0, // on FactAnd, error
        48, // on FactApply, goto 47
        119, // on FactFunc, goto 118
        0, // on FactOr, error
        51, // on Identifier, goto 50
        0, // on Item, error
        0, // on Item+, error
        52, // on Operator, goto 51
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        53, // on Value, goto 52
        0, // on Value?, error
        54, // on Variable, goto 53
        0, // on __Program, error
        // State 88
        0, // on (Operator Value), error
        43, // on (Operator Value)+, goto 42
        44, // on Application, goto 43
        0, // on Applications, error
        45, // on Atom, goto 44
        0, // on Fact, error
        0, // on FactAnd, error
        48, // on FactApply, goto 47
        120, // on FactFunc, goto 119
        0, // on FactOr, error
        51, // on Identifier, goto 50
        0, // on Item, error
        0, // on Item+, error
        52, // on Operator, goto 51
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        53, // on Value, goto 52
        0, // on Value?, error
        54, // on Variable, goto 53
        0, // on __Program, error
        // State 89
        0, // on (Operator Value), error
        62, // on (Operator Value)+, goto 61
        63, // on Application, goto 62
        121, // on Applications, goto 120
        65, // on Atom, goto 64
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        66, // on Identifier, goto 65
        0, // on Item, error
        0, // on Item+, error
        67, // on Operator, goto 66
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        68, // on Value, goto 67
        0, // on Value?, error
        69, // on Variable, goto 68
        0, // on __Program, error
        // State 90
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
        // State 91
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
        // State 92
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
        // State 93
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
        122, // on Operator, goto 121
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
        45, // on Atom, goto 44
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        91, // on Identifier, goto 90
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        92, // on Value, goto 91
        0, // on Value?, error
        93, // on Variable, goto 92
        0, // on __Program, error
        // State 96
        0, // on (Operator Value), error
        43, // on (Operator Value)+, goto 42
        44, // on Application, goto 43
        0, // on Applications, error
        45, // on Atom, goto 44
        0, // on Fact, error
        0, // on FactAnd, error
        48, // on FactApply, goto 47
        123, // on FactFunc, goto 122
        0, // on FactOr, error
        51, // on Identifier, goto 50
        0, // on Item, error
        0, // on Item+, error
        52, // on Operator, goto 51
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        53, // on Value, goto 52
        0, // on Value?, error
        54, // on Variable, goto 53
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
        0, // on Operator, error
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
        // State 102
        0, // on (Operator Value), error
        0, // on (Operator Value)+, error
        0, // on Application, error
        0, // on Applications, error
        65, // on Atom, goto 64
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
        127, // on Value, goto 126
        0, // on Value?, error
        69, // on Variable, goto 68
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
        62, // on (Operator Value)+, goto 61
        128, // on Application, goto 127
        0, // on Applications, error
        65, // on Atom, goto 64
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        66, // on Identifier, goto 65
        0, // on Item, error
        0, // on Item+, error
        67, // on Operator, goto 66
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        68, // on Value, goto 67
        0, // on Value?, error
        69, // on Variable, goto 68
        0, // on __Program, error
        // State 105
        0, // on (Operator Value), error
        62, // on (Operator Value)+, goto 61
        63, // on Application, goto 62
        129, // on Applications, goto 128
        65, // on Atom, goto 64
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        66, // on Identifier, goto 65
        0, // on Item, error
        0, // on Item+, error
        67, // on Operator, goto 66
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        68, // on Value, goto 67
        0, // on Value?, error
        69, // on Variable, goto 68
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
        130, // on Operator, goto 129
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
        65, // on Atom, goto 64
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
        69, // on Variable, goto 68
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
        32, // on Atom, goto 31
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        79, // on Identifier, goto 78
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        113, // on Value, goto 112
        0, // on Value?, error
        36, // on Variable, goto 35
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
        45, // on Atom, goto 44
        0, // on Fact, error
        0, // on FactAnd, error
        0, // on FactApply, error
        0, // on FactFunc, error
        0, // on FactOr, error
        91, // on Identifier, goto 90
        0, // on Item, error
        0, // on Item+, error
        0, // on Operator, error
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        117, // on Value, goto 116
        0, // on Value?, error
        93, // on Variable, goto 92
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
        43, // on (Operator Value)+, goto 42
        44, // on Application, goto 43
        0, // on Applications, error
        45, // on Atom, goto 44
        0, // on Fact, error
        0, // on FactAnd, error
        48, // on FactApply, goto 47
        134, // on FactFunc, goto 133
        0, // on FactOr, error
        51, // on Identifier, goto 50
        0, // on Item, error
        0, // on Item+, error
        52, // on Operator, goto 51
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        53, // on Value, goto 52
        0, // on Value?, error
        54, // on Variable, goto 53
        0, // on __Program, error
        // State 125
        0, // on (Operator Value), error
        43, // on (Operator Value)+, goto 42
        44, // on Application, goto 43
        0, // on Applications, error
        45, // on Atom, goto 44
        0, // on Fact, error
        0, // on FactAnd, error
        48, // on FactApply, goto 47
        135, // on FactFunc, goto 134
        0, // on FactOr, error
        51, // on Identifier, goto 50
        0, // on Item, error
        0, // on Item+, error
        52, // on Operator, goto 51
        0, // on Operator?, error
        0, // on OperatorValue, error
        0, // on Program, error
        0, // on Rule, error
        53, // on Value, goto 52
        0, // on Value?, error
        54, // on Variable, goto 53
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
        65, // on Atom, goto 64
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
        127, // on Value, goto 126
        0, // on Value?, error
        69, // on Variable, goto 68
        0, // on __Program, error
        // State 130
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
        // State 135
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
                _ => {
                    return Err(__lalrpop_util::ParseError::UnrecognizedToken {
                        token: Some(__lookahead),
                        expected: vec![],
                    });
                }
            };
            loop {
                let __state = *__states.last().unwrap() as usize;
                let __action = __ACTION[__state * 14 + __integer];
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
                            (8, __tok0) => __Symbol::Term_22exists_22(__tok0),
                            _ => unreachable!(),
                        },
                        9 => match __lookahead.1 {
                            (9, __tok0) => __Symbol::Term_22forall_22(__tok0),
                            _ => unreachable!(),
                        },
                        10 => match __lookahead.1 {
                            (10, __tok0) => __Symbol::Termr_23_22_5c_27_5b_5e_5c_27_5d_2b_5c_27_22_23(__tok0),
                            _ => unreachable!(),
                        },
                        11 => match __lookahead.1 {
                            (11, __tok0) => __Symbol::Termr_23_22_5b_2d_7c_21_40_23_24_25_5e_26_2a_3d_2b_2f_3a_3f_7e_3c_3e_5d_2b_22_23(__tok0),
                            _ => unreachable!(),
                        },
                        12 => match __lookahead.1 {
                            (12, __tok0) => __Symbol::Termr_23_22_5bA_2dZa_2dz_5d_5bA_2dZa_2dz0_2d9___5d_2a_22_23(__tok0),
                            _ => unreachable!(),
                        },
                        13 => match __lookahead.1 {
                            (13, __tok0) => __Symbol::Termr_23_22_5bA_2dZa_2dz_5d_5bA_2dZa_2dz0_2d9___5d_2a_3a_22_23(__tok0),
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
                // Application = Value => ActionFn(16);
                let __sym0 = __pop_NtValue(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action16::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtApplication(__nt), __end));
                2
            }
            5 => {
                // Application = Operator => ActionFn(17);
                let __sym0 = __pop_NtOperator(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action17::<>(input, __sym0);
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
                // Application = Identifier, "(", Applications, ")" => ActionFn(19);
                let __sym3 = __pop_Term_22_29_22(__symbols);
                let __sym2 = __pop_NtApplications(__symbols);
                let __sym1 = __pop_Term_22_28_22(__symbols);
                let __sym0 = __pop_NtIdentifier(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = super::__action19::<>(input, __sym0, __sym1, __sym2, __sym3);
                let __states_len = __states.len();
                __states.truncate(__states_len - 4);
                __symbols.push((__start, __Symbol::NtApplication(__nt), __end));
                2
            }
            11 => {
                // Applications = Application => ActionFn(20);
                let __sym0 = __pop_NtApplication(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action20::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtApplications(__nt), __end));
                3
            }
            12 => {
                // Applications = Applications, ",", Application => ActionFn(21);
                let __sym2 = __pop_NtApplication(__symbols);
                let __sym1 = __pop_Term_22_2c_22(__symbols);
                let __sym0 = __pop_NtApplications(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = super::__action21::<>(input, __sym0, __sym1, __sym2);
                let __states_len = __states.len();
                __states.truncate(__states_len - 3);
                __symbols.push((__start, __Symbol::NtApplications(__nt), __end));
                3
            }
            13 => {
                // Applications = Applications, "," => ActionFn(22);
                let __sym1 = __pop_Term_22_2c_22(__symbols);
                let __sym0 = __pop_NtApplications(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = super::__action22::<>(input, __sym0, __sym1);
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
                // FactApply = Application => ActionFn(15);
                let __sym0 = __pop_NtApplication(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action15::<>(input, __sym0);
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
                // FactFunc = "exists", Variable, "->", FactFunc => ActionFn(12);
                let __sym3 = __pop_NtFactFunc(__symbols);
                let __sym2 = __pop_Term_22_2d_3e_22(__symbols);
                let __sym1 = __pop_NtVariable(__symbols);
                let __sym0 = __pop_Term_22exists_22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = super::__action12::<>(input, __sym0, __sym1, __sym2, __sym3);
                let __states_len = __states.len();
                __states.truncate(__states_len - 4);
                __symbols.push((__start, __Symbol::NtFactFunc(__nt), __end));
                8
            }
            22 => {
                // FactFunc = "forall", Variable, "->", FactFunc => ActionFn(13);
                let __sym3 = __pop_NtFactFunc(__symbols);
                let __sym2 = __pop_Term_22_2d_3e_22(__symbols);
                let __sym1 = __pop_NtVariable(__symbols);
                let __sym0 = __pop_Term_22forall_22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = super::__action13::<>(input, __sym0, __sym1, __sym2, __sym3);
                let __states_len = __states.len();
                __states.truncate(__states_len - 4);
                __symbols.push((__start, __Symbol::NtFactFunc(__nt), __end));
                8
            }
            23 => {
                // FactFunc = Variable, "->", FactFunc => ActionFn(14);
                let __sym2 = __pop_NtFactFunc(__symbols);
                let __sym1 = __pop_Term_22_2d_3e_22(__symbols);
                let __sym0 = __pop_NtVariable(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = super::__action14::<>(input, __sym0, __sym1, __sym2);
                let __states_len = __states.len();
                __states.truncate(__states_len - 3);
                __symbols.push((__start, __Symbol::NtFactFunc(__nt), __end));
                8
            }
            24 => {
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
            25 => {
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
            26 => {
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
            27 => {
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
            28 => {
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
            29 => {
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
            30 => {
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
            31 => {
                // Operator = r#"[A-Za-z][A-Za-z0-9_]*:"# => ActionFn(24);
                let __sym0 = __pop_Termr_23_22_5bA_2dZa_2dz_5d_5bA_2dZa_2dz0_2d9___5d_2a_3a_22_23(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action24::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtOperator(__nt), __end));
                13
            }
            32 => {
                // Operator = r#"[-|!@#$%^&*=+/:?~<>]+"# => ActionFn(25);
                let __sym0 = __pop_Termr_23_22_5b_2d_7c_21_40_23_24_25_5e_26_2a_3d_2b_2f_3a_3f_7e_3c_3e_5d_2b_22_23(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action25::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtOperator(__nt), __end));
                13
            }
            33 => {
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
            34 => {
                // Operator? =  => ActionFn(33);
                let __start = __symbols.last().map(|s| s.2.clone()).unwrap_or_default();
                let __end = __lookahead_start.cloned().unwrap_or_else(|| __start.clone());
                let __nt = super::__action33::<>(input, &__start, &__end);
                let __states_len = __states.len();
                __states.truncate(__states_len - 0);
                __symbols.push((__start, __Symbol::NtOperator_3f(__nt), __end));
                14
            }
            35 => {
                // OperatorValue = Operator, Value => ActionFn(23);
                let __sym1 = __pop_NtValue(__symbols);
                let __sym0 = __pop_NtOperator(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = super::__action23::<>(input, __sym0, __sym1);
                let __states_len = __states.len();
                __states.truncate(__states_len - 2);
                __symbols.push((__start, __Symbol::NtOperatorValue(__nt), __end));
                15
            }
            36 => {
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
            37 => {
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
            38 => {
                // Value = Atom => ActionFn(26);
                let __sym0 = __pop_NtAtom(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action26::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtValue(__nt), __end));
                18
            }
            39 => {
                // Value = Variable => ActionFn(27);
                let __sym0 = __pop_NtVariable(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action27::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtValue(__nt), __end));
                18
            }
            40 => {
                // Value = "(", Application, ")" => ActionFn(28);
                let __sym2 = __pop_Term_22_29_22(__symbols);
                let __sym1 = __pop_NtApplication(__symbols);
                let __sym0 = __pop_Term_22_28_22(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = super::__action28::<>(input, __sym0, __sym1, __sym2);
                let __states_len = __states.len();
                __states.truncate(__states_len - 3);
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
                            __current_match = Some((11, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        35 ... 38 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
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
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 1;
                            continue;
                        }
                        44 => /* ',' */ {
                            __current_match = Some((2, __index + 1));
                            __current_state = 5;
                            continue;
                        }
                        45 => /* '-' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 6;
                            continue;
                        }
                        46 => /* '.' */ {
                            __current_match = Some((4, __index + 1));
                            __current_state = 7;
                            continue;
                        }
                        47 => /* '/' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 8;
                            continue;
                        }
                        59 => /* ';' */ {
                            __current_match = Some((6, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        60 => /* '<' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        61 => /* '=' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 10;
                            continue;
                        }
                        62 ... 64 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 1;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 11;
                            continue;
                        }
                        94 => /* '^' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        97 ... 100 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 11;
                            continue;
                        }
                        101 => /* 'e' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        102 => /* 'f' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 13;
                            continue;
                        }
                        103 ... 122 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 11;
                            continue;
                        }
                        124 => /* '|' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        126 => /* '~' */ {
                            __current_match = Some((11, __index + 1));
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
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        35 ... 38 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 15;
                            continue;
                        }
                        42 ... 43 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 15;
                            continue;
                        }
                        45 => /* '-' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        47 => /* '/' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        60 ... 64 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 15;
                            continue;
                        }
                        94 => /* '^' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        124 => /* '|' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        126 => /* '~' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
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
                            __current_state = 16;
                            continue;
                        }
                        40 ... 1114111 => {
                            __current_state = 16;
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
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        35 ... 38 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 15;
                            continue;
                        }
                        42 ... 43 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 15;
                            continue;
                        }
                        45 => /* '-' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        47 => /* '/' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        60 ... 61 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 15;
                            continue;
                        }
                        62 => /* '>' */ {
                            __current_match = Some((3, __index + 1));
                            __current_state = 17;
                            continue;
                        }
                        63 ... 64 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 15;
                            continue;
                        }
                        94 => /* '^' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        124 => /* '|' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        126 => /* '~' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
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
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        35 ... 38 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 15;
                            continue;
                        }
                        42 ... 43 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 15;
                            continue;
                        }
                        45 => /* '-' */ {
                            __current_match = Some((5, __index + 1));
                            __current_state = 18;
                            continue;
                        }
                        47 => /* '/' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        60 ... 64 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 15;
                            continue;
                        }
                        94 => /* '^' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        124 => /* '|' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        126 => /* '~' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
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
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        35 ... 38 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 15;
                            continue;
                        }
                        42 ... 43 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 15;
                            continue;
                        }
                        45 => /* '-' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        47 => /* '/' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        60 ... 61 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 15;
                            continue;
                        }
                        62 => /* '>' */ {
                            __current_match = Some((7, __index + 1));
                            __current_state = 19;
                            continue;
                        }
                        63 ... 64 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 15;
                            continue;
                        }
                        94 => /* '^' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        124 => /* '|' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        126 => /* '~' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
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
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 20;
                            continue;
                        }
                        97 ... 122 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
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
                        48 ... 57 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 20;
                            continue;
                        }
                        97 ... 119 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        120 => /* 'x' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 22;
                            continue;
                        }
                        121 ... 122 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                13 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        48 ... 57 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 20;
                            continue;
                        }
                        97 ... 110 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        111 => /* 'o' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 23;
                            continue;
                        }
                        112 ... 122 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
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
                        _ => {
                            return __current_match;
                        }
                    }
                }
                15 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        33 => /* '!' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        35 ... 38 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 15;
                            continue;
                        }
                        42 ... 43 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 15;
                            continue;
                        }
                        45 => /* '-' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        47 => /* '/' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        60 ... 64 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 15;
                            continue;
                        }
                        94 => /* '^' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        124 => /* '|' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        126 => /* '~' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                16 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        0 ... 38 => {
                            __current_state = 24;
                            continue;
                        }
                        39 => /* '\'' */ {
                            __current_match = Some((10, __index + 1));
                            __current_state = 25;
                            continue;
                        }
                        40 ... 1114111 => {
                            __current_state = 24;
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
                        33 => /* '!' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        35 ... 38 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 15;
                            continue;
                        }
                        42 ... 43 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 15;
                            continue;
                        }
                        45 => /* '-' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        47 => /* '/' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        60 ... 64 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 15;
                            continue;
                        }
                        94 => /* '^' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        124 => /* '|' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        126 => /* '~' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
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
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        35 ... 38 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 15;
                            continue;
                        }
                        42 ... 43 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 15;
                            continue;
                        }
                        45 => /* '-' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        47 => /* '/' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        60 ... 64 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 15;
                            continue;
                        }
                        94 => /* '^' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        124 => /* '|' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        126 => /* '~' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
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
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        35 ... 38 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 15;
                            continue;
                        }
                        42 ... 43 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 15;
                            continue;
                        }
                        45 => /* '-' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        47 => /* '/' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        60 ... 64 => {
                            __current_match = Some((11, __index + __ch.len_utf8()));
                            __current_state = 15;
                            continue;
                        }
                        94 => /* '^' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        124 => /* '|' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
                            continue;
                        }
                        126 => /* '~' */ {
                            __current_match = Some((11, __index + 1));
                            __current_state = 15;
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
                        48 ... 57 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 20;
                            continue;
                        }
                        97 ... 122 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
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
                        _ => {
                            return __current_match;
                        }
                    }
                }
                22 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        48 ... 57 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 20;
                            continue;
                        }
                        97 ... 104 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        105 => /* 'i' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 26;
                            continue;
                        }
                        106 ... 122 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                23 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        48 ... 57 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 20;
                            continue;
                        }
                        97 ... 113 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        114 => /* 'r' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 27;
                            continue;
                        }
                        115 ... 122 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
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
                        0 ... 38 => {
                            __current_state = 24;
                            continue;
                        }
                        39 => /* '\'' */ {
                            __current_match = Some((10, __index + 1));
                            __current_state = 25;
                            continue;
                        }
                        40 ... 1114111 => {
                            __current_state = 24;
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
                        _ => {
                            return __current_match;
                        }
                    }
                }
                26 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        48 ... 57 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 20;
                            continue;
                        }
                        97 ... 114 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        115 => /* 's' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 28;
                            continue;
                        }
                        116 ... 122 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                27 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch as u32 {
                        48 ... 57 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 20;
                            continue;
                        }
                        97 => /* 'a' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 29;
                            continue;
                        }
                        98 ... 122 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
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
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 20;
                            continue;
                        }
                        97 ... 115 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        116 => /* 't' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 30;
                            continue;
                        }
                        117 ... 122 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
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
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 20;
                            continue;
                        }
                        97 ... 107 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        108 => /* 'l' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 31;
                            continue;
                        }
                        109 ... 122 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
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
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 20;
                            continue;
                        }
                        97 ... 114 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        115 => /* 's' */ {
                            __current_match = Some((8, __index + 1));
                            __current_state = 32;
                            continue;
                        }
                        116 ... 122 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
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
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 20;
                            continue;
                        }
                        97 ... 107 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        108 => /* 'l' */ {
                            __current_match = Some((9, __index + 1));
                            __current_state = 33;
                            continue;
                        }
                        109 ... 122 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
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
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 20;
                            continue;
                        }
                        97 ... 122 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
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
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        58 => /* ':' */ {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        65 ... 90 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
                            continue;
                        }
                        95 => /* '_' */ {
                            __current_match = Some((12, __index + 1));
                            __current_state = 20;
                            continue;
                        }
                        97 ... 122 => {
                            __current_match = Some((12, __index + __ch.len_utf8()));
                            __current_state = 20;
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
    (_, v, _): (usize, Variable, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, b, _): (usize, Fact, usize),
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
    (_, v, _): (usize, Variable, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, b, _): (usize, Fact, usize),
) -> Fact
{
    Fact { data: Box::new(FactData::ForAll(v, b)) }
}

#[allow(unused_variables)]
pub fn __action14<
    'input,
>(
    input: &'input str,
    (_, v, _): (usize, Variable, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, b, _): (usize, Fact, usize),
) -> Fact
{
    Fact { data: Box::new(FactData::Lambda(v, b)) }
}

#[allow(unused_variables)]
pub fn __action15<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, Application, usize),
) -> Fact
{
    Fact { data: Box::new(FactData::Apply(__0)) }
}

#[allow(unused_variables)]
pub fn __action16<
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
pub fn __action17<
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
pub fn __action18<
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
pub fn __action19<
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
pub fn __action20<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, Application, usize),
) -> Vec<Application>
{
    vec![__0]
}

#[allow(unused_variables)]
pub fn __action21<
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
pub fn __action22<
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
pub fn __action23<
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
pub fn __action24<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> Operator
{
    Operator::Colon(intern(__0))
}

#[allow(unused_variables)]
pub fn __action25<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> Operator
{
    Operator::Symbols(intern(__0))
}

#[allow(unused_variables)]
pub fn __action26<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, Atom, usize),
) -> Value
{
    Value::Atom(__0)
}

#[allow(unused_variables)]
pub fn __action27<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, Variable, usize),
) -> Value
{
    Value::Variable(__0)
}

#[allow(unused_variables)]
pub fn __action28<
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
    __action18(
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
    __action18(
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
