#![allow(unused_imports)]
#![allow(unused_variables)]
use ast::*;
use lalrpop_intern::intern;
extern crate lalrpop_util as __lalrpop_util;
use self::__lalrpop_util::ParseError as __ParseError;

mod __parse__Program {
    #![allow(non_snake_case, non_camel_case_types, unused_mut, unused_variables, unused_imports)]

    use ast::*;
    use lalrpop_intern::intern;
    extern crate lalrpop_util as __lalrpop_util;
    use self::__lalrpop_util::ParseError as __ParseError;
    pub fn parse_Program<
        'input,
    >(
        input: &'input str,
    ) -> Result<Program, __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __tokens = super::__intern_token::__Matcher::new(input);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match try!(__state0(input, None, &mut __tokens, __lookahead)) {
            (_, Some(__lookahead), _) => {
                Err(__ParseError::ExtraToken { token: __lookahead })
            }
            (_, None, __Nonterminal::____Program(__nt)) => {
                Ok(__nt)
            }
            _ => unreachable!(),
        }
    }

    #[allow(dead_code)]
    pub enum __Nonterminal<> {
        Application(Application),
        Bit(Bit),
        Bit_2b(::std::vec::Vec<Bit>),
        Fact(Fact),
        FactAnd(Fact),
        FactApply(Fact),
        FactFunc(Fact),
        FactOr(Fact),
        Item(Item),
        Item_2b(::std::vec::Vec<Item>),
        Program(Program),
        Rule(Rule),
        Variable(Variable),
        ____Program(Program),
    }

    // State 0
    //   Application = (*) Bit+ ["."]
    //   Application = (*) Bit+ [":-"]
    //   Bit = (*) Variable ["."]
    //   Bit = (*) Variable [":-"]
    //   Bit = (*) Variable ["["]
    //   Bit = (*) Variable [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) Variable [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) Variable [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) Variable [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" ["."]
    //   Bit = (*) "[" Fact "]" [":-"]
    //   Bit = (*) "[" Fact "]" ["["]
    //   Bit = (*) "[" Fact "]" [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) "[" Fact "]" [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["."]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [":-"]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["["]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["."]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [":-"]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["["]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["."]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [":-"]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["["]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit ["."]
    //   Bit+ = (*) Bit [":-"]
    //   Bit+ = (*) Bit ["["]
    //   Bit+ = (*) Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit ["."]
    //   Bit+ = (*) Bit+ Bit [":-"]
    //   Bit+ = (*) Bit+ Bit ["["]
    //   Bit+ = (*) Bit+ Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit+ Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Item = (*) Application "." [EOF]
    //   Item = (*) Application "." ["["]
    //   Item = (*) Application "." [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Item = (*) Application "." [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Item = (*) Application "." [r#"[A-Za-z0-9_]+:"#]
    //   Item = (*) Application "." [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Item = (*) Rule [EOF]
    //   Item = (*) Rule ["["]
    //   Item = (*) Rule [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Item = (*) Rule [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Item = (*) Rule [r#"[A-Za-z0-9_]+:"#]
    //   Item = (*) Rule [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Item+ = (*) Item [EOF]
    //   Item+ = (*) Item ["["]
    //   Item+ = (*) Item [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Item+ = (*) Item [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Item+ = (*) Item [r#"[A-Za-z0-9_]+:"#]
    //   Item+ = (*) Item [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Item+ = (*) Item+ Item [EOF]
    //   Item+ = (*) Item+ Item ["["]
    //   Item+ = (*) Item+ Item [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Item+ = (*) Item+ Item [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Item+ = (*) Item+ Item [r#"[A-Za-z0-9_]+:"#]
    //   Item+ = (*) Item+ Item [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Program = (*) Item+ [EOF]
    //   Rule = (*) Application ":-" Fact "." [EOF]
    //   Rule = (*) Application ":-" Fact "." ["["]
    //   Rule = (*) Application ":-" Fact "." [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Rule = (*) Application ":-" Fact "." [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Rule = (*) Application ":-" Fact "." [r#"[A-Za-z0-9_]+:"#]
    //   Rule = (*) Application ":-" Fact "." [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["."]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [":-"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["["]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   __Program = (*) Program [EOF]
    //
    //   "[" -> Shift(S9)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Shift(S10)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Shift(S11)
    //   r#"[A-Za-z0-9_]+:"# -> Shift(S12)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Shift(S13)
    //
    //   Application -> S1
    //   Bit -> S2
    //   Bit+ -> S3
    //   Item -> S4
    //   Item+ -> S5
    //   Program -> S6
    //   Rule -> S7
    //   Variable -> S8
    pub fn __state0<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (6, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym0 = &mut Some((__tok0));
                __result = try!(__state9(input, __lookbehind, __tokens, __sym0));
            }
            Some((_, (10, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym0 = &mut Some((__tok0));
                __result = try!(__state10(input, __lookbehind, __tokens, __sym0));
            }
            Some((_, (11, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym0 = &mut Some((__tok0));
                __result = try!(__state11(input, __lookbehind, __tokens, __sym0));
            }
            Some((_, (12, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym0 = &mut Some((__tok0));
                __result = try!(__state12(input, __lookbehind, __tokens, __sym0));
            }
            Some((_, (13, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym0 = &mut Some((__tok0));
                __result = try!(__state13(input, __lookbehind, __tokens, __sym0));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        loop {
            let (__lookbehind, __lookahead, __nt) = __result;
            match __nt {
                __Nonterminal::Application(__nt) => {
                    let __sym0 = &mut Some(__nt);
                    __result = try!(__state1(input, __lookbehind, __tokens, __lookahead, __sym0));
                }
                __Nonterminal::Bit(__nt) => {
                    let __sym0 = &mut Some(__nt);
                    __result = try!(__state2(input, __lookbehind, __tokens, __lookahead, __sym0));
                }
                __Nonterminal::Bit_2b(__nt) => {
                    let __sym0 = &mut Some(__nt);
                    __result = try!(__state3(input, __lookbehind, __tokens, __lookahead, __sym0));
                }
                __Nonterminal::Item(__nt) => {
                    let __sym0 = &mut Some(__nt);
                    __result = try!(__state4(input, __lookbehind, __tokens, __lookahead, __sym0));
                }
                __Nonterminal::Item_2b(__nt) => {
                    let __sym0 = &mut Some(__nt);
                    __result = try!(__state5(input, __lookbehind, __tokens, __lookahead, __sym0));
                }
                __Nonterminal::Program(__nt) => {
                    let __sym0 = &mut Some(__nt);
                    __result = try!(__state6(input, __lookbehind, __tokens, __lookahead, __sym0));
                }
                __Nonterminal::Rule(__nt) => {
                    let __sym0 = &mut Some(__nt);
                    __result = try!(__state7(input, __lookbehind, __tokens, __lookahead, __sym0));
                }
                __Nonterminal::Variable(__nt) => {
                    let __sym0 = &mut Some(__nt);
                    __result = try!(__state8(input, __lookbehind, __tokens, __lookahead, __sym0));
                }
                _ => {
                    return Ok((__lookbehind, __lookahead, __nt));
                }
            }
        }
    }

    // State 1
    //   Item = Application (*) "." [EOF]
    //   Item = Application (*) "." ["["]
    //   Item = Application (*) "." [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Item = Application (*) "." [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Item = Application (*) "." [r#"[A-Za-z0-9_]+:"#]
    //   Item = Application (*) "." [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Rule = Application (*) ":-" Fact "." [EOF]
    //   Rule = Application (*) ":-" Fact "." ["["]
    //   Rule = Application (*) ":-" Fact "." [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Rule = Application (*) ":-" Fact "." [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Rule = Application (*) ":-" Fact "." [r#"[A-Za-z0-9_]+:"#]
    //   Rule = Application (*) ":-" Fact "." [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "." -> Shift(S14)
    //   ":-" -> Shift(S15)
    //
    pub fn __state1<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Application>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (2, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state14(input, __lookbehind, __tokens, __sym0, __sym1));
            }
            Some((_, (3, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state15(input, __lookbehind, __tokens, __sym0, __sym1));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        return Ok(__result);
    }

    // State 2
    //   Bit+ = Bit (*) ["."]
    //   Bit+ = Bit (*) [":-"]
    //   Bit+ = Bit (*) ["["]
    //   Bit+ = Bit (*) [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = Bit (*) [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = Bit (*) [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = Bit (*) [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "." -> Reduce(Bit+ = Bit => ActionFn(22);)
    //   ":-" -> Reduce(Bit+ = Bit => ActionFn(22);)
    //   "[" -> Reduce(Bit+ = Bit => ActionFn(22);)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Reduce(Bit+ = Bit => ActionFn(22);)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Reduce(Bit+ = Bit => ActionFn(22);)
    //   r#"[A-Za-z0-9_]+:"# -> Reduce(Bit+ = Bit => ActionFn(22);)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Reduce(Bit+ = Bit => ActionFn(22);)
    //
    pub fn __state2<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Bit>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (2, _), _)) |
            Some((_, (3, _), _)) |
            Some((_, (6, _), _)) |
            Some((_, (10, _), _)) |
            Some((_, (11, _), _)) |
            Some((_, (12, _), _)) |
            Some((_, (13, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action22(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Bit_2b(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 3
    //   Application = Bit+ (*) ["."]
    //   Application = Bit+ (*) [":-"]
    //   Bit = (*) Variable ["."]
    //   Bit = (*) Variable [":-"]
    //   Bit = (*) Variable ["["]
    //   Bit = (*) Variable [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) Variable [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) Variable [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) Variable [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" ["."]
    //   Bit = (*) "[" Fact "]" [":-"]
    //   Bit = (*) "[" Fact "]" ["["]
    //   Bit = (*) "[" Fact "]" [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) "[" Fact "]" [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["."]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [":-"]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["["]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["."]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [":-"]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["["]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["."]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [":-"]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["["]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = Bit+ (*) Bit ["."]
    //   Bit+ = Bit+ (*) Bit [":-"]
    //   Bit+ = Bit+ (*) Bit ["["]
    //   Bit+ = Bit+ (*) Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = Bit+ (*) Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = Bit+ (*) Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = Bit+ (*) Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["."]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [":-"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["["]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "." -> Reduce(Application = Bit+ => ActionFn(15);)
    //   ":-" -> Reduce(Application = Bit+ => ActionFn(15);)
    //   "[" -> Shift(S9)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Shift(S10)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Shift(S11)
    //   r#"[A-Za-z0-9_]+:"# -> Shift(S12)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Shift(S13)
    //
    //   Bit -> S16
    //   Variable -> S8
    pub fn __state3<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<::std::vec::Vec<Bit>>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (6, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state9(input, __lookbehind, __tokens, __sym1));
            }
            Some((_, (10, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state10(input, __lookbehind, __tokens, __sym1));
            }
            Some((_, (11, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state11(input, __lookbehind, __tokens, __sym1));
            }
            Some((_, (12, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state12(input, __lookbehind, __tokens, __sym1));
            }
            Some((_, (13, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state13(input, __lookbehind, __tokens, __sym1));
            }
            Some((_, (2, _), _)) |
            Some((_, (3, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action15(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Application(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        while __sym0.is_some() {
            let (__lookbehind, __lookahead, __nt) = __result;
            match __nt {
                __Nonterminal::Bit(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state16(input, __lookbehind, __tokens, __lookahead, __sym0, __sym1));
                }
                __Nonterminal::Variable(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state8(input, __lookbehind, __tokens, __lookahead, __sym1));
                }
                _ => {
                    return Ok((__lookbehind, __lookahead, __nt));
                }
            }
        }
        return Ok(__result);
    }

    // State 4
    //   Item+ = Item (*) [EOF]
    //   Item+ = Item (*) ["["]
    //   Item+ = Item (*) [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Item+ = Item (*) [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Item+ = Item (*) [r#"[A-Za-z0-9_]+:"#]
    //   Item+ = Item (*) [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   EOF -> Reduce(Item+ = Item => ActionFn(24);)
    //   "[" -> Reduce(Item+ = Item => ActionFn(24);)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Reduce(Item+ = Item => ActionFn(24);)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Reduce(Item+ = Item => ActionFn(24);)
    //   r#"[A-Za-z0-9_]+:"# -> Reduce(Item+ = Item => ActionFn(24);)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Reduce(Item+ = Item => ActionFn(24);)
    //
    pub fn __state4<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Item>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            None |
            Some((_, (6, _), _)) |
            Some((_, (10, _), _)) |
            Some((_, (11, _), _)) |
            Some((_, (12, _), _)) |
            Some((_, (13, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action24(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Item_2b(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 5
    //   Application = (*) Bit+ ["."]
    //   Application = (*) Bit+ [":-"]
    //   Bit = (*) Variable ["."]
    //   Bit = (*) Variable [":-"]
    //   Bit = (*) Variable ["["]
    //   Bit = (*) Variable [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) Variable [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) Variable [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) Variable [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" ["."]
    //   Bit = (*) "[" Fact "]" [":-"]
    //   Bit = (*) "[" Fact "]" ["["]
    //   Bit = (*) "[" Fact "]" [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) "[" Fact "]" [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["."]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [":-"]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["["]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["."]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [":-"]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["["]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["."]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [":-"]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["["]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit ["."]
    //   Bit+ = (*) Bit [":-"]
    //   Bit+ = (*) Bit ["["]
    //   Bit+ = (*) Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit ["."]
    //   Bit+ = (*) Bit+ Bit [":-"]
    //   Bit+ = (*) Bit+ Bit ["["]
    //   Bit+ = (*) Bit+ Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit+ Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Item = (*) Application "." [EOF]
    //   Item = (*) Application "." ["["]
    //   Item = (*) Application "." [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Item = (*) Application "." [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Item = (*) Application "." [r#"[A-Za-z0-9_]+:"#]
    //   Item = (*) Application "." [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Item = (*) Rule [EOF]
    //   Item = (*) Rule ["["]
    //   Item = (*) Rule [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Item = (*) Rule [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Item = (*) Rule [r#"[A-Za-z0-9_]+:"#]
    //   Item = (*) Rule [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Item+ = Item+ (*) Item [EOF]
    //   Item+ = Item+ (*) Item ["["]
    //   Item+ = Item+ (*) Item [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Item+ = Item+ (*) Item [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Item+ = Item+ (*) Item [r#"[A-Za-z0-9_]+:"#]
    //   Item+ = Item+ (*) Item [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Program = Item+ (*) [EOF]
    //   Rule = (*) Application ":-" Fact "." [EOF]
    //   Rule = (*) Application ":-" Fact "." ["["]
    //   Rule = (*) Application ":-" Fact "." [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Rule = (*) Application ":-" Fact "." [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Rule = (*) Application ":-" Fact "." [r#"[A-Za-z0-9_]+:"#]
    //   Rule = (*) Application ":-" Fact "." [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["."]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [":-"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["["]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   EOF -> Reduce(Program = Item+ => ActionFn(1);)
    //   "[" -> Shift(S9)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Shift(S10)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Shift(S11)
    //   r#"[A-Za-z0-9_]+:"# -> Shift(S12)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Shift(S13)
    //
    //   Application -> S1
    //   Bit -> S2
    //   Bit+ -> S3
    //   Item -> S17
    //   Rule -> S7
    //   Variable -> S8
    pub fn __state5<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<::std::vec::Vec<Item>>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (6, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state9(input, __lookbehind, __tokens, __sym1));
            }
            Some((_, (10, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state10(input, __lookbehind, __tokens, __sym1));
            }
            Some((_, (11, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state11(input, __lookbehind, __tokens, __sym1));
            }
            Some((_, (12, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state12(input, __lookbehind, __tokens, __sym1));
            }
            Some((_, (13, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state13(input, __lookbehind, __tokens, __sym1));
            }
            None => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action1(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Program(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        while __sym0.is_some() {
            let (__lookbehind, __lookahead, __nt) = __result;
            match __nt {
                __Nonterminal::Application(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state1(input, __lookbehind, __tokens, __lookahead, __sym1));
                }
                __Nonterminal::Bit(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state2(input, __lookbehind, __tokens, __lookahead, __sym1));
                }
                __Nonterminal::Bit_2b(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state3(input, __lookbehind, __tokens, __lookahead, __sym1));
                }
                __Nonterminal::Item(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state17(input, __lookbehind, __tokens, __lookahead, __sym0, __sym1));
                }
                __Nonterminal::Rule(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state7(input, __lookbehind, __tokens, __lookahead, __sym1));
                }
                __Nonterminal::Variable(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state8(input, __lookbehind, __tokens, __lookahead, __sym1));
                }
                _ => {
                    return Ok((__lookbehind, __lookahead, __nt));
                }
            }
        }
        return Ok(__result);
    }

    // State 6
    //   __Program = Program (*) [EOF]
    //
    //   EOF -> Reduce(__Program = Program => ActionFn(0);)
    //
    pub fn __state6<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Program>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            None => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action0(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::____Program(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 7
    //   Item = Rule (*) [EOF]
    //   Item = Rule (*) ["["]
    //   Item = Rule (*) [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Item = Rule (*) [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Item = Rule (*) [r#"[A-Za-z0-9_]+:"#]
    //   Item = Rule (*) [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   EOF -> Reduce(Item = Rule => ActionFn(3);)
    //   "[" -> Reduce(Item = Rule => ActionFn(3);)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Reduce(Item = Rule => ActionFn(3);)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Reduce(Item = Rule => ActionFn(3);)
    //   r#"[A-Za-z0-9_]+:"# -> Reduce(Item = Rule => ActionFn(3);)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Reduce(Item = Rule => ActionFn(3);)
    //
    pub fn __state7<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Rule>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            None |
            Some((_, (6, _), _)) |
            Some((_, (10, _), _)) |
            Some((_, (11, _), _)) |
            Some((_, (12, _), _)) |
            Some((_, (13, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action3(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Item(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 8
    //   Bit = Variable (*) ["."]
    //   Bit = Variable (*) [":-"]
    //   Bit = Variable (*) ["["]
    //   Bit = Variable (*) [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = Variable (*) [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = Variable (*) [r#"[A-Za-z0-9_]+:"#]
    //   Bit = Variable (*) [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "." -> Reduce(Bit = Variable => ActionFn(19);)
    //   ":-" -> Reduce(Bit = Variable => ActionFn(19);)
    //   "[" -> Reduce(Bit = Variable => ActionFn(19);)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Reduce(Bit = Variable => ActionFn(19);)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Reduce(Bit = Variable => ActionFn(19);)
    //   r#"[A-Za-z0-9_]+:"# -> Reduce(Bit = Variable => ActionFn(19);)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Reduce(Bit = Variable => ActionFn(19);)
    //
    pub fn __state8<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Variable>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (2, _), _)) |
            Some((_, (3, _), _)) |
            Some((_, (6, _), _)) |
            Some((_, (10, _), _)) |
            Some((_, (11, _), _)) |
            Some((_, (12, _), _)) |
            Some((_, (13, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action19(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Bit(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 9
    //   Application = (*) Bit+ ["=>"]
    //   Bit = (*) Variable ["=>"]
    //   Bit = (*) Variable ["["]
    //   Bit = (*) Variable [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) Variable [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) Variable [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) Variable [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" ["=>"]
    //   Bit = (*) "[" Fact "]" ["["]
    //   Bit = (*) "[" Fact "]" [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) "[" Fact "]" [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = "[" (*) Fact "]" ["."]
    //   Bit = "[" (*) Fact "]" [":-"]
    //   Bit = "[" (*) Fact "]" ["["]
    //   Bit = "[" (*) Fact "]" [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = "[" (*) Fact "]" [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = "[" (*) Fact "]" [r#"[A-Za-z0-9_]+:"#]
    //   Bit = "[" (*) Fact "]" [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["=>"]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["["]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["=>"]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["["]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["=>"]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["["]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit ["=>"]
    //   Bit+ = (*) Bit ["["]
    //   Bit+ = (*) Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit ["=>"]
    //   Bit+ = (*) Bit+ Bit ["["]
    //   Bit+ = (*) Bit+ Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit+ Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Fact = (*) FactAnd ["]"]
    //   FactAnd = (*) FactAnd ";" FactOr [";"]
    //   FactAnd = (*) FactAnd ";" FactOr ["]"]
    //   FactAnd = (*) FactOr [";"]
    //   FactAnd = (*) FactOr ["]"]
    //   FactApply = (*) Application ["=>"]
    //   FactFunc = (*) FactApply "=>" FactFunc [","]
    //   FactFunc = (*) FactApply "=>" FactFunc [";"]
    //   FactFunc = (*) FactApply "=>" FactFunc ["]"]
    //   FactFunc = (*) Variable "->" FactFunc [","]
    //   FactFunc = (*) Variable "->" FactFunc [";"]
    //   FactFunc = (*) Variable "->" FactFunc ["]"]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [","]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [";"]
    //   FactFunc = (*) "exists" Variable "->" FactFunc ["]"]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [","]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [";"]
    //   FactFunc = (*) "forall" Variable "->" FactFunc ["]"]
    //   FactOr = (*) FactFunc [","]
    //   FactOr = (*) FactFunc [";"]
    //   FactOr = (*) FactFunc ["]"]
    //   FactOr = (*) FactOr "," FactFunc [","]
    //   FactOr = (*) FactOr "," FactFunc [";"]
    //   FactOr = (*) FactOr "," FactFunc ["]"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["->"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["=>"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["["]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "[" -> Shift(S27)
    //   "exists" -> Shift(S28)
    //   "forall" -> Shift(S29)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Shift(S30)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Shift(S31)
    //   r#"[A-Za-z0-9_]+:"# -> Shift(S32)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Shift(S33)
    //
    //   Application -> S18
    //   Bit -> S19
    //   Bit+ -> S20
    //   Fact -> S21
    //   FactAnd -> S22
    //   FactApply -> S23
    //   FactFunc -> S24
    //   FactOr -> S25
    //   Variable -> S26
    pub fn __state9<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (6, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state27(input, __lookbehind, __tokens, __sym1));
            }
            Some((_, (8, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state28(input, __lookbehind, __tokens, __sym1));
            }
            Some((_, (9, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state29(input, __lookbehind, __tokens, __sym1));
            }
            Some((_, (10, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state30(input, __lookbehind, __tokens, __sym1));
            }
            Some((_, (11, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state31(input, __lookbehind, __tokens, __sym1));
            }
            Some((_, (12, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state32(input, __lookbehind, __tokens, __sym1));
            }
            Some((_, (13, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state33(input, __lookbehind, __tokens, __sym1));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        while __sym0.is_some() {
            let (__lookbehind, __lookahead, __nt) = __result;
            match __nt {
                __Nonterminal::Application(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state18(input, __lookbehind, __tokens, __lookahead, __sym1));
                }
                __Nonterminal::Bit(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state19(input, __lookbehind, __tokens, __lookahead, __sym1));
                }
                __Nonterminal::Bit_2b(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state20(input, __lookbehind, __tokens, __lookahead, __sym1));
                }
                __Nonterminal::Fact(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state21(input, __lookbehind, __tokens, __lookahead, __sym0, __sym1));
                }
                __Nonterminal::FactAnd(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state22(input, __lookbehind, __tokens, __lookahead, __sym1));
                }
                __Nonterminal::FactApply(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state23(input, __lookbehind, __tokens, __lookahead, __sym1));
                }
                __Nonterminal::FactFunc(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state24(input, __lookbehind, __tokens, __lookahead, __sym1));
                }
                __Nonterminal::FactOr(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state25(input, __lookbehind, __tokens, __lookahead, __sym1));
                }
                __Nonterminal::Variable(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state26(input, __lookbehind, __tokens, __lookahead, __sym1));
                }
                _ => {
                    return Ok((__lookbehind, __lookahead, __nt));
                }
            }
        }
        return Ok(__result);
    }

    // State 10
    //   Bit = r#"[-!@#$%^&*=+/?~\\\\;,.]+"# (*) ["."]
    //   Bit = r#"[-!@#$%^&*=+/?~\\\\;,.]+"# (*) [":-"]
    //   Bit = r#"[-!@#$%^&*=+/?~\\\\;,.]+"# (*) ["["]
    //   Bit = r#"[-!@#$%^&*=+/?~\\\\;,.]+"# (*) [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = r#"[-!@#$%^&*=+/?~\\\\;,.]+"# (*) [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = r#"[-!@#$%^&*=+/?~\\\\;,.]+"# (*) [r#"[A-Za-z0-9_]+:"#]
    //   Bit = r#"[-!@#$%^&*=+/?~\\\\;,.]+"# (*) [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "." -> Reduce(Bit = r#"[-!@#$%^&*=+/?~\\\\;,.]+"# => ActionFn(17);)
    //   ":-" -> Reduce(Bit = r#"[-!@#$%^&*=+/?~\\\\;,.]+"# => ActionFn(17);)
    //   "[" -> Reduce(Bit = r#"[-!@#$%^&*=+/?~\\\\;,.]+"# => ActionFn(17);)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Reduce(Bit = r#"[-!@#$%^&*=+/?~\\\\;,.]+"# => ActionFn(17);)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Reduce(Bit = r#"[-!@#$%^&*=+/?~\\\\;,.]+"# => ActionFn(17);)
    //   r#"[A-Za-z0-9_]+:"# -> Reduce(Bit = r#"[-!@#$%^&*=+/?~\\\\;,.]+"# => ActionFn(17);)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Reduce(Bit = r#"[-!@#$%^&*=+/?~\\\\;,.]+"# => ActionFn(17);)
    //
    pub fn __state10<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (2, _), _)) |
            Some((_, (3, _), _)) |
            Some((_, (6, _), _)) |
            Some((_, (10, _), _)) |
            Some((_, (11, _), _)) |
            Some((_, (12, _), _)) |
            Some((_, (13, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action17(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Bit(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 11
    //   Variable = r#"[A-Z][_A-Za-z0-9]*"# (*) ["."]
    //   Variable = r#"[A-Z][_A-Za-z0-9]*"# (*) [":-"]
    //   Variable = r#"[A-Z][_A-Za-z0-9]*"# (*) ["["]
    //   Variable = r#"[A-Z][_A-Za-z0-9]*"# (*) [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Variable = r#"[A-Z][_A-Za-z0-9]*"# (*) [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Variable = r#"[A-Z][_A-Za-z0-9]*"# (*) [r#"[A-Za-z0-9_]+:"#]
    //   Variable = r#"[A-Z][_A-Za-z0-9]*"# (*) [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "." -> Reduce(Variable = r#"[A-Z][_A-Za-z0-9]*"# => ActionFn(21);)
    //   ":-" -> Reduce(Variable = r#"[A-Z][_A-Za-z0-9]*"# => ActionFn(21);)
    //   "[" -> Reduce(Variable = r#"[A-Z][_A-Za-z0-9]*"# => ActionFn(21);)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Reduce(Variable = r#"[A-Z][_A-Za-z0-9]*"# => ActionFn(21);)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Reduce(Variable = r#"[A-Z][_A-Za-z0-9]*"# => ActionFn(21);)
    //   r#"[A-Za-z0-9_]+:"# -> Reduce(Variable = r#"[A-Z][_A-Za-z0-9]*"# => ActionFn(21);)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Reduce(Variable = r#"[A-Z][_A-Za-z0-9]*"# => ActionFn(21);)
    //
    pub fn __state11<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (2, _), _)) |
            Some((_, (3, _), _)) |
            Some((_, (6, _), _)) |
            Some((_, (10, _), _)) |
            Some((_, (11, _), _)) |
            Some((_, (12, _), _)) |
            Some((_, (13, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action21(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Variable(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 12
    //   Bit = r#"[A-Za-z0-9_]+:"# (*) ["."]
    //   Bit = r#"[A-Za-z0-9_]+:"# (*) [":-"]
    //   Bit = r#"[A-Za-z0-9_]+:"# (*) ["["]
    //   Bit = r#"[A-Za-z0-9_]+:"# (*) [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = r#"[A-Za-z0-9_]+:"# (*) [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = r#"[A-Za-z0-9_]+:"# (*) [r#"[A-Za-z0-9_]+:"#]
    //   Bit = r#"[A-Za-z0-9_]+:"# (*) [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "." -> Reduce(Bit = r#"[A-Za-z0-9_]+:"# => ActionFn(16);)
    //   ":-" -> Reduce(Bit = r#"[A-Za-z0-9_]+:"# => ActionFn(16);)
    //   "[" -> Reduce(Bit = r#"[A-Za-z0-9_]+:"# => ActionFn(16);)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Reduce(Bit = r#"[A-Za-z0-9_]+:"# => ActionFn(16);)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Reduce(Bit = r#"[A-Za-z0-9_]+:"# => ActionFn(16);)
    //   r#"[A-Za-z0-9_]+:"# -> Reduce(Bit = r#"[A-Za-z0-9_]+:"# => ActionFn(16);)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Reduce(Bit = r#"[A-Za-z0-9_]+:"# => ActionFn(16);)
    //
    pub fn __state12<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (2, _), _)) |
            Some((_, (3, _), _)) |
            Some((_, (6, _), _)) |
            Some((_, (10, _), _)) |
            Some((_, (11, _), _)) |
            Some((_, (12, _), _)) |
            Some((_, (13, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action16(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Bit(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 13
    //   Bit = r#"[a-z_][_A-Za-z0-9]*"# (*) ["."]
    //   Bit = r#"[a-z_][_A-Za-z0-9]*"# (*) [":-"]
    //   Bit = r#"[a-z_][_A-Za-z0-9]*"# (*) ["["]
    //   Bit = r#"[a-z_][_A-Za-z0-9]*"# (*) [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = r#"[a-z_][_A-Za-z0-9]*"# (*) [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = r#"[a-z_][_A-Za-z0-9]*"# (*) [r#"[A-Za-z0-9_]+:"#]
    //   Bit = r#"[a-z_][_A-Za-z0-9]*"# (*) [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "." -> Reduce(Bit = r#"[a-z_][_A-Za-z0-9]*"# => ActionFn(18);)
    //   ":-" -> Reduce(Bit = r#"[a-z_][_A-Za-z0-9]*"# => ActionFn(18);)
    //   "[" -> Reduce(Bit = r#"[a-z_][_A-Za-z0-9]*"# => ActionFn(18);)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Reduce(Bit = r#"[a-z_][_A-Za-z0-9]*"# => ActionFn(18);)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Reduce(Bit = r#"[a-z_][_A-Za-z0-9]*"# => ActionFn(18);)
    //   r#"[A-Za-z0-9_]+:"# -> Reduce(Bit = r#"[a-z_][_A-Za-z0-9]*"# => ActionFn(18);)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Reduce(Bit = r#"[a-z_][_A-Za-z0-9]*"# => ActionFn(18);)
    //
    pub fn __state13<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (2, _), _)) |
            Some((_, (3, _), _)) |
            Some((_, (6, _), _)) |
            Some((_, (10, _), _)) |
            Some((_, (11, _), _)) |
            Some((_, (12, _), _)) |
            Some((_, (13, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action18(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Bit(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 14
    //   Item = Application "." (*) [EOF]
    //   Item = Application "." (*) ["["]
    //   Item = Application "." (*) [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Item = Application "." (*) [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Item = Application "." (*) [r#"[A-Za-z0-9_]+:"#]
    //   Item = Application "." (*) [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   EOF -> Reduce(Item = Application, "." => ActionFn(2);)
    //   "[" -> Reduce(Item = Application, "." => ActionFn(2);)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Reduce(Item = Application, "." => ActionFn(2);)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Reduce(Item = Application, "." => ActionFn(2);)
    //   r#"[A-Za-z0-9_]+:"# -> Reduce(Item = Application, "." => ActionFn(2);)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Reduce(Item = Application, "." => ActionFn(2);)
    //
    pub fn __state14<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<Application>,
        __sym1: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            None |
            Some((_, (6, _), _)) |
            Some((_, (10, _), _)) |
            Some((_, (11, _), _)) |
            Some((_, (12, _), _)) |
            Some((_, (13, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __sym1 = __sym1.take().unwrap();
                let __nt = super::__action2(input, __sym0, __sym1, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Item(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 15
    //   Application = (*) Bit+ ["=>"]
    //   Bit = (*) Variable ["=>"]
    //   Bit = (*) Variable ["["]
    //   Bit = (*) Variable [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) Variable [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) Variable [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) Variable [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" ["=>"]
    //   Bit = (*) "[" Fact "]" ["["]
    //   Bit = (*) "[" Fact "]" [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) "[" Fact "]" [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["=>"]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["["]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["=>"]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["["]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["=>"]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["["]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit ["=>"]
    //   Bit+ = (*) Bit ["["]
    //   Bit+ = (*) Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit ["=>"]
    //   Bit+ = (*) Bit+ Bit ["["]
    //   Bit+ = (*) Bit+ Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit+ Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Fact = (*) FactAnd ["."]
    //   FactAnd = (*) FactAnd ";" FactOr ["."]
    //   FactAnd = (*) FactAnd ";" FactOr [";"]
    //   FactAnd = (*) FactOr ["."]
    //   FactAnd = (*) FactOr [";"]
    //   FactApply = (*) Application ["=>"]
    //   FactFunc = (*) FactApply "=>" FactFunc [","]
    //   FactFunc = (*) FactApply "=>" FactFunc ["."]
    //   FactFunc = (*) FactApply "=>" FactFunc [";"]
    //   FactFunc = (*) Variable "->" FactFunc [","]
    //   FactFunc = (*) Variable "->" FactFunc ["."]
    //   FactFunc = (*) Variable "->" FactFunc [";"]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [","]
    //   FactFunc = (*) "exists" Variable "->" FactFunc ["."]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [";"]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [","]
    //   FactFunc = (*) "forall" Variable "->" FactFunc ["."]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [";"]
    //   FactOr = (*) FactFunc [","]
    //   FactOr = (*) FactFunc ["."]
    //   FactOr = (*) FactFunc [";"]
    //   FactOr = (*) FactOr "," FactFunc [","]
    //   FactOr = (*) FactOr "," FactFunc ["."]
    //   FactOr = (*) FactOr "," FactFunc [";"]
    //   Rule = Application ":-" (*) Fact "." [EOF]
    //   Rule = Application ":-" (*) Fact "." ["["]
    //   Rule = Application ":-" (*) Fact "." [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Rule = Application ":-" (*) Fact "." [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Rule = Application ":-" (*) Fact "." [r#"[A-Za-z0-9_]+:"#]
    //   Rule = Application ":-" (*) Fact "." [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["->"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["=>"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["["]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "[" -> Shift(S27)
    //   "exists" -> Shift(S40)
    //   "forall" -> Shift(S41)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Shift(S30)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Shift(S31)
    //   r#"[A-Za-z0-9_]+:"# -> Shift(S32)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Shift(S33)
    //
    //   Application -> S18
    //   Bit -> S19
    //   Bit+ -> S20
    //   Fact -> S34
    //   FactAnd -> S35
    //   FactApply -> S36
    //   FactFunc -> S37
    //   FactOr -> S38
    //   Variable -> S39
    pub fn __state15<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<Application>,
        __sym1: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (6, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state27(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (8, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state40(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (9, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state41(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (10, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state30(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (11, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state31(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (12, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state32(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (13, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state33(input, __lookbehind, __tokens, __sym2));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        while __sym1.is_some() {
            let (__lookbehind, __lookahead, __nt) = __result;
            match __nt {
                __Nonterminal::Application(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state18(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::Bit(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state19(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::Bit_2b(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state20(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::Fact(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state34(input, __lookbehind, __tokens, __lookahead, __sym0, __sym1, __sym2));
                }
                __Nonterminal::FactAnd(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state35(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::FactApply(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state36(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::FactFunc(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state37(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::FactOr(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state38(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::Variable(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state39(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                _ => {
                    return Ok((__lookbehind, __lookahead, __nt));
                }
            }
        }
        return Ok(__result);
    }

    // State 16
    //   Bit+ = Bit+ Bit (*) ["."]
    //   Bit+ = Bit+ Bit (*) [":-"]
    //   Bit+ = Bit+ Bit (*) ["["]
    //   Bit+ = Bit+ Bit (*) [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = Bit+ Bit (*) [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = Bit+ Bit (*) [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = Bit+ Bit (*) [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "." -> Reduce(Bit+ = Bit+, Bit => ActionFn(23);)
    //   ":-" -> Reduce(Bit+ = Bit+, Bit => ActionFn(23);)
    //   "[" -> Reduce(Bit+ = Bit+, Bit => ActionFn(23);)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Reduce(Bit+ = Bit+, Bit => ActionFn(23);)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Reduce(Bit+ = Bit+, Bit => ActionFn(23);)
    //   r#"[A-Za-z0-9_]+:"# -> Reduce(Bit+ = Bit+, Bit => ActionFn(23);)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Reduce(Bit+ = Bit+, Bit => ActionFn(23);)
    //
    pub fn __state16<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<::std::vec::Vec<Bit>>,
        __sym1: &mut Option<Bit>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (2, _), _)) |
            Some((_, (3, _), _)) |
            Some((_, (6, _), _)) |
            Some((_, (10, _), _)) |
            Some((_, (11, _), _)) |
            Some((_, (12, _), _)) |
            Some((_, (13, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __sym1 = __sym1.take().unwrap();
                let __nt = super::__action23(input, __sym0, __sym1, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Bit_2b(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 17
    //   Item+ = Item+ Item (*) [EOF]
    //   Item+ = Item+ Item (*) ["["]
    //   Item+ = Item+ Item (*) [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Item+ = Item+ Item (*) [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Item+ = Item+ Item (*) [r#"[A-Za-z0-9_]+:"#]
    //   Item+ = Item+ Item (*) [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   EOF -> Reduce(Item+ = Item+, Item => ActionFn(25);)
    //   "[" -> Reduce(Item+ = Item+, Item => ActionFn(25);)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Reduce(Item+ = Item+, Item => ActionFn(25);)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Reduce(Item+ = Item+, Item => ActionFn(25);)
    //   r#"[A-Za-z0-9_]+:"# -> Reduce(Item+ = Item+, Item => ActionFn(25);)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Reduce(Item+ = Item+, Item => ActionFn(25);)
    //
    pub fn __state17<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<::std::vec::Vec<Item>>,
        __sym1: &mut Option<Item>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            None |
            Some((_, (6, _), _)) |
            Some((_, (10, _), _)) |
            Some((_, (11, _), _)) |
            Some((_, (12, _), _)) |
            Some((_, (13, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __sym1 = __sym1.take().unwrap();
                let __nt = super::__action25(input, __sym0, __sym1, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Item_2b(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 18
    //   FactApply = Application (*) ["=>"]
    //
    //   "=>" -> Reduce(FactApply = Application => ActionFn(14);)
    //
    pub fn __state18<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Application>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (5, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action14(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::FactApply(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 19
    //   Bit+ = Bit (*) ["=>"]
    //   Bit+ = Bit (*) ["["]
    //   Bit+ = Bit (*) [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = Bit (*) [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = Bit (*) [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = Bit (*) [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "=>" -> Reduce(Bit+ = Bit => ActionFn(22);)
    //   "[" -> Reduce(Bit+ = Bit => ActionFn(22);)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Reduce(Bit+ = Bit => ActionFn(22);)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Reduce(Bit+ = Bit => ActionFn(22);)
    //   r#"[A-Za-z0-9_]+:"# -> Reduce(Bit+ = Bit => ActionFn(22);)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Reduce(Bit+ = Bit => ActionFn(22);)
    //
    pub fn __state19<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Bit>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (5, _), _)) |
            Some((_, (6, _), _)) |
            Some((_, (10, _), _)) |
            Some((_, (11, _), _)) |
            Some((_, (12, _), _)) |
            Some((_, (13, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action22(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Bit_2b(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 20
    //   Application = Bit+ (*) ["=>"]
    //   Bit = (*) Variable ["=>"]
    //   Bit = (*) Variable ["["]
    //   Bit = (*) Variable [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) Variable [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) Variable [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) Variable [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" ["=>"]
    //   Bit = (*) "[" Fact "]" ["["]
    //   Bit = (*) "[" Fact "]" [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) "[" Fact "]" [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["=>"]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["["]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["=>"]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["["]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["=>"]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["["]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = Bit+ (*) Bit ["=>"]
    //   Bit+ = Bit+ (*) Bit ["["]
    //   Bit+ = Bit+ (*) Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = Bit+ (*) Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = Bit+ (*) Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = Bit+ (*) Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["=>"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["["]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "=>" -> Reduce(Application = Bit+ => ActionFn(15);)
    //   "[" -> Shift(S27)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Shift(S30)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Shift(S44)
    //   r#"[A-Za-z0-9_]+:"# -> Shift(S32)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Shift(S33)
    //
    //   Bit -> S42
    //   Variable -> S43
    pub fn __state20<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<::std::vec::Vec<Bit>>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (6, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state27(input, __lookbehind, __tokens, __sym1));
            }
            Some((_, (10, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state30(input, __lookbehind, __tokens, __sym1));
            }
            Some((_, (11, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state44(input, __lookbehind, __tokens, __sym1));
            }
            Some((_, (12, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state32(input, __lookbehind, __tokens, __sym1));
            }
            Some((_, (13, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state33(input, __lookbehind, __tokens, __sym1));
            }
            Some((_, (5, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action15(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Application(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        while __sym0.is_some() {
            let (__lookbehind, __lookahead, __nt) = __result;
            match __nt {
                __Nonterminal::Bit(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state42(input, __lookbehind, __tokens, __lookahead, __sym0, __sym1));
                }
                __Nonterminal::Variable(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state43(input, __lookbehind, __tokens, __lookahead, __sym1));
                }
                _ => {
                    return Ok((__lookbehind, __lookahead, __nt));
                }
            }
        }
        return Ok(__result);
    }

    // State 21
    //   Bit = "[" Fact (*) "]" ["."]
    //   Bit = "[" Fact (*) "]" [":-"]
    //   Bit = "[" Fact (*) "]" ["["]
    //   Bit = "[" Fact (*) "]" [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = "[" Fact (*) "]" [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = "[" Fact (*) "]" [r#"[A-Za-z0-9_]+:"#]
    //   Bit = "[" Fact (*) "]" [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "]" -> Shift(S45)
    //
    pub fn __state21<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<&'input str>,
        __sym1: &mut Option<Fact>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (7, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state45(input, __lookbehind, __tokens, __sym0, __sym1, __sym2));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        return Ok(__result);
    }

    // State 22
    //   Fact = FactAnd (*) ["]"]
    //   FactAnd = FactAnd (*) ";" FactOr [";"]
    //   FactAnd = FactAnd (*) ";" FactOr ["]"]
    //
    //   ";" -> Shift(S46)
    //   "]" -> Reduce(Fact = FactAnd => ActionFn(5);)
    //
    pub fn __state22<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Fact>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (4, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state46(input, __lookbehind, __tokens, __sym0, __sym1));
            }
            Some((_, (7, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action5(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Fact(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        return Ok(__result);
    }

    // State 23
    //   FactFunc = FactApply (*) "=>" FactFunc [","]
    //   FactFunc = FactApply (*) "=>" FactFunc [";"]
    //   FactFunc = FactApply (*) "=>" FactFunc ["]"]
    //
    //   "=>" -> Shift(S47)
    //
    pub fn __state23<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Fact>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (5, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state47(input, __lookbehind, __tokens, __sym0, __sym1));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        return Ok(__result);
    }

    // State 24
    //   FactOr = FactFunc (*) [","]
    //   FactOr = FactFunc (*) [";"]
    //   FactOr = FactFunc (*) ["]"]
    //
    //   "," -> Reduce(FactOr = FactFunc => ActionFn(8);)
    //   ";" -> Reduce(FactOr = FactFunc => ActionFn(8);)
    //   "]" -> Reduce(FactOr = FactFunc => ActionFn(8);)
    //
    pub fn __state24<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Fact>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (0, _), _)) |
            Some((_, (4, _), _)) |
            Some((_, (7, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action8(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::FactOr(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 25
    //   FactAnd = FactOr (*) [";"]
    //   FactAnd = FactOr (*) ["]"]
    //   FactOr = FactOr (*) "," FactFunc [","]
    //   FactOr = FactOr (*) "," FactFunc [";"]
    //   FactOr = FactOr (*) "," FactFunc ["]"]
    //
    //   "," -> Shift(S48)
    //   ";" -> Reduce(FactAnd = FactOr => ActionFn(6);)
    //   "]" -> Reduce(FactAnd = FactOr => ActionFn(6);)
    //
    pub fn __state25<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Fact>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (0, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state48(input, __lookbehind, __tokens, __sym0, __sym1));
            }
            Some((_, (4, _), _)) |
            Some((_, (7, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action6(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::FactAnd(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        return Ok(__result);
    }

    // State 26
    //   Bit = Variable (*) ["=>"]
    //   Bit = Variable (*) ["["]
    //   Bit = Variable (*) [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = Variable (*) [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = Variable (*) [r#"[A-Za-z0-9_]+:"#]
    //   Bit = Variable (*) [r#"[a-z_][_A-Za-z0-9]*"#]
    //   FactFunc = Variable (*) "->" FactFunc [","]
    //   FactFunc = Variable (*) "->" FactFunc [";"]
    //   FactFunc = Variable (*) "->" FactFunc ["]"]
    //
    //   "->" -> Shift(S49)
    //   "=>" -> Reduce(Bit = Variable => ActionFn(19);)
    //   "[" -> Reduce(Bit = Variable => ActionFn(19);)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Reduce(Bit = Variable => ActionFn(19);)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Reduce(Bit = Variable => ActionFn(19);)
    //   r#"[A-Za-z0-9_]+:"# -> Reduce(Bit = Variable => ActionFn(19);)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Reduce(Bit = Variable => ActionFn(19);)
    //
    pub fn __state26<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Variable>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (1, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state49(input, __lookbehind, __tokens, __sym0, __sym1));
            }
            Some((_, (5, _), _)) |
            Some((_, (6, _), _)) |
            Some((_, (10, _), _)) |
            Some((_, (11, _), _)) |
            Some((_, (12, _), _)) |
            Some((_, (13, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action19(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Bit(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        return Ok(__result);
    }

    // State 27
    //   Application = (*) Bit+ ["=>"]
    //   Bit = (*) Variable ["=>"]
    //   Bit = (*) Variable ["["]
    //   Bit = (*) Variable [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) Variable [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) Variable [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) Variable [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" ["=>"]
    //   Bit = (*) "[" Fact "]" ["["]
    //   Bit = (*) "[" Fact "]" [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) "[" Fact "]" [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = "[" (*) Fact "]" ["=>"]
    //   Bit = "[" (*) Fact "]" ["["]
    //   Bit = "[" (*) Fact "]" [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = "[" (*) Fact "]" [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = "[" (*) Fact "]" [r#"[A-Za-z0-9_]+:"#]
    //   Bit = "[" (*) Fact "]" [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["=>"]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["["]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["=>"]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["["]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["=>"]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["["]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit ["=>"]
    //   Bit+ = (*) Bit ["["]
    //   Bit+ = (*) Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit ["=>"]
    //   Bit+ = (*) Bit+ Bit ["["]
    //   Bit+ = (*) Bit+ Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit+ Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Fact = (*) FactAnd ["]"]
    //   FactAnd = (*) FactAnd ";" FactOr [";"]
    //   FactAnd = (*) FactAnd ";" FactOr ["]"]
    //   FactAnd = (*) FactOr [";"]
    //   FactAnd = (*) FactOr ["]"]
    //   FactApply = (*) Application ["=>"]
    //   FactFunc = (*) FactApply "=>" FactFunc [","]
    //   FactFunc = (*) FactApply "=>" FactFunc [";"]
    //   FactFunc = (*) FactApply "=>" FactFunc ["]"]
    //   FactFunc = (*) Variable "->" FactFunc [","]
    //   FactFunc = (*) Variable "->" FactFunc [";"]
    //   FactFunc = (*) Variable "->" FactFunc ["]"]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [","]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [";"]
    //   FactFunc = (*) "exists" Variable "->" FactFunc ["]"]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [","]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [";"]
    //   FactFunc = (*) "forall" Variable "->" FactFunc ["]"]
    //   FactOr = (*) FactFunc [","]
    //   FactOr = (*) FactFunc [";"]
    //   FactOr = (*) FactFunc ["]"]
    //   FactOr = (*) FactOr "," FactFunc [","]
    //   FactOr = (*) FactOr "," FactFunc [";"]
    //   FactOr = (*) FactOr "," FactFunc ["]"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["->"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["=>"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["["]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "[" -> Shift(S27)
    //   "exists" -> Shift(S28)
    //   "forall" -> Shift(S29)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Shift(S30)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Shift(S31)
    //   r#"[A-Za-z0-9_]+:"# -> Shift(S32)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Shift(S33)
    //
    //   Application -> S18
    //   Bit -> S19
    //   Bit+ -> S20
    //   Fact -> S50
    //   FactAnd -> S22
    //   FactApply -> S23
    //   FactFunc -> S24
    //   FactOr -> S25
    //   Variable -> S26
    pub fn __state27<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (6, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state27(input, __lookbehind, __tokens, __sym1));
            }
            Some((_, (8, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state28(input, __lookbehind, __tokens, __sym1));
            }
            Some((_, (9, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state29(input, __lookbehind, __tokens, __sym1));
            }
            Some((_, (10, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state30(input, __lookbehind, __tokens, __sym1));
            }
            Some((_, (11, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state31(input, __lookbehind, __tokens, __sym1));
            }
            Some((_, (12, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state32(input, __lookbehind, __tokens, __sym1));
            }
            Some((_, (13, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state33(input, __lookbehind, __tokens, __sym1));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        while __sym0.is_some() {
            let (__lookbehind, __lookahead, __nt) = __result;
            match __nt {
                __Nonterminal::Application(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state18(input, __lookbehind, __tokens, __lookahead, __sym1));
                }
                __Nonterminal::Bit(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state19(input, __lookbehind, __tokens, __lookahead, __sym1));
                }
                __Nonterminal::Bit_2b(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state20(input, __lookbehind, __tokens, __lookahead, __sym1));
                }
                __Nonterminal::Fact(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state50(input, __lookbehind, __tokens, __lookahead, __sym0, __sym1));
                }
                __Nonterminal::FactAnd(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state22(input, __lookbehind, __tokens, __lookahead, __sym1));
                }
                __Nonterminal::FactApply(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state23(input, __lookbehind, __tokens, __lookahead, __sym1));
                }
                __Nonterminal::FactFunc(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state24(input, __lookbehind, __tokens, __lookahead, __sym1));
                }
                __Nonterminal::FactOr(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state25(input, __lookbehind, __tokens, __lookahead, __sym1));
                }
                __Nonterminal::Variable(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state26(input, __lookbehind, __tokens, __lookahead, __sym1));
                }
                _ => {
                    return Ok((__lookbehind, __lookahead, __nt));
                }
            }
        }
        return Ok(__result);
    }

    // State 28
    //   FactFunc = "exists" (*) Variable "->" FactFunc [","]
    //   FactFunc = "exists" (*) Variable "->" FactFunc [";"]
    //   FactFunc = "exists" (*) Variable "->" FactFunc ["]"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["->"]
    //
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Shift(S52)
    //
    //   Variable -> S51
    pub fn __state28<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (11, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state52(input, __lookbehind, __tokens, __sym1));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        while __sym0.is_some() {
            let (__lookbehind, __lookahead, __nt) = __result;
            match __nt {
                __Nonterminal::Variable(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state51(input, __lookbehind, __tokens, __lookahead, __sym0, __sym1));
                }
                _ => {
                    return Ok((__lookbehind, __lookahead, __nt));
                }
            }
        }
        return Ok(__result);
    }

    // State 29
    //   FactFunc = "forall" (*) Variable "->" FactFunc [","]
    //   FactFunc = "forall" (*) Variable "->" FactFunc [";"]
    //   FactFunc = "forall" (*) Variable "->" FactFunc ["]"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["->"]
    //
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Shift(S52)
    //
    //   Variable -> S53
    pub fn __state29<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (11, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state52(input, __lookbehind, __tokens, __sym1));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        while __sym0.is_some() {
            let (__lookbehind, __lookahead, __nt) = __result;
            match __nt {
                __Nonterminal::Variable(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state53(input, __lookbehind, __tokens, __lookahead, __sym0, __sym1));
                }
                _ => {
                    return Ok((__lookbehind, __lookahead, __nt));
                }
            }
        }
        return Ok(__result);
    }

    // State 30
    //   Bit = r#"[-!@#$%^&*=+/?~\\\\;,.]+"# (*) ["=>"]
    //   Bit = r#"[-!@#$%^&*=+/?~\\\\;,.]+"# (*) ["["]
    //   Bit = r#"[-!@#$%^&*=+/?~\\\\;,.]+"# (*) [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = r#"[-!@#$%^&*=+/?~\\\\;,.]+"# (*) [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = r#"[-!@#$%^&*=+/?~\\\\;,.]+"# (*) [r#"[A-Za-z0-9_]+:"#]
    //   Bit = r#"[-!@#$%^&*=+/?~\\\\;,.]+"# (*) [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "=>" -> Reduce(Bit = r#"[-!@#$%^&*=+/?~\\\\;,.]+"# => ActionFn(17);)
    //   "[" -> Reduce(Bit = r#"[-!@#$%^&*=+/?~\\\\;,.]+"# => ActionFn(17);)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Reduce(Bit = r#"[-!@#$%^&*=+/?~\\\\;,.]+"# => ActionFn(17);)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Reduce(Bit = r#"[-!@#$%^&*=+/?~\\\\;,.]+"# => ActionFn(17);)
    //   r#"[A-Za-z0-9_]+:"# -> Reduce(Bit = r#"[-!@#$%^&*=+/?~\\\\;,.]+"# => ActionFn(17);)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Reduce(Bit = r#"[-!@#$%^&*=+/?~\\\\;,.]+"# => ActionFn(17);)
    //
    pub fn __state30<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (5, _), _)) |
            Some((_, (6, _), _)) |
            Some((_, (10, _), _)) |
            Some((_, (11, _), _)) |
            Some((_, (12, _), _)) |
            Some((_, (13, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action17(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Bit(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 31
    //   Variable = r#"[A-Z][_A-Za-z0-9]*"# (*) ["->"]
    //   Variable = r#"[A-Z][_A-Za-z0-9]*"# (*) ["=>"]
    //   Variable = r#"[A-Z][_A-Za-z0-9]*"# (*) ["["]
    //   Variable = r#"[A-Z][_A-Za-z0-9]*"# (*) [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Variable = r#"[A-Z][_A-Za-z0-9]*"# (*) [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Variable = r#"[A-Z][_A-Za-z0-9]*"# (*) [r#"[A-Za-z0-9_]+:"#]
    //   Variable = r#"[A-Z][_A-Za-z0-9]*"# (*) [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "->" -> Reduce(Variable = r#"[A-Z][_A-Za-z0-9]*"# => ActionFn(21);)
    //   "=>" -> Reduce(Variable = r#"[A-Z][_A-Za-z0-9]*"# => ActionFn(21);)
    //   "[" -> Reduce(Variable = r#"[A-Z][_A-Za-z0-9]*"# => ActionFn(21);)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Reduce(Variable = r#"[A-Z][_A-Za-z0-9]*"# => ActionFn(21);)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Reduce(Variable = r#"[A-Z][_A-Za-z0-9]*"# => ActionFn(21);)
    //   r#"[A-Za-z0-9_]+:"# -> Reduce(Variable = r#"[A-Z][_A-Za-z0-9]*"# => ActionFn(21);)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Reduce(Variable = r#"[A-Z][_A-Za-z0-9]*"# => ActionFn(21);)
    //
    pub fn __state31<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (1, _), _)) |
            Some((_, (5, _), _)) |
            Some((_, (6, _), _)) |
            Some((_, (10, _), _)) |
            Some((_, (11, _), _)) |
            Some((_, (12, _), _)) |
            Some((_, (13, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action21(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Variable(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 32
    //   Bit = r#"[A-Za-z0-9_]+:"# (*) ["=>"]
    //   Bit = r#"[A-Za-z0-9_]+:"# (*) ["["]
    //   Bit = r#"[A-Za-z0-9_]+:"# (*) [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = r#"[A-Za-z0-9_]+:"# (*) [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = r#"[A-Za-z0-9_]+:"# (*) [r#"[A-Za-z0-9_]+:"#]
    //   Bit = r#"[A-Za-z0-9_]+:"# (*) [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "=>" -> Reduce(Bit = r#"[A-Za-z0-9_]+:"# => ActionFn(16);)
    //   "[" -> Reduce(Bit = r#"[A-Za-z0-9_]+:"# => ActionFn(16);)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Reduce(Bit = r#"[A-Za-z0-9_]+:"# => ActionFn(16);)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Reduce(Bit = r#"[A-Za-z0-9_]+:"# => ActionFn(16);)
    //   r#"[A-Za-z0-9_]+:"# -> Reduce(Bit = r#"[A-Za-z0-9_]+:"# => ActionFn(16);)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Reduce(Bit = r#"[A-Za-z0-9_]+:"# => ActionFn(16);)
    //
    pub fn __state32<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (5, _), _)) |
            Some((_, (6, _), _)) |
            Some((_, (10, _), _)) |
            Some((_, (11, _), _)) |
            Some((_, (12, _), _)) |
            Some((_, (13, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action16(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Bit(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 33
    //   Bit = r#"[a-z_][_A-Za-z0-9]*"# (*) ["=>"]
    //   Bit = r#"[a-z_][_A-Za-z0-9]*"# (*) ["["]
    //   Bit = r#"[a-z_][_A-Za-z0-9]*"# (*) [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = r#"[a-z_][_A-Za-z0-9]*"# (*) [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = r#"[a-z_][_A-Za-z0-9]*"# (*) [r#"[A-Za-z0-9_]+:"#]
    //   Bit = r#"[a-z_][_A-Za-z0-9]*"# (*) [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "=>" -> Reduce(Bit = r#"[a-z_][_A-Za-z0-9]*"# => ActionFn(18);)
    //   "[" -> Reduce(Bit = r#"[a-z_][_A-Za-z0-9]*"# => ActionFn(18);)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Reduce(Bit = r#"[a-z_][_A-Za-z0-9]*"# => ActionFn(18);)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Reduce(Bit = r#"[a-z_][_A-Za-z0-9]*"# => ActionFn(18);)
    //   r#"[A-Za-z0-9_]+:"# -> Reduce(Bit = r#"[a-z_][_A-Za-z0-9]*"# => ActionFn(18);)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Reduce(Bit = r#"[a-z_][_A-Za-z0-9]*"# => ActionFn(18);)
    //
    pub fn __state33<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (5, _), _)) |
            Some((_, (6, _), _)) |
            Some((_, (10, _), _)) |
            Some((_, (11, _), _)) |
            Some((_, (12, _), _)) |
            Some((_, (13, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action18(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Bit(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 34
    //   Rule = Application ":-" Fact (*) "." [EOF]
    //   Rule = Application ":-" Fact (*) "." ["["]
    //   Rule = Application ":-" Fact (*) "." [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Rule = Application ":-" Fact (*) "." [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Rule = Application ":-" Fact (*) "." [r#"[A-Za-z0-9_]+:"#]
    //   Rule = Application ":-" Fact (*) "." [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "." -> Shift(S54)
    //
    pub fn __state34<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Application>,
        __sym1: &mut Option<&'input str>,
        __sym2: &mut Option<Fact>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (2, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state54(input, __lookbehind, __tokens, __sym0, __sym1, __sym2, __sym3));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        return Ok(__result);
    }

    // State 35
    //   Fact = FactAnd (*) ["."]
    //   FactAnd = FactAnd (*) ";" FactOr ["."]
    //   FactAnd = FactAnd (*) ";" FactOr [";"]
    //
    //   "." -> Reduce(Fact = FactAnd => ActionFn(5);)
    //   ";" -> Shift(S55)
    //
    pub fn __state35<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Fact>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (4, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state55(input, __lookbehind, __tokens, __sym0, __sym1));
            }
            Some((_, (2, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action5(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Fact(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        return Ok(__result);
    }

    // State 36
    //   FactFunc = FactApply (*) "=>" FactFunc [","]
    //   FactFunc = FactApply (*) "=>" FactFunc ["."]
    //   FactFunc = FactApply (*) "=>" FactFunc [";"]
    //
    //   "=>" -> Shift(S56)
    //
    pub fn __state36<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Fact>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (5, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state56(input, __lookbehind, __tokens, __sym0, __sym1));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        return Ok(__result);
    }

    // State 37
    //   FactOr = FactFunc (*) [","]
    //   FactOr = FactFunc (*) ["."]
    //   FactOr = FactFunc (*) [";"]
    //
    //   "," -> Reduce(FactOr = FactFunc => ActionFn(8);)
    //   "." -> Reduce(FactOr = FactFunc => ActionFn(8);)
    //   ";" -> Reduce(FactOr = FactFunc => ActionFn(8);)
    //
    pub fn __state37<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Fact>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (0, _), _)) |
            Some((_, (2, _), _)) |
            Some((_, (4, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action8(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::FactOr(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 38
    //   FactAnd = FactOr (*) ["."]
    //   FactAnd = FactOr (*) [";"]
    //   FactOr = FactOr (*) "," FactFunc [","]
    //   FactOr = FactOr (*) "," FactFunc ["."]
    //   FactOr = FactOr (*) "," FactFunc [";"]
    //
    //   "," -> Shift(S57)
    //   "." -> Reduce(FactAnd = FactOr => ActionFn(6);)
    //   ";" -> Reduce(FactAnd = FactOr => ActionFn(6);)
    //
    pub fn __state38<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Fact>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (0, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state57(input, __lookbehind, __tokens, __sym0, __sym1));
            }
            Some((_, (2, _), _)) |
            Some((_, (4, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action6(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::FactAnd(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        return Ok(__result);
    }

    // State 39
    //   Bit = Variable (*) ["=>"]
    //   Bit = Variable (*) ["["]
    //   Bit = Variable (*) [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = Variable (*) [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = Variable (*) [r#"[A-Za-z0-9_]+:"#]
    //   Bit = Variable (*) [r#"[a-z_][_A-Za-z0-9]*"#]
    //   FactFunc = Variable (*) "->" FactFunc [","]
    //   FactFunc = Variable (*) "->" FactFunc ["."]
    //   FactFunc = Variable (*) "->" FactFunc [";"]
    //
    //   "->" -> Shift(S58)
    //   "=>" -> Reduce(Bit = Variable => ActionFn(19);)
    //   "[" -> Reduce(Bit = Variable => ActionFn(19);)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Reduce(Bit = Variable => ActionFn(19);)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Reduce(Bit = Variable => ActionFn(19);)
    //   r#"[A-Za-z0-9_]+:"# -> Reduce(Bit = Variable => ActionFn(19);)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Reduce(Bit = Variable => ActionFn(19);)
    //
    pub fn __state39<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Variable>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (1, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state58(input, __lookbehind, __tokens, __sym0, __sym1));
            }
            Some((_, (5, _), _)) |
            Some((_, (6, _), _)) |
            Some((_, (10, _), _)) |
            Some((_, (11, _), _)) |
            Some((_, (12, _), _)) |
            Some((_, (13, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action19(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Bit(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        return Ok(__result);
    }

    // State 40
    //   FactFunc = "exists" (*) Variable "->" FactFunc [","]
    //   FactFunc = "exists" (*) Variable "->" FactFunc ["."]
    //   FactFunc = "exists" (*) Variable "->" FactFunc [";"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["->"]
    //
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Shift(S52)
    //
    //   Variable -> S59
    pub fn __state40<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (11, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state52(input, __lookbehind, __tokens, __sym1));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        while __sym0.is_some() {
            let (__lookbehind, __lookahead, __nt) = __result;
            match __nt {
                __Nonterminal::Variable(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state59(input, __lookbehind, __tokens, __lookahead, __sym0, __sym1));
                }
                _ => {
                    return Ok((__lookbehind, __lookahead, __nt));
                }
            }
        }
        return Ok(__result);
    }

    // State 41
    //   FactFunc = "forall" (*) Variable "->" FactFunc [","]
    //   FactFunc = "forall" (*) Variable "->" FactFunc ["."]
    //   FactFunc = "forall" (*) Variable "->" FactFunc [";"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["->"]
    //
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Shift(S52)
    //
    //   Variable -> S60
    pub fn __state41<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (11, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym1 = &mut Some((__tok0));
                __result = try!(__state52(input, __lookbehind, __tokens, __sym1));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        while __sym0.is_some() {
            let (__lookbehind, __lookahead, __nt) = __result;
            match __nt {
                __Nonterminal::Variable(__nt) => {
                    let __sym1 = &mut Some(__nt);
                    __result = try!(__state60(input, __lookbehind, __tokens, __lookahead, __sym0, __sym1));
                }
                _ => {
                    return Ok((__lookbehind, __lookahead, __nt));
                }
            }
        }
        return Ok(__result);
    }

    // State 42
    //   Bit+ = Bit+ Bit (*) ["=>"]
    //   Bit+ = Bit+ Bit (*) ["["]
    //   Bit+ = Bit+ Bit (*) [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = Bit+ Bit (*) [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = Bit+ Bit (*) [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = Bit+ Bit (*) [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "=>" -> Reduce(Bit+ = Bit+, Bit => ActionFn(23);)
    //   "[" -> Reduce(Bit+ = Bit+, Bit => ActionFn(23);)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Reduce(Bit+ = Bit+, Bit => ActionFn(23);)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Reduce(Bit+ = Bit+, Bit => ActionFn(23);)
    //   r#"[A-Za-z0-9_]+:"# -> Reduce(Bit+ = Bit+, Bit => ActionFn(23);)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Reduce(Bit+ = Bit+, Bit => ActionFn(23);)
    //
    pub fn __state42<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<::std::vec::Vec<Bit>>,
        __sym1: &mut Option<Bit>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (5, _), _)) |
            Some((_, (6, _), _)) |
            Some((_, (10, _), _)) |
            Some((_, (11, _), _)) |
            Some((_, (12, _), _)) |
            Some((_, (13, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __sym1 = __sym1.take().unwrap();
                let __nt = super::__action23(input, __sym0, __sym1, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Bit_2b(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 43
    //   Bit = Variable (*) ["=>"]
    //   Bit = Variable (*) ["["]
    //   Bit = Variable (*) [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = Variable (*) [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = Variable (*) [r#"[A-Za-z0-9_]+:"#]
    //   Bit = Variable (*) [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "=>" -> Reduce(Bit = Variable => ActionFn(19);)
    //   "[" -> Reduce(Bit = Variable => ActionFn(19);)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Reduce(Bit = Variable => ActionFn(19);)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Reduce(Bit = Variable => ActionFn(19);)
    //   r#"[A-Za-z0-9_]+:"# -> Reduce(Bit = Variable => ActionFn(19);)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Reduce(Bit = Variable => ActionFn(19);)
    //
    pub fn __state43<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Variable>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (5, _), _)) |
            Some((_, (6, _), _)) |
            Some((_, (10, _), _)) |
            Some((_, (11, _), _)) |
            Some((_, (12, _), _)) |
            Some((_, (13, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action19(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Bit(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 44
    //   Variable = r#"[A-Z][_A-Za-z0-9]*"# (*) ["=>"]
    //   Variable = r#"[A-Z][_A-Za-z0-9]*"# (*) ["["]
    //   Variable = r#"[A-Z][_A-Za-z0-9]*"# (*) [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Variable = r#"[A-Z][_A-Za-z0-9]*"# (*) [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Variable = r#"[A-Z][_A-Za-z0-9]*"# (*) [r#"[A-Za-z0-9_]+:"#]
    //   Variable = r#"[A-Z][_A-Za-z0-9]*"# (*) [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "=>" -> Reduce(Variable = r#"[A-Z][_A-Za-z0-9]*"# => ActionFn(21);)
    //   "[" -> Reduce(Variable = r#"[A-Z][_A-Za-z0-9]*"# => ActionFn(21);)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Reduce(Variable = r#"[A-Z][_A-Za-z0-9]*"# => ActionFn(21);)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Reduce(Variable = r#"[A-Z][_A-Za-z0-9]*"# => ActionFn(21);)
    //   r#"[A-Za-z0-9_]+:"# -> Reduce(Variable = r#"[A-Z][_A-Za-z0-9]*"# => ActionFn(21);)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Reduce(Variable = r#"[A-Z][_A-Za-z0-9]*"# => ActionFn(21);)
    //
    pub fn __state44<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (5, _), _)) |
            Some((_, (6, _), _)) |
            Some((_, (10, _), _)) |
            Some((_, (11, _), _)) |
            Some((_, (12, _), _)) |
            Some((_, (13, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action21(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Variable(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 45
    //   Bit = "[" Fact "]" (*) ["."]
    //   Bit = "[" Fact "]" (*) [":-"]
    //   Bit = "[" Fact "]" (*) ["["]
    //   Bit = "[" Fact "]" (*) [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = "[" Fact "]" (*) [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = "[" Fact "]" (*) [r#"[A-Za-z0-9_]+:"#]
    //   Bit = "[" Fact "]" (*) [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "." -> Reduce(Bit = "[", Fact, "]" => ActionFn(20);)
    //   ":-" -> Reduce(Bit = "[", Fact, "]" => ActionFn(20);)
    //   "[" -> Reduce(Bit = "[", Fact, "]" => ActionFn(20);)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Reduce(Bit = "[", Fact, "]" => ActionFn(20);)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Reduce(Bit = "[", Fact, "]" => ActionFn(20);)
    //   r#"[A-Za-z0-9_]+:"# -> Reduce(Bit = "[", Fact, "]" => ActionFn(20);)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Reduce(Bit = "[", Fact, "]" => ActionFn(20);)
    //
    pub fn __state45<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<&'input str>,
        __sym1: &mut Option<Fact>,
        __sym2: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (2, _), _)) |
            Some((_, (3, _), _)) |
            Some((_, (6, _), _)) |
            Some((_, (10, _), _)) |
            Some((_, (11, _), _)) |
            Some((_, (12, _), _)) |
            Some((_, (13, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __sym1 = __sym1.take().unwrap();
                let __sym2 = __sym2.take().unwrap();
                let __nt = super::__action20(input, __sym0, __sym1, __sym2, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Bit(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 46
    //   Application = (*) Bit+ ["=>"]
    //   Bit = (*) Variable ["=>"]
    //   Bit = (*) Variable ["["]
    //   Bit = (*) Variable [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) Variable [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) Variable [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) Variable [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" ["=>"]
    //   Bit = (*) "[" Fact "]" ["["]
    //   Bit = (*) "[" Fact "]" [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) "[" Fact "]" [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["=>"]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["["]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["=>"]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["["]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["=>"]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["["]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit ["=>"]
    //   Bit+ = (*) Bit ["["]
    //   Bit+ = (*) Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit ["=>"]
    //   Bit+ = (*) Bit+ Bit ["["]
    //   Bit+ = (*) Bit+ Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit+ Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   FactAnd = FactAnd ";" (*) FactOr [";"]
    //   FactAnd = FactAnd ";" (*) FactOr ["]"]
    //   FactApply = (*) Application ["=>"]
    //   FactFunc = (*) FactApply "=>" FactFunc [","]
    //   FactFunc = (*) FactApply "=>" FactFunc [";"]
    //   FactFunc = (*) FactApply "=>" FactFunc ["]"]
    //   FactFunc = (*) Variable "->" FactFunc [","]
    //   FactFunc = (*) Variable "->" FactFunc [";"]
    //   FactFunc = (*) Variable "->" FactFunc ["]"]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [","]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [";"]
    //   FactFunc = (*) "exists" Variable "->" FactFunc ["]"]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [","]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [";"]
    //   FactFunc = (*) "forall" Variable "->" FactFunc ["]"]
    //   FactOr = (*) FactFunc [","]
    //   FactOr = (*) FactFunc [";"]
    //   FactOr = (*) FactFunc ["]"]
    //   FactOr = (*) FactOr "," FactFunc [","]
    //   FactOr = (*) FactOr "," FactFunc [";"]
    //   FactOr = (*) FactOr "," FactFunc ["]"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["->"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["=>"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["["]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "[" -> Shift(S27)
    //   "exists" -> Shift(S28)
    //   "forall" -> Shift(S29)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Shift(S30)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Shift(S31)
    //   r#"[A-Za-z0-9_]+:"# -> Shift(S32)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Shift(S33)
    //
    //   Application -> S18
    //   Bit -> S19
    //   Bit+ -> S20
    //   FactApply -> S23
    //   FactFunc -> S24
    //   FactOr -> S61
    //   Variable -> S26
    pub fn __state46<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<Fact>,
        __sym1: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (6, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state27(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (8, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state28(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (9, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state29(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (10, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state30(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (11, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state31(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (12, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state32(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (13, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state33(input, __lookbehind, __tokens, __sym2));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        while __sym1.is_some() {
            let (__lookbehind, __lookahead, __nt) = __result;
            match __nt {
                __Nonterminal::Application(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state18(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::Bit(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state19(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::Bit_2b(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state20(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::FactApply(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state23(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::FactFunc(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state24(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::FactOr(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state61(input, __lookbehind, __tokens, __lookahead, __sym0, __sym1, __sym2));
                }
                __Nonterminal::Variable(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state26(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                _ => {
                    return Ok((__lookbehind, __lookahead, __nt));
                }
            }
        }
        return Ok(__result);
    }

    // State 47
    //   Application = (*) Bit+ ["=>"]
    //   Bit = (*) Variable ["=>"]
    //   Bit = (*) Variable ["["]
    //   Bit = (*) Variable [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) Variable [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) Variable [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) Variable [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" ["=>"]
    //   Bit = (*) "[" Fact "]" ["["]
    //   Bit = (*) "[" Fact "]" [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) "[" Fact "]" [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["=>"]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["["]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["=>"]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["["]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["=>"]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["["]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit ["=>"]
    //   Bit+ = (*) Bit ["["]
    //   Bit+ = (*) Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit ["=>"]
    //   Bit+ = (*) Bit+ Bit ["["]
    //   Bit+ = (*) Bit+ Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit+ Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   FactApply = (*) Application ["=>"]
    //   FactFunc = (*) FactApply "=>" FactFunc [","]
    //   FactFunc = (*) FactApply "=>" FactFunc [";"]
    //   FactFunc = (*) FactApply "=>" FactFunc ["]"]
    //   FactFunc = FactApply "=>" (*) FactFunc [","]
    //   FactFunc = FactApply "=>" (*) FactFunc [";"]
    //   FactFunc = FactApply "=>" (*) FactFunc ["]"]
    //   FactFunc = (*) Variable "->" FactFunc [","]
    //   FactFunc = (*) Variable "->" FactFunc [";"]
    //   FactFunc = (*) Variable "->" FactFunc ["]"]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [","]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [";"]
    //   FactFunc = (*) "exists" Variable "->" FactFunc ["]"]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [","]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [";"]
    //   FactFunc = (*) "forall" Variable "->" FactFunc ["]"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["->"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["=>"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["["]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "[" -> Shift(S27)
    //   "exists" -> Shift(S28)
    //   "forall" -> Shift(S29)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Shift(S30)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Shift(S31)
    //   r#"[A-Za-z0-9_]+:"# -> Shift(S32)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Shift(S33)
    //
    //   Application -> S18
    //   Bit -> S19
    //   Bit+ -> S20
    //   FactApply -> S23
    //   FactFunc -> S62
    //   Variable -> S26
    pub fn __state47<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<Fact>,
        __sym1: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (6, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state27(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (8, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state28(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (9, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state29(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (10, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state30(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (11, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state31(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (12, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state32(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (13, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state33(input, __lookbehind, __tokens, __sym2));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        while __sym1.is_some() {
            let (__lookbehind, __lookahead, __nt) = __result;
            match __nt {
                __Nonterminal::Application(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state18(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::Bit(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state19(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::Bit_2b(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state20(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::FactApply(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state23(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::FactFunc(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state62(input, __lookbehind, __tokens, __lookahead, __sym0, __sym1, __sym2));
                }
                __Nonterminal::Variable(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state26(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                _ => {
                    return Ok((__lookbehind, __lookahead, __nt));
                }
            }
        }
        return Ok(__result);
    }

    // State 48
    //   Application = (*) Bit+ ["=>"]
    //   Bit = (*) Variable ["=>"]
    //   Bit = (*) Variable ["["]
    //   Bit = (*) Variable [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) Variable [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) Variable [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) Variable [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" ["=>"]
    //   Bit = (*) "[" Fact "]" ["["]
    //   Bit = (*) "[" Fact "]" [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) "[" Fact "]" [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["=>"]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["["]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["=>"]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["["]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["=>"]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["["]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit ["=>"]
    //   Bit+ = (*) Bit ["["]
    //   Bit+ = (*) Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit ["=>"]
    //   Bit+ = (*) Bit+ Bit ["["]
    //   Bit+ = (*) Bit+ Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit+ Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   FactApply = (*) Application ["=>"]
    //   FactFunc = (*) FactApply "=>" FactFunc [","]
    //   FactFunc = (*) FactApply "=>" FactFunc [";"]
    //   FactFunc = (*) FactApply "=>" FactFunc ["]"]
    //   FactFunc = (*) Variable "->" FactFunc [","]
    //   FactFunc = (*) Variable "->" FactFunc [";"]
    //   FactFunc = (*) Variable "->" FactFunc ["]"]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [","]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [";"]
    //   FactFunc = (*) "exists" Variable "->" FactFunc ["]"]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [","]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [";"]
    //   FactFunc = (*) "forall" Variable "->" FactFunc ["]"]
    //   FactOr = FactOr "," (*) FactFunc [","]
    //   FactOr = FactOr "," (*) FactFunc [";"]
    //   FactOr = FactOr "," (*) FactFunc ["]"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["->"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["=>"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["["]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "[" -> Shift(S27)
    //   "exists" -> Shift(S28)
    //   "forall" -> Shift(S29)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Shift(S30)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Shift(S31)
    //   r#"[A-Za-z0-9_]+:"# -> Shift(S32)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Shift(S33)
    //
    //   Application -> S18
    //   Bit -> S19
    //   Bit+ -> S20
    //   FactApply -> S23
    //   FactFunc -> S63
    //   Variable -> S26
    pub fn __state48<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<Fact>,
        __sym1: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (6, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state27(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (8, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state28(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (9, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state29(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (10, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state30(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (11, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state31(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (12, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state32(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (13, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state33(input, __lookbehind, __tokens, __sym2));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        while __sym1.is_some() {
            let (__lookbehind, __lookahead, __nt) = __result;
            match __nt {
                __Nonterminal::Application(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state18(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::Bit(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state19(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::Bit_2b(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state20(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::FactApply(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state23(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::FactFunc(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state63(input, __lookbehind, __tokens, __lookahead, __sym0, __sym1, __sym2));
                }
                __Nonterminal::Variable(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state26(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                _ => {
                    return Ok((__lookbehind, __lookahead, __nt));
                }
            }
        }
        return Ok(__result);
    }

    // State 49
    //   Application = (*) Bit+ ["=>"]
    //   Bit = (*) Variable ["=>"]
    //   Bit = (*) Variable ["["]
    //   Bit = (*) Variable [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) Variable [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) Variable [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) Variable [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" ["=>"]
    //   Bit = (*) "[" Fact "]" ["["]
    //   Bit = (*) "[" Fact "]" [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) "[" Fact "]" [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["=>"]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["["]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["=>"]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["["]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["=>"]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["["]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit ["=>"]
    //   Bit+ = (*) Bit ["["]
    //   Bit+ = (*) Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit ["=>"]
    //   Bit+ = (*) Bit+ Bit ["["]
    //   Bit+ = (*) Bit+ Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit+ Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   FactApply = (*) Application ["=>"]
    //   FactFunc = (*) FactApply "=>" FactFunc [","]
    //   FactFunc = (*) FactApply "=>" FactFunc [";"]
    //   FactFunc = (*) FactApply "=>" FactFunc ["]"]
    //   FactFunc = (*) Variable "->" FactFunc [","]
    //   FactFunc = (*) Variable "->" FactFunc [";"]
    //   FactFunc = (*) Variable "->" FactFunc ["]"]
    //   FactFunc = Variable "->" (*) FactFunc [","]
    //   FactFunc = Variable "->" (*) FactFunc [";"]
    //   FactFunc = Variable "->" (*) FactFunc ["]"]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [","]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [";"]
    //   FactFunc = (*) "exists" Variable "->" FactFunc ["]"]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [","]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [";"]
    //   FactFunc = (*) "forall" Variable "->" FactFunc ["]"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["->"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["=>"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["["]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "[" -> Shift(S27)
    //   "exists" -> Shift(S28)
    //   "forall" -> Shift(S29)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Shift(S30)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Shift(S31)
    //   r#"[A-Za-z0-9_]+:"# -> Shift(S32)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Shift(S33)
    //
    //   Application -> S18
    //   Bit -> S19
    //   Bit+ -> S20
    //   FactApply -> S23
    //   FactFunc -> S64
    //   Variable -> S26
    pub fn __state49<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<Variable>,
        __sym1: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (6, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state27(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (8, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state28(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (9, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state29(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (10, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state30(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (11, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state31(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (12, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state32(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (13, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state33(input, __lookbehind, __tokens, __sym2));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        while __sym1.is_some() {
            let (__lookbehind, __lookahead, __nt) = __result;
            match __nt {
                __Nonterminal::Application(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state18(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::Bit(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state19(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::Bit_2b(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state20(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::FactApply(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state23(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::FactFunc(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state64(input, __lookbehind, __tokens, __lookahead, __sym0, __sym1, __sym2));
                }
                __Nonterminal::Variable(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state26(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                _ => {
                    return Ok((__lookbehind, __lookahead, __nt));
                }
            }
        }
        return Ok(__result);
    }

    // State 50
    //   Bit = "[" Fact (*) "]" ["=>"]
    //   Bit = "[" Fact (*) "]" ["["]
    //   Bit = "[" Fact (*) "]" [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = "[" Fact (*) "]" [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = "[" Fact (*) "]" [r#"[A-Za-z0-9_]+:"#]
    //   Bit = "[" Fact (*) "]" [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "]" -> Shift(S65)
    //
    pub fn __state50<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<&'input str>,
        __sym1: &mut Option<Fact>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (7, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state65(input, __lookbehind, __tokens, __sym0, __sym1, __sym2));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        return Ok(__result);
    }

    // State 51
    //   FactFunc = "exists" Variable (*) "->" FactFunc [","]
    //   FactFunc = "exists" Variable (*) "->" FactFunc [";"]
    //   FactFunc = "exists" Variable (*) "->" FactFunc ["]"]
    //
    //   "->" -> Shift(S66)
    //
    pub fn __state51<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<&'input str>,
        __sym1: &mut Option<Variable>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (1, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state66(input, __lookbehind, __tokens, __sym0, __sym1, __sym2));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        return Ok(__result);
    }

    // State 52
    //   Variable = r#"[A-Z][_A-Za-z0-9]*"# (*) ["->"]
    //
    //   "->" -> Reduce(Variable = r#"[A-Z][_A-Za-z0-9]*"# => ActionFn(21);)
    //
    pub fn __state52<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (1, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __nt = super::__action21(input, __sym0, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Variable(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 53
    //   FactFunc = "forall" Variable (*) "->" FactFunc [","]
    //   FactFunc = "forall" Variable (*) "->" FactFunc [";"]
    //   FactFunc = "forall" Variable (*) "->" FactFunc ["]"]
    //
    //   "->" -> Shift(S67)
    //
    pub fn __state53<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<&'input str>,
        __sym1: &mut Option<Variable>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (1, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state67(input, __lookbehind, __tokens, __sym0, __sym1, __sym2));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        return Ok(__result);
    }

    // State 54
    //   Rule = Application ":-" Fact "." (*) [EOF]
    //   Rule = Application ":-" Fact "." (*) ["["]
    //   Rule = Application ":-" Fact "." (*) [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Rule = Application ":-" Fact "." (*) [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Rule = Application ":-" Fact "." (*) [r#"[A-Za-z0-9_]+:"#]
    //   Rule = Application ":-" Fact "." (*) [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   EOF -> Reduce(Rule = Application, ":-", Fact, "." => ActionFn(4);)
    //   "[" -> Reduce(Rule = Application, ":-", Fact, "." => ActionFn(4);)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Reduce(Rule = Application, ":-", Fact, "." => ActionFn(4);)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Reduce(Rule = Application, ":-", Fact, "." => ActionFn(4);)
    //   r#"[A-Za-z0-9_]+:"# -> Reduce(Rule = Application, ":-", Fact, "." => ActionFn(4);)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Reduce(Rule = Application, ":-", Fact, "." => ActionFn(4);)
    //
    pub fn __state54<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<Application>,
        __sym1: &mut Option<&'input str>,
        __sym2: &mut Option<Fact>,
        __sym3: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            None |
            Some((_, (6, _), _)) |
            Some((_, (10, _), _)) |
            Some((_, (11, _), _)) |
            Some((_, (12, _), _)) |
            Some((_, (13, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __sym1 = __sym1.take().unwrap();
                let __sym2 = __sym2.take().unwrap();
                let __sym3 = __sym3.take().unwrap();
                let __nt = super::__action4(input, __sym0, __sym1, __sym2, __sym3, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Rule(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 55
    //   Application = (*) Bit+ ["=>"]
    //   Bit = (*) Variable ["=>"]
    //   Bit = (*) Variable ["["]
    //   Bit = (*) Variable [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) Variable [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) Variable [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) Variable [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" ["=>"]
    //   Bit = (*) "[" Fact "]" ["["]
    //   Bit = (*) "[" Fact "]" [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) "[" Fact "]" [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["=>"]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["["]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["=>"]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["["]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["=>"]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["["]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit ["=>"]
    //   Bit+ = (*) Bit ["["]
    //   Bit+ = (*) Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit ["=>"]
    //   Bit+ = (*) Bit+ Bit ["["]
    //   Bit+ = (*) Bit+ Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit+ Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   FactAnd = FactAnd ";" (*) FactOr ["."]
    //   FactAnd = FactAnd ";" (*) FactOr [";"]
    //   FactApply = (*) Application ["=>"]
    //   FactFunc = (*) FactApply "=>" FactFunc [","]
    //   FactFunc = (*) FactApply "=>" FactFunc ["."]
    //   FactFunc = (*) FactApply "=>" FactFunc [";"]
    //   FactFunc = (*) Variable "->" FactFunc [","]
    //   FactFunc = (*) Variable "->" FactFunc ["."]
    //   FactFunc = (*) Variable "->" FactFunc [";"]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [","]
    //   FactFunc = (*) "exists" Variable "->" FactFunc ["."]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [";"]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [","]
    //   FactFunc = (*) "forall" Variable "->" FactFunc ["."]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [";"]
    //   FactOr = (*) FactFunc [","]
    //   FactOr = (*) FactFunc ["."]
    //   FactOr = (*) FactFunc [";"]
    //   FactOr = (*) FactOr "," FactFunc [","]
    //   FactOr = (*) FactOr "," FactFunc ["."]
    //   FactOr = (*) FactOr "," FactFunc [";"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["->"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["=>"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["["]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "[" -> Shift(S27)
    //   "exists" -> Shift(S40)
    //   "forall" -> Shift(S41)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Shift(S30)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Shift(S31)
    //   r#"[A-Za-z0-9_]+:"# -> Shift(S32)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Shift(S33)
    //
    //   Application -> S18
    //   Bit -> S19
    //   Bit+ -> S20
    //   FactApply -> S36
    //   FactFunc -> S37
    //   FactOr -> S68
    //   Variable -> S39
    pub fn __state55<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<Fact>,
        __sym1: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (6, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state27(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (8, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state40(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (9, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state41(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (10, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state30(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (11, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state31(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (12, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state32(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (13, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state33(input, __lookbehind, __tokens, __sym2));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        while __sym1.is_some() {
            let (__lookbehind, __lookahead, __nt) = __result;
            match __nt {
                __Nonterminal::Application(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state18(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::Bit(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state19(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::Bit_2b(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state20(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::FactApply(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state36(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::FactFunc(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state37(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::FactOr(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state68(input, __lookbehind, __tokens, __lookahead, __sym0, __sym1, __sym2));
                }
                __Nonterminal::Variable(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state39(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                _ => {
                    return Ok((__lookbehind, __lookahead, __nt));
                }
            }
        }
        return Ok(__result);
    }

    // State 56
    //   Application = (*) Bit+ ["=>"]
    //   Bit = (*) Variable ["=>"]
    //   Bit = (*) Variable ["["]
    //   Bit = (*) Variable [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) Variable [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) Variable [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) Variable [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" ["=>"]
    //   Bit = (*) "[" Fact "]" ["["]
    //   Bit = (*) "[" Fact "]" [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) "[" Fact "]" [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["=>"]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["["]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["=>"]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["["]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["=>"]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["["]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit ["=>"]
    //   Bit+ = (*) Bit ["["]
    //   Bit+ = (*) Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit ["=>"]
    //   Bit+ = (*) Bit+ Bit ["["]
    //   Bit+ = (*) Bit+ Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit+ Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   FactApply = (*) Application ["=>"]
    //   FactFunc = (*) FactApply "=>" FactFunc [","]
    //   FactFunc = (*) FactApply "=>" FactFunc ["."]
    //   FactFunc = (*) FactApply "=>" FactFunc [";"]
    //   FactFunc = FactApply "=>" (*) FactFunc [","]
    //   FactFunc = FactApply "=>" (*) FactFunc ["."]
    //   FactFunc = FactApply "=>" (*) FactFunc [";"]
    //   FactFunc = (*) Variable "->" FactFunc [","]
    //   FactFunc = (*) Variable "->" FactFunc ["."]
    //   FactFunc = (*) Variable "->" FactFunc [";"]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [","]
    //   FactFunc = (*) "exists" Variable "->" FactFunc ["."]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [";"]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [","]
    //   FactFunc = (*) "forall" Variable "->" FactFunc ["."]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [";"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["->"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["=>"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["["]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "[" -> Shift(S27)
    //   "exists" -> Shift(S40)
    //   "forall" -> Shift(S41)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Shift(S30)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Shift(S31)
    //   r#"[A-Za-z0-9_]+:"# -> Shift(S32)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Shift(S33)
    //
    //   Application -> S18
    //   Bit -> S19
    //   Bit+ -> S20
    //   FactApply -> S36
    //   FactFunc -> S69
    //   Variable -> S39
    pub fn __state56<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<Fact>,
        __sym1: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (6, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state27(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (8, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state40(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (9, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state41(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (10, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state30(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (11, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state31(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (12, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state32(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (13, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state33(input, __lookbehind, __tokens, __sym2));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        while __sym1.is_some() {
            let (__lookbehind, __lookahead, __nt) = __result;
            match __nt {
                __Nonterminal::Application(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state18(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::Bit(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state19(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::Bit_2b(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state20(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::FactApply(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state36(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::FactFunc(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state69(input, __lookbehind, __tokens, __lookahead, __sym0, __sym1, __sym2));
                }
                __Nonterminal::Variable(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state39(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                _ => {
                    return Ok((__lookbehind, __lookahead, __nt));
                }
            }
        }
        return Ok(__result);
    }

    // State 57
    //   Application = (*) Bit+ ["=>"]
    //   Bit = (*) Variable ["=>"]
    //   Bit = (*) Variable ["["]
    //   Bit = (*) Variable [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) Variable [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) Variable [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) Variable [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" ["=>"]
    //   Bit = (*) "[" Fact "]" ["["]
    //   Bit = (*) "[" Fact "]" [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) "[" Fact "]" [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["=>"]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["["]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["=>"]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["["]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["=>"]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["["]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit ["=>"]
    //   Bit+ = (*) Bit ["["]
    //   Bit+ = (*) Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit ["=>"]
    //   Bit+ = (*) Bit+ Bit ["["]
    //   Bit+ = (*) Bit+ Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit+ Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   FactApply = (*) Application ["=>"]
    //   FactFunc = (*) FactApply "=>" FactFunc [","]
    //   FactFunc = (*) FactApply "=>" FactFunc ["."]
    //   FactFunc = (*) FactApply "=>" FactFunc [";"]
    //   FactFunc = (*) Variable "->" FactFunc [","]
    //   FactFunc = (*) Variable "->" FactFunc ["."]
    //   FactFunc = (*) Variable "->" FactFunc [";"]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [","]
    //   FactFunc = (*) "exists" Variable "->" FactFunc ["."]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [";"]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [","]
    //   FactFunc = (*) "forall" Variable "->" FactFunc ["."]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [";"]
    //   FactOr = FactOr "," (*) FactFunc [","]
    //   FactOr = FactOr "," (*) FactFunc ["."]
    //   FactOr = FactOr "," (*) FactFunc [";"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["->"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["=>"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["["]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "[" -> Shift(S27)
    //   "exists" -> Shift(S40)
    //   "forall" -> Shift(S41)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Shift(S30)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Shift(S31)
    //   r#"[A-Za-z0-9_]+:"# -> Shift(S32)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Shift(S33)
    //
    //   Application -> S18
    //   Bit -> S19
    //   Bit+ -> S20
    //   FactApply -> S36
    //   FactFunc -> S70
    //   Variable -> S39
    pub fn __state57<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<Fact>,
        __sym1: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (6, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state27(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (8, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state40(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (9, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state41(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (10, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state30(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (11, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state31(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (12, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state32(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (13, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state33(input, __lookbehind, __tokens, __sym2));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        while __sym1.is_some() {
            let (__lookbehind, __lookahead, __nt) = __result;
            match __nt {
                __Nonterminal::Application(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state18(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::Bit(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state19(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::Bit_2b(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state20(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::FactApply(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state36(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::FactFunc(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state70(input, __lookbehind, __tokens, __lookahead, __sym0, __sym1, __sym2));
                }
                __Nonterminal::Variable(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state39(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                _ => {
                    return Ok((__lookbehind, __lookahead, __nt));
                }
            }
        }
        return Ok(__result);
    }

    // State 58
    //   Application = (*) Bit+ ["=>"]
    //   Bit = (*) Variable ["=>"]
    //   Bit = (*) Variable ["["]
    //   Bit = (*) Variable [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) Variable [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) Variable [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) Variable [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" ["=>"]
    //   Bit = (*) "[" Fact "]" ["["]
    //   Bit = (*) "[" Fact "]" [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) "[" Fact "]" [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["=>"]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["["]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["=>"]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["["]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["=>"]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["["]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit ["=>"]
    //   Bit+ = (*) Bit ["["]
    //   Bit+ = (*) Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit ["=>"]
    //   Bit+ = (*) Bit+ Bit ["["]
    //   Bit+ = (*) Bit+ Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit+ Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   FactApply = (*) Application ["=>"]
    //   FactFunc = (*) FactApply "=>" FactFunc [","]
    //   FactFunc = (*) FactApply "=>" FactFunc ["."]
    //   FactFunc = (*) FactApply "=>" FactFunc [";"]
    //   FactFunc = (*) Variable "->" FactFunc [","]
    //   FactFunc = (*) Variable "->" FactFunc ["."]
    //   FactFunc = (*) Variable "->" FactFunc [";"]
    //   FactFunc = Variable "->" (*) FactFunc [","]
    //   FactFunc = Variable "->" (*) FactFunc ["."]
    //   FactFunc = Variable "->" (*) FactFunc [";"]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [","]
    //   FactFunc = (*) "exists" Variable "->" FactFunc ["."]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [";"]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [","]
    //   FactFunc = (*) "forall" Variable "->" FactFunc ["."]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [";"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["->"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["=>"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["["]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "[" -> Shift(S27)
    //   "exists" -> Shift(S40)
    //   "forall" -> Shift(S41)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Shift(S30)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Shift(S31)
    //   r#"[A-Za-z0-9_]+:"# -> Shift(S32)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Shift(S33)
    //
    //   Application -> S18
    //   Bit -> S19
    //   Bit+ -> S20
    //   FactApply -> S36
    //   FactFunc -> S71
    //   Variable -> S39
    pub fn __state58<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<Variable>,
        __sym1: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (6, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state27(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (8, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state40(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (9, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state41(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (10, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state30(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (11, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state31(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (12, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state32(input, __lookbehind, __tokens, __sym2));
            }
            Some((_, (13, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state33(input, __lookbehind, __tokens, __sym2));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        while __sym1.is_some() {
            let (__lookbehind, __lookahead, __nt) = __result;
            match __nt {
                __Nonterminal::Application(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state18(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::Bit(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state19(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::Bit_2b(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state20(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::FactApply(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state36(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                __Nonterminal::FactFunc(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state71(input, __lookbehind, __tokens, __lookahead, __sym0, __sym1, __sym2));
                }
                __Nonterminal::Variable(__nt) => {
                    let __sym2 = &mut Some(__nt);
                    __result = try!(__state39(input, __lookbehind, __tokens, __lookahead, __sym2));
                }
                _ => {
                    return Ok((__lookbehind, __lookahead, __nt));
                }
            }
        }
        return Ok(__result);
    }

    // State 59
    //   FactFunc = "exists" Variable (*) "->" FactFunc [","]
    //   FactFunc = "exists" Variable (*) "->" FactFunc ["."]
    //   FactFunc = "exists" Variable (*) "->" FactFunc [";"]
    //
    //   "->" -> Shift(S72)
    //
    pub fn __state59<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<&'input str>,
        __sym1: &mut Option<Variable>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (1, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state72(input, __lookbehind, __tokens, __sym0, __sym1, __sym2));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        return Ok(__result);
    }

    // State 60
    //   FactFunc = "forall" Variable (*) "->" FactFunc [","]
    //   FactFunc = "forall" Variable (*) "->" FactFunc ["."]
    //   FactFunc = "forall" Variable (*) "->" FactFunc [";"]
    //
    //   "->" -> Shift(S73)
    //
    pub fn __state60<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<&'input str>,
        __sym1: &mut Option<Variable>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (1, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym2 = &mut Some((__tok0));
                __result = try!(__state73(input, __lookbehind, __tokens, __sym0, __sym1, __sym2));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        return Ok(__result);
    }

    // State 61
    //   FactAnd = FactAnd ";" FactOr (*) [";"]
    //   FactAnd = FactAnd ";" FactOr (*) ["]"]
    //   FactOr = FactOr (*) "," FactFunc [","]
    //   FactOr = FactOr (*) "," FactFunc [";"]
    //   FactOr = FactOr (*) "," FactFunc ["]"]
    //
    //   "," -> Shift(S48)
    //   ";" -> Reduce(FactAnd = FactAnd, ";", FactOr => ActionFn(7);)
    //   "]" -> Reduce(FactAnd = FactAnd, ";", FactOr => ActionFn(7);)
    //
    pub fn __state61<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Fact>,
        __sym1: &mut Option<&'input str>,
        __sym2: &mut Option<Fact>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (0, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state48(input, __lookbehind, __tokens, __sym2, __sym3));
            }
            Some((_, (4, _), _)) |
            Some((_, (7, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __sym1 = __sym1.take().unwrap();
                let __sym2 = __sym2.take().unwrap();
                let __nt = super::__action7(input, __sym0, __sym1, __sym2, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::FactAnd(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        return Ok(__result);
    }

    // State 62
    //   FactFunc = FactApply "=>" FactFunc (*) [","]
    //   FactFunc = FactApply "=>" FactFunc (*) [";"]
    //   FactFunc = FactApply "=>" FactFunc (*) ["]"]
    //
    //   "," -> Reduce(FactFunc = FactApply, "=>", FactFunc => ActionFn(10);)
    //   ";" -> Reduce(FactFunc = FactApply, "=>", FactFunc => ActionFn(10);)
    //   "]" -> Reduce(FactFunc = FactApply, "=>", FactFunc => ActionFn(10);)
    //
    pub fn __state62<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Fact>,
        __sym1: &mut Option<&'input str>,
        __sym2: &mut Option<Fact>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (0, _), _)) |
            Some((_, (4, _), _)) |
            Some((_, (7, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __sym1 = __sym1.take().unwrap();
                let __sym2 = __sym2.take().unwrap();
                let __nt = super::__action10(input, __sym0, __sym1, __sym2, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::FactFunc(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 63
    //   FactOr = FactOr "," FactFunc (*) [","]
    //   FactOr = FactOr "," FactFunc (*) [";"]
    //   FactOr = FactOr "," FactFunc (*) ["]"]
    //
    //   "," -> Reduce(FactOr = FactOr, ",", FactFunc => ActionFn(9);)
    //   ";" -> Reduce(FactOr = FactOr, ",", FactFunc => ActionFn(9);)
    //   "]" -> Reduce(FactOr = FactOr, ",", FactFunc => ActionFn(9);)
    //
    pub fn __state63<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Fact>,
        __sym1: &mut Option<&'input str>,
        __sym2: &mut Option<Fact>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (0, _), _)) |
            Some((_, (4, _), _)) |
            Some((_, (7, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __sym1 = __sym1.take().unwrap();
                let __sym2 = __sym2.take().unwrap();
                let __nt = super::__action9(input, __sym0, __sym1, __sym2, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::FactOr(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 64
    //   FactFunc = Variable "->" FactFunc (*) [","]
    //   FactFunc = Variable "->" FactFunc (*) [";"]
    //   FactFunc = Variable "->" FactFunc (*) ["]"]
    //
    //   "," -> Reduce(FactFunc = Variable, "->", FactFunc => ActionFn(13);)
    //   ";" -> Reduce(FactFunc = Variable, "->", FactFunc => ActionFn(13);)
    //   "]" -> Reduce(FactFunc = Variable, "->", FactFunc => ActionFn(13);)
    //
    pub fn __state64<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Variable>,
        __sym1: &mut Option<&'input str>,
        __sym2: &mut Option<Fact>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (0, _), _)) |
            Some((_, (4, _), _)) |
            Some((_, (7, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __sym1 = __sym1.take().unwrap();
                let __sym2 = __sym2.take().unwrap();
                let __nt = super::__action13(input, __sym0, __sym1, __sym2, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::FactFunc(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 65
    //   Bit = "[" Fact "]" (*) ["=>"]
    //   Bit = "[" Fact "]" (*) ["["]
    //   Bit = "[" Fact "]" (*) [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = "[" Fact "]" (*) [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = "[" Fact "]" (*) [r#"[A-Za-z0-9_]+:"#]
    //   Bit = "[" Fact "]" (*) [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "=>" -> Reduce(Bit = "[", Fact, "]" => ActionFn(20);)
    //   "[" -> Reduce(Bit = "[", Fact, "]" => ActionFn(20);)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Reduce(Bit = "[", Fact, "]" => ActionFn(20);)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Reduce(Bit = "[", Fact, "]" => ActionFn(20);)
    //   r#"[A-Za-z0-9_]+:"# -> Reduce(Bit = "[", Fact, "]" => ActionFn(20);)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Reduce(Bit = "[", Fact, "]" => ActionFn(20);)
    //
    pub fn __state65<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<&'input str>,
        __sym1: &mut Option<Fact>,
        __sym2: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (5, _), _)) |
            Some((_, (6, _), _)) |
            Some((_, (10, _), _)) |
            Some((_, (11, _), _)) |
            Some((_, (12, _), _)) |
            Some((_, (13, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __sym1 = __sym1.take().unwrap();
                let __sym2 = __sym2.take().unwrap();
                let __nt = super::__action20(input, __sym0, __sym1, __sym2, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::Bit(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 66
    //   Application = (*) Bit+ ["=>"]
    //   Bit = (*) Variable ["=>"]
    //   Bit = (*) Variable ["["]
    //   Bit = (*) Variable [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) Variable [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) Variable [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) Variable [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" ["=>"]
    //   Bit = (*) "[" Fact "]" ["["]
    //   Bit = (*) "[" Fact "]" [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) "[" Fact "]" [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["=>"]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["["]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["=>"]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["["]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["=>"]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["["]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit ["=>"]
    //   Bit+ = (*) Bit ["["]
    //   Bit+ = (*) Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit ["=>"]
    //   Bit+ = (*) Bit+ Bit ["["]
    //   Bit+ = (*) Bit+ Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit+ Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   FactApply = (*) Application ["=>"]
    //   FactFunc = (*) FactApply "=>" FactFunc [","]
    //   FactFunc = (*) FactApply "=>" FactFunc [";"]
    //   FactFunc = (*) FactApply "=>" FactFunc ["]"]
    //   FactFunc = (*) Variable "->" FactFunc [","]
    //   FactFunc = (*) Variable "->" FactFunc [";"]
    //   FactFunc = (*) Variable "->" FactFunc ["]"]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [","]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [";"]
    //   FactFunc = (*) "exists" Variable "->" FactFunc ["]"]
    //   FactFunc = "exists" Variable "->" (*) FactFunc [","]
    //   FactFunc = "exists" Variable "->" (*) FactFunc [";"]
    //   FactFunc = "exists" Variable "->" (*) FactFunc ["]"]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [","]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [";"]
    //   FactFunc = (*) "forall" Variable "->" FactFunc ["]"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["->"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["=>"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["["]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "[" -> Shift(S27)
    //   "exists" -> Shift(S28)
    //   "forall" -> Shift(S29)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Shift(S30)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Shift(S31)
    //   r#"[A-Za-z0-9_]+:"# -> Shift(S32)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Shift(S33)
    //
    //   Application -> S18
    //   Bit -> S19
    //   Bit+ -> S20
    //   FactApply -> S23
    //   FactFunc -> S74
    //   Variable -> S26
    pub fn __state66<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<&'input str>,
        __sym1: &mut Option<Variable>,
        __sym2: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (6, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state27(input, __lookbehind, __tokens, __sym3));
            }
            Some((_, (8, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state28(input, __lookbehind, __tokens, __sym3));
            }
            Some((_, (9, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state29(input, __lookbehind, __tokens, __sym3));
            }
            Some((_, (10, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state30(input, __lookbehind, __tokens, __sym3));
            }
            Some((_, (11, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state31(input, __lookbehind, __tokens, __sym3));
            }
            Some((_, (12, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state32(input, __lookbehind, __tokens, __sym3));
            }
            Some((_, (13, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state33(input, __lookbehind, __tokens, __sym3));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        while __sym2.is_some() {
            let (__lookbehind, __lookahead, __nt) = __result;
            match __nt {
                __Nonterminal::Application(__nt) => {
                    let __sym3 = &mut Some(__nt);
                    __result = try!(__state18(input, __lookbehind, __tokens, __lookahead, __sym3));
                }
                __Nonterminal::Bit(__nt) => {
                    let __sym3 = &mut Some(__nt);
                    __result = try!(__state19(input, __lookbehind, __tokens, __lookahead, __sym3));
                }
                __Nonterminal::Bit_2b(__nt) => {
                    let __sym3 = &mut Some(__nt);
                    __result = try!(__state20(input, __lookbehind, __tokens, __lookahead, __sym3));
                }
                __Nonterminal::FactApply(__nt) => {
                    let __sym3 = &mut Some(__nt);
                    __result = try!(__state23(input, __lookbehind, __tokens, __lookahead, __sym3));
                }
                __Nonterminal::FactFunc(__nt) => {
                    let __sym3 = &mut Some(__nt);
                    __result = try!(__state74(input, __lookbehind, __tokens, __lookahead, __sym0, __sym1, __sym2, __sym3));
                }
                __Nonterminal::Variable(__nt) => {
                    let __sym3 = &mut Some(__nt);
                    __result = try!(__state26(input, __lookbehind, __tokens, __lookahead, __sym3));
                }
                _ => {
                    return Ok((__lookbehind, __lookahead, __nt));
                }
            }
        }
        return Ok(__result);
    }

    // State 67
    //   Application = (*) Bit+ ["=>"]
    //   Bit = (*) Variable ["=>"]
    //   Bit = (*) Variable ["["]
    //   Bit = (*) Variable [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) Variable [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) Variable [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) Variable [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" ["=>"]
    //   Bit = (*) "[" Fact "]" ["["]
    //   Bit = (*) "[" Fact "]" [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) "[" Fact "]" [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["=>"]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["["]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["=>"]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["["]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["=>"]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["["]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit ["=>"]
    //   Bit+ = (*) Bit ["["]
    //   Bit+ = (*) Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit ["=>"]
    //   Bit+ = (*) Bit+ Bit ["["]
    //   Bit+ = (*) Bit+ Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit+ Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   FactApply = (*) Application ["=>"]
    //   FactFunc = (*) FactApply "=>" FactFunc [","]
    //   FactFunc = (*) FactApply "=>" FactFunc [";"]
    //   FactFunc = (*) FactApply "=>" FactFunc ["]"]
    //   FactFunc = (*) Variable "->" FactFunc [","]
    //   FactFunc = (*) Variable "->" FactFunc [";"]
    //   FactFunc = (*) Variable "->" FactFunc ["]"]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [","]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [";"]
    //   FactFunc = (*) "exists" Variable "->" FactFunc ["]"]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [","]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [";"]
    //   FactFunc = (*) "forall" Variable "->" FactFunc ["]"]
    //   FactFunc = "forall" Variable "->" (*) FactFunc [","]
    //   FactFunc = "forall" Variable "->" (*) FactFunc [";"]
    //   FactFunc = "forall" Variable "->" (*) FactFunc ["]"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["->"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["=>"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["["]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "[" -> Shift(S27)
    //   "exists" -> Shift(S28)
    //   "forall" -> Shift(S29)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Shift(S30)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Shift(S31)
    //   r#"[A-Za-z0-9_]+:"# -> Shift(S32)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Shift(S33)
    //
    //   Application -> S18
    //   Bit -> S19
    //   Bit+ -> S20
    //   FactApply -> S23
    //   FactFunc -> S75
    //   Variable -> S26
    pub fn __state67<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<&'input str>,
        __sym1: &mut Option<Variable>,
        __sym2: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (6, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state27(input, __lookbehind, __tokens, __sym3));
            }
            Some((_, (8, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state28(input, __lookbehind, __tokens, __sym3));
            }
            Some((_, (9, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state29(input, __lookbehind, __tokens, __sym3));
            }
            Some((_, (10, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state30(input, __lookbehind, __tokens, __sym3));
            }
            Some((_, (11, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state31(input, __lookbehind, __tokens, __sym3));
            }
            Some((_, (12, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state32(input, __lookbehind, __tokens, __sym3));
            }
            Some((_, (13, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state33(input, __lookbehind, __tokens, __sym3));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        while __sym2.is_some() {
            let (__lookbehind, __lookahead, __nt) = __result;
            match __nt {
                __Nonterminal::Application(__nt) => {
                    let __sym3 = &mut Some(__nt);
                    __result = try!(__state18(input, __lookbehind, __tokens, __lookahead, __sym3));
                }
                __Nonterminal::Bit(__nt) => {
                    let __sym3 = &mut Some(__nt);
                    __result = try!(__state19(input, __lookbehind, __tokens, __lookahead, __sym3));
                }
                __Nonterminal::Bit_2b(__nt) => {
                    let __sym3 = &mut Some(__nt);
                    __result = try!(__state20(input, __lookbehind, __tokens, __lookahead, __sym3));
                }
                __Nonterminal::FactApply(__nt) => {
                    let __sym3 = &mut Some(__nt);
                    __result = try!(__state23(input, __lookbehind, __tokens, __lookahead, __sym3));
                }
                __Nonterminal::FactFunc(__nt) => {
                    let __sym3 = &mut Some(__nt);
                    __result = try!(__state75(input, __lookbehind, __tokens, __lookahead, __sym0, __sym1, __sym2, __sym3));
                }
                __Nonterminal::Variable(__nt) => {
                    let __sym3 = &mut Some(__nt);
                    __result = try!(__state26(input, __lookbehind, __tokens, __lookahead, __sym3));
                }
                _ => {
                    return Ok((__lookbehind, __lookahead, __nt));
                }
            }
        }
        return Ok(__result);
    }

    // State 68
    //   FactAnd = FactAnd ";" FactOr (*) ["."]
    //   FactAnd = FactAnd ";" FactOr (*) [";"]
    //   FactOr = FactOr (*) "," FactFunc [","]
    //   FactOr = FactOr (*) "," FactFunc ["."]
    //   FactOr = FactOr (*) "," FactFunc [";"]
    //
    //   "," -> Shift(S57)
    //   "." -> Reduce(FactAnd = FactAnd, ";", FactOr => ActionFn(7);)
    //   ";" -> Reduce(FactAnd = FactAnd, ";", FactOr => ActionFn(7);)
    //
    pub fn __state68<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Fact>,
        __sym1: &mut Option<&'input str>,
        __sym2: &mut Option<Fact>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (0, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state57(input, __lookbehind, __tokens, __sym2, __sym3));
            }
            Some((_, (2, _), _)) |
            Some((_, (4, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __sym1 = __sym1.take().unwrap();
                let __sym2 = __sym2.take().unwrap();
                let __nt = super::__action7(input, __sym0, __sym1, __sym2, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::FactAnd(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        return Ok(__result);
    }

    // State 69
    //   FactFunc = FactApply "=>" FactFunc (*) [","]
    //   FactFunc = FactApply "=>" FactFunc (*) ["."]
    //   FactFunc = FactApply "=>" FactFunc (*) [";"]
    //
    //   "," -> Reduce(FactFunc = FactApply, "=>", FactFunc => ActionFn(10);)
    //   "." -> Reduce(FactFunc = FactApply, "=>", FactFunc => ActionFn(10);)
    //   ";" -> Reduce(FactFunc = FactApply, "=>", FactFunc => ActionFn(10);)
    //
    pub fn __state69<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Fact>,
        __sym1: &mut Option<&'input str>,
        __sym2: &mut Option<Fact>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (0, _), _)) |
            Some((_, (2, _), _)) |
            Some((_, (4, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __sym1 = __sym1.take().unwrap();
                let __sym2 = __sym2.take().unwrap();
                let __nt = super::__action10(input, __sym0, __sym1, __sym2, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::FactFunc(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 70
    //   FactOr = FactOr "," FactFunc (*) [","]
    //   FactOr = FactOr "," FactFunc (*) ["."]
    //   FactOr = FactOr "," FactFunc (*) [";"]
    //
    //   "," -> Reduce(FactOr = FactOr, ",", FactFunc => ActionFn(9);)
    //   "." -> Reduce(FactOr = FactOr, ",", FactFunc => ActionFn(9);)
    //   ";" -> Reduce(FactOr = FactOr, ",", FactFunc => ActionFn(9);)
    //
    pub fn __state70<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Fact>,
        __sym1: &mut Option<&'input str>,
        __sym2: &mut Option<Fact>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (0, _), _)) |
            Some((_, (2, _), _)) |
            Some((_, (4, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __sym1 = __sym1.take().unwrap();
                let __sym2 = __sym2.take().unwrap();
                let __nt = super::__action9(input, __sym0, __sym1, __sym2, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::FactOr(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 71
    //   FactFunc = Variable "->" FactFunc (*) [","]
    //   FactFunc = Variable "->" FactFunc (*) ["."]
    //   FactFunc = Variable "->" FactFunc (*) [";"]
    //
    //   "," -> Reduce(FactFunc = Variable, "->", FactFunc => ActionFn(13);)
    //   "." -> Reduce(FactFunc = Variable, "->", FactFunc => ActionFn(13);)
    //   ";" -> Reduce(FactFunc = Variable, "->", FactFunc => ActionFn(13);)
    //
    pub fn __state71<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<Variable>,
        __sym1: &mut Option<&'input str>,
        __sym2: &mut Option<Fact>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (0, _), _)) |
            Some((_, (2, _), _)) |
            Some((_, (4, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __sym1 = __sym1.take().unwrap();
                let __sym2 = __sym2.take().unwrap();
                let __nt = super::__action13(input, __sym0, __sym1, __sym2, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::FactFunc(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 72
    //   Application = (*) Bit+ ["=>"]
    //   Bit = (*) Variable ["=>"]
    //   Bit = (*) Variable ["["]
    //   Bit = (*) Variable [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) Variable [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) Variable [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) Variable [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" ["=>"]
    //   Bit = (*) "[" Fact "]" ["["]
    //   Bit = (*) "[" Fact "]" [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) "[" Fact "]" [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["=>"]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["["]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["=>"]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["["]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["=>"]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["["]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit ["=>"]
    //   Bit+ = (*) Bit ["["]
    //   Bit+ = (*) Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit ["=>"]
    //   Bit+ = (*) Bit+ Bit ["["]
    //   Bit+ = (*) Bit+ Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit+ Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   FactApply = (*) Application ["=>"]
    //   FactFunc = (*) FactApply "=>" FactFunc [","]
    //   FactFunc = (*) FactApply "=>" FactFunc ["."]
    //   FactFunc = (*) FactApply "=>" FactFunc [";"]
    //   FactFunc = (*) Variable "->" FactFunc [","]
    //   FactFunc = (*) Variable "->" FactFunc ["."]
    //   FactFunc = (*) Variable "->" FactFunc [";"]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [","]
    //   FactFunc = (*) "exists" Variable "->" FactFunc ["."]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [";"]
    //   FactFunc = "exists" Variable "->" (*) FactFunc [","]
    //   FactFunc = "exists" Variable "->" (*) FactFunc ["."]
    //   FactFunc = "exists" Variable "->" (*) FactFunc [";"]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [","]
    //   FactFunc = (*) "forall" Variable "->" FactFunc ["."]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [";"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["->"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["=>"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["["]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "[" -> Shift(S27)
    //   "exists" -> Shift(S40)
    //   "forall" -> Shift(S41)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Shift(S30)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Shift(S31)
    //   r#"[A-Za-z0-9_]+:"# -> Shift(S32)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Shift(S33)
    //
    //   Application -> S18
    //   Bit -> S19
    //   Bit+ -> S20
    //   FactApply -> S36
    //   FactFunc -> S76
    //   Variable -> S39
    pub fn __state72<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<&'input str>,
        __sym1: &mut Option<Variable>,
        __sym2: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (6, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state27(input, __lookbehind, __tokens, __sym3));
            }
            Some((_, (8, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state40(input, __lookbehind, __tokens, __sym3));
            }
            Some((_, (9, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state41(input, __lookbehind, __tokens, __sym3));
            }
            Some((_, (10, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state30(input, __lookbehind, __tokens, __sym3));
            }
            Some((_, (11, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state31(input, __lookbehind, __tokens, __sym3));
            }
            Some((_, (12, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state32(input, __lookbehind, __tokens, __sym3));
            }
            Some((_, (13, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state33(input, __lookbehind, __tokens, __sym3));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        while __sym2.is_some() {
            let (__lookbehind, __lookahead, __nt) = __result;
            match __nt {
                __Nonterminal::Application(__nt) => {
                    let __sym3 = &mut Some(__nt);
                    __result = try!(__state18(input, __lookbehind, __tokens, __lookahead, __sym3));
                }
                __Nonterminal::Bit(__nt) => {
                    let __sym3 = &mut Some(__nt);
                    __result = try!(__state19(input, __lookbehind, __tokens, __lookahead, __sym3));
                }
                __Nonterminal::Bit_2b(__nt) => {
                    let __sym3 = &mut Some(__nt);
                    __result = try!(__state20(input, __lookbehind, __tokens, __lookahead, __sym3));
                }
                __Nonterminal::FactApply(__nt) => {
                    let __sym3 = &mut Some(__nt);
                    __result = try!(__state36(input, __lookbehind, __tokens, __lookahead, __sym3));
                }
                __Nonterminal::FactFunc(__nt) => {
                    let __sym3 = &mut Some(__nt);
                    __result = try!(__state76(input, __lookbehind, __tokens, __lookahead, __sym0, __sym1, __sym2, __sym3));
                }
                __Nonterminal::Variable(__nt) => {
                    let __sym3 = &mut Some(__nt);
                    __result = try!(__state39(input, __lookbehind, __tokens, __lookahead, __sym3));
                }
                _ => {
                    return Ok((__lookbehind, __lookahead, __nt));
                }
            }
        }
        return Ok(__result);
    }

    // State 73
    //   Application = (*) Bit+ ["=>"]
    //   Bit = (*) Variable ["=>"]
    //   Bit = (*) Variable ["["]
    //   Bit = (*) Variable [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) Variable [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) Variable [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) Variable [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" ["=>"]
    //   Bit = (*) "[" Fact "]" ["["]
    //   Bit = (*) "[" Fact "]" [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) "[" Fact "]" [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) "[" Fact "]" [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["=>"]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# ["["]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[-!@#$%^&*=+/?~\\\\;,.]+"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["=>"]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# ["["]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[A-Za-z0-9_]+:"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["=>"]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# ["["]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Bit = (*) r#"[a-z_][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit ["=>"]
    //   Bit+ = (*) Bit ["["]
    //   Bit+ = (*) Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit ["=>"]
    //   Bit+ = (*) Bit+ Bit ["["]
    //   Bit+ = (*) Bit+ Bit [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Bit+ = (*) Bit+ Bit [r#"[A-Za-z0-9_]+:"#]
    //   Bit+ = (*) Bit+ Bit [r#"[a-z_][_A-Za-z0-9]*"#]
    //   FactApply = (*) Application ["=>"]
    //   FactFunc = (*) FactApply "=>" FactFunc [","]
    //   FactFunc = (*) FactApply "=>" FactFunc ["."]
    //   FactFunc = (*) FactApply "=>" FactFunc [";"]
    //   FactFunc = (*) Variable "->" FactFunc [","]
    //   FactFunc = (*) Variable "->" FactFunc ["."]
    //   FactFunc = (*) Variable "->" FactFunc [";"]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [","]
    //   FactFunc = (*) "exists" Variable "->" FactFunc ["."]
    //   FactFunc = (*) "exists" Variable "->" FactFunc [";"]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [","]
    //   FactFunc = (*) "forall" Variable "->" FactFunc ["."]
    //   FactFunc = (*) "forall" Variable "->" FactFunc [";"]
    //   FactFunc = "forall" Variable "->" (*) FactFunc [","]
    //   FactFunc = "forall" Variable "->" (*) FactFunc ["."]
    //   FactFunc = "forall" Variable "->" (*) FactFunc [";"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["->"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["=>"]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# ["["]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[-!@#$%^&*=+/?~\\\\;,.]+"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Z][_A-Za-z0-9]*"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[A-Za-z0-9_]+:"#]
    //   Variable = (*) r#"[A-Z][_A-Za-z0-9]*"# [r#"[a-z_][_A-Za-z0-9]*"#]
    //
    //   "[" -> Shift(S27)
    //   "exists" -> Shift(S40)
    //   "forall" -> Shift(S41)
    //   r#"[-!@#$%^&*=+/?~\\\\;,.]+"# -> Shift(S30)
    //   r#"[A-Z][_A-Za-z0-9]*"# -> Shift(S31)
    //   r#"[A-Za-z0-9_]+:"# -> Shift(S32)
    //   r#"[a-z_][_A-Za-z0-9]*"# -> Shift(S33)
    //
    //   Application -> S18
    //   Bit -> S19
    //   Bit+ -> S20
    //   FactApply -> S36
    //   FactFunc -> S77
    //   Variable -> S39
    pub fn __state73<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __sym0: &mut Option<&'input str>,
        __sym1: &mut Option<Variable>,
        __sym2: &mut Option<&'input str>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        let __lookahead = match __tokens.next() {
            Some(Ok(v)) => Some(v),
            None => None,
            Some(Err(e)) => return Err(e),
        };
        match __lookahead {
            Some((_, (6, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state27(input, __lookbehind, __tokens, __sym3));
            }
            Some((_, (8, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state40(input, __lookbehind, __tokens, __sym3));
            }
            Some((_, (9, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state41(input, __lookbehind, __tokens, __sym3));
            }
            Some((_, (10, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state30(input, __lookbehind, __tokens, __sym3));
            }
            Some((_, (11, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state31(input, __lookbehind, __tokens, __sym3));
            }
            Some((_, (12, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state32(input, __lookbehind, __tokens, __sym3));
            }
            Some((_, (13, __tok0), __loc)) => {
                let mut __lookbehind = Some(__loc);
                let mut __sym3 = &mut Some((__tok0));
                __result = try!(__state33(input, __lookbehind, __tokens, __sym3));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
        while __sym2.is_some() {
            let (__lookbehind, __lookahead, __nt) = __result;
            match __nt {
                __Nonterminal::Application(__nt) => {
                    let __sym3 = &mut Some(__nt);
                    __result = try!(__state18(input, __lookbehind, __tokens, __lookahead, __sym3));
                }
                __Nonterminal::Bit(__nt) => {
                    let __sym3 = &mut Some(__nt);
                    __result = try!(__state19(input, __lookbehind, __tokens, __lookahead, __sym3));
                }
                __Nonterminal::Bit_2b(__nt) => {
                    let __sym3 = &mut Some(__nt);
                    __result = try!(__state20(input, __lookbehind, __tokens, __lookahead, __sym3));
                }
                __Nonterminal::FactApply(__nt) => {
                    let __sym3 = &mut Some(__nt);
                    __result = try!(__state36(input, __lookbehind, __tokens, __lookahead, __sym3));
                }
                __Nonterminal::FactFunc(__nt) => {
                    let __sym3 = &mut Some(__nt);
                    __result = try!(__state77(input, __lookbehind, __tokens, __lookahead, __sym0, __sym1, __sym2, __sym3));
                }
                __Nonterminal::Variable(__nt) => {
                    let __sym3 = &mut Some(__nt);
                    __result = try!(__state39(input, __lookbehind, __tokens, __lookahead, __sym3));
                }
                _ => {
                    return Ok((__lookbehind, __lookahead, __nt));
                }
            }
        }
        return Ok(__result);
    }

    // State 74
    //   FactFunc = "exists" Variable "->" FactFunc (*) [","]
    //   FactFunc = "exists" Variable "->" FactFunc (*) [";"]
    //   FactFunc = "exists" Variable "->" FactFunc (*) ["]"]
    //
    //   "," -> Reduce(FactFunc = "exists", Variable, "->", FactFunc => ActionFn(11);)
    //   ";" -> Reduce(FactFunc = "exists", Variable, "->", FactFunc => ActionFn(11);)
    //   "]" -> Reduce(FactFunc = "exists", Variable, "->", FactFunc => ActionFn(11);)
    //
    pub fn __state74<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<&'input str>,
        __sym1: &mut Option<Variable>,
        __sym2: &mut Option<&'input str>,
        __sym3: &mut Option<Fact>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (0, _), _)) |
            Some((_, (4, _), _)) |
            Some((_, (7, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __sym1 = __sym1.take().unwrap();
                let __sym2 = __sym2.take().unwrap();
                let __sym3 = __sym3.take().unwrap();
                let __nt = super::__action11(input, __sym0, __sym1, __sym2, __sym3, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::FactFunc(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 75
    //   FactFunc = "forall" Variable "->" FactFunc (*) [","]
    //   FactFunc = "forall" Variable "->" FactFunc (*) [";"]
    //   FactFunc = "forall" Variable "->" FactFunc (*) ["]"]
    //
    //   "," -> Reduce(FactFunc = "forall", Variable, "->", FactFunc => ActionFn(12);)
    //   ";" -> Reduce(FactFunc = "forall", Variable, "->", FactFunc => ActionFn(12);)
    //   "]" -> Reduce(FactFunc = "forall", Variable, "->", FactFunc => ActionFn(12);)
    //
    pub fn __state75<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<&'input str>,
        __sym1: &mut Option<Variable>,
        __sym2: &mut Option<&'input str>,
        __sym3: &mut Option<Fact>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (0, _), _)) |
            Some((_, (4, _), _)) |
            Some((_, (7, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __sym1 = __sym1.take().unwrap();
                let __sym2 = __sym2.take().unwrap();
                let __sym3 = __sym3.take().unwrap();
                let __nt = super::__action12(input, __sym0, __sym1, __sym2, __sym3, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::FactFunc(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 76
    //   FactFunc = "exists" Variable "->" FactFunc (*) [","]
    //   FactFunc = "exists" Variable "->" FactFunc (*) ["."]
    //   FactFunc = "exists" Variable "->" FactFunc (*) [";"]
    //
    //   "," -> Reduce(FactFunc = "exists", Variable, "->", FactFunc => ActionFn(11);)
    //   "." -> Reduce(FactFunc = "exists", Variable, "->", FactFunc => ActionFn(11);)
    //   ";" -> Reduce(FactFunc = "exists", Variable, "->", FactFunc => ActionFn(11);)
    //
    pub fn __state76<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<&'input str>,
        __sym1: &mut Option<Variable>,
        __sym2: &mut Option<&'input str>,
        __sym3: &mut Option<Fact>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (0, _), _)) |
            Some((_, (2, _), _)) |
            Some((_, (4, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __sym1 = __sym1.take().unwrap();
                let __sym2 = __sym2.take().unwrap();
                let __sym3 = __sym3.take().unwrap();
                let __nt = super::__action11(input, __sym0, __sym1, __sym2, __sym3, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::FactFunc(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }

    // State 77
    //   FactFunc = "forall" Variable "->" FactFunc (*) [","]
    //   FactFunc = "forall" Variable "->" FactFunc (*) ["."]
    //   FactFunc = "forall" Variable "->" FactFunc (*) [";"]
    //
    //   "," -> Reduce(FactFunc = "forall", Variable, "->", FactFunc => ActionFn(12);)
    //   "." -> Reduce(FactFunc = "forall", Variable, "->", FactFunc => ActionFn(12);)
    //   ";" -> Reduce(FactFunc = "forall", Variable, "->", FactFunc => ActionFn(12);)
    //
    pub fn __state77<
        'input,
        __TOKENS: Iterator<Item=Result<(usize, (usize, &'input str), usize),__ParseError<usize,(usize, &'input str),()>>>,
    >(
        input: &'input str,
        __lookbehind: Option<usize>,
        __tokens: &mut __TOKENS,
        __lookahead: Option<(usize, (usize, &'input str), usize)>,
        __sym0: &mut Option<&'input str>,
        __sym1: &mut Option<Variable>,
        __sym2: &mut Option<&'input str>,
        __sym3: &mut Option<Fact>,
    ) -> Result<(Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>), __ParseError<usize,(usize, &'input str),()>>
    {
        let mut __result: (Option<usize>, Option<(usize, (usize, &'input str), usize)>, __Nonterminal<>);
        match __lookahead {
            Some((_, (0, _), _)) |
            Some((_, (2, _), _)) |
            Some((_, (4, _), _)) => {
                let __sym0 = __sym0.take().unwrap();
                let __sym1 = __sym1.take().unwrap();
                let __sym2 = __sym2.take().unwrap();
                let __sym3 = __sym3.take().unwrap();
                let __nt = super::__action12(input, __sym0, __sym1, __sym2, __sym3, &__lookbehind, &__lookahead);
                return Ok((__lookbehind, __lookahead, __Nonterminal::FactFunc(__nt)));
            }
            _ => {
                return Err(__ParseError::UnrecognizedToken {
                    token: __lookahead,
                    expected: vec![],
                });
            }
        }
    }
}
pub use self::__parse__Program::parse_Program;
mod __intern_token {
    extern crate lalrpop_util as __lalrpop_util;
    use self::__lalrpop_util::ParseError as __ParseError;
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
                    match __ch {
                        '!' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '#' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '$' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '%' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '&' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '*' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '+' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        ',' => {
                            __current_match = Some((0, __index + 1));
                            __current_state = 2;
                            continue;
                        }
                        '-' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 3;
                            continue;
                        }
                        '.' => {
                            __current_match = Some((2, __index + 1));
                            __current_state = 4;
                            continue;
                        }
                        '/' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '0' => {
                            __current_state = 5;
                            continue;
                        }
                        '1' => {
                            __current_state = 5;
                            continue;
                        }
                        '2' => {
                            __current_state = 5;
                            continue;
                        }
                        '3' => {
                            __current_state = 5;
                            continue;
                        }
                        '4' => {
                            __current_state = 5;
                            continue;
                        }
                        '5' => {
                            __current_state = 5;
                            continue;
                        }
                        '6' => {
                            __current_state = 5;
                            continue;
                        }
                        '7' => {
                            __current_state = 5;
                            continue;
                        }
                        '8' => {
                            __current_state = 5;
                            continue;
                        }
                        '9' => {
                            __current_state = 5;
                            continue;
                        }
                        ':' => {
                            __current_state = 6;
                            continue;
                        }
                        ';' => {
                            __current_match = Some((4, __index + 1));
                            __current_state = 7;
                            continue;
                        }
                        '=' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 8;
                            continue;
                        }
                        '?' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '@' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        'A' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'B' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'C' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'D' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'E' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'F' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'G' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'H' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'I' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'J' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'K' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'L' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'M' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'N' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'O' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'P' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'Q' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'R' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'S' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'T' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'U' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'V' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'W' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'X' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'Y' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'Z' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        '[' => {
                            __current_match = Some((6, __index + 1));
                            __current_state = 10;
                            continue;
                        }
                        '\\' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        ']' => {
                            __current_match = Some((7, __index + 1));
                            __current_state = 11;
                            continue;
                        }
                        '^' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '_' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'a' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'b' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'c' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'd' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'e' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 13;
                            continue;
                        }
                        'f' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 14;
                            continue;
                        }
                        'g' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'h' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'i' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'j' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'k' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'l' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'm' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'n' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'o' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'p' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'r' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        's' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        't' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'u' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'v' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'w' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'x' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '~' => {
                            __current_match = Some((10, __index + 1));
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
                    match __ch {
                        '!' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '#' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '$' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '%' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '&' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '*' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '+' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        ',' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '-' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '.' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '/' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        ';' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '=' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '?' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '@' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '\\' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '^' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '~' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                2 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        '!' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '#' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '$' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '%' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '&' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '*' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '+' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        ',' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '-' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '.' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '/' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        ';' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '=' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '?' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '@' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '\\' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '^' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '~' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                3 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        '!' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '#' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '$' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '%' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '&' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '*' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '+' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        ',' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '-' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '.' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '/' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        ';' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '=' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '>' => {
                            __current_match = Some((1, __index + 1));
                            __current_state = 16;
                            continue;
                        }
                        '?' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '@' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '\\' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '^' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '~' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                4 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        '!' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '#' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '$' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '%' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '&' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '*' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '+' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        ',' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '-' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '.' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '/' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        ';' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '=' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '?' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '@' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '\\' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '^' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '~' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                5 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        '0' => {
                            __current_state = 5;
                            continue;
                        }
                        '1' => {
                            __current_state = 5;
                            continue;
                        }
                        '2' => {
                            __current_state = 5;
                            continue;
                        }
                        '3' => {
                            __current_state = 5;
                            continue;
                        }
                        '4' => {
                            __current_state = 5;
                            continue;
                        }
                        '5' => {
                            __current_state = 5;
                            continue;
                        }
                        '6' => {
                            __current_state = 5;
                            continue;
                        }
                        '7' => {
                            __current_state = 5;
                            continue;
                        }
                        '8' => {
                            __current_state = 5;
                            continue;
                        }
                        '9' => {
                            __current_state = 5;
                            continue;
                        }
                        ':' => {
                            __current_match = Some((12, __index + 1));
                            __current_state = 17;
                            continue;
                        }
                        'A' => {
                            __current_state = 5;
                            continue;
                        }
                        'B' => {
                            __current_state = 5;
                            continue;
                        }
                        'C' => {
                            __current_state = 5;
                            continue;
                        }
                        'D' => {
                            __current_state = 5;
                            continue;
                        }
                        'E' => {
                            __current_state = 5;
                            continue;
                        }
                        'F' => {
                            __current_state = 5;
                            continue;
                        }
                        'G' => {
                            __current_state = 5;
                            continue;
                        }
                        'H' => {
                            __current_state = 5;
                            continue;
                        }
                        'I' => {
                            __current_state = 5;
                            continue;
                        }
                        'J' => {
                            __current_state = 5;
                            continue;
                        }
                        'K' => {
                            __current_state = 5;
                            continue;
                        }
                        'L' => {
                            __current_state = 5;
                            continue;
                        }
                        'M' => {
                            __current_state = 5;
                            continue;
                        }
                        'N' => {
                            __current_state = 5;
                            continue;
                        }
                        'O' => {
                            __current_state = 5;
                            continue;
                        }
                        'P' => {
                            __current_state = 5;
                            continue;
                        }
                        'Q' => {
                            __current_state = 5;
                            continue;
                        }
                        'R' => {
                            __current_state = 5;
                            continue;
                        }
                        'S' => {
                            __current_state = 5;
                            continue;
                        }
                        'T' => {
                            __current_state = 5;
                            continue;
                        }
                        'U' => {
                            __current_state = 5;
                            continue;
                        }
                        'V' => {
                            __current_state = 5;
                            continue;
                        }
                        'W' => {
                            __current_state = 5;
                            continue;
                        }
                        'X' => {
                            __current_state = 5;
                            continue;
                        }
                        'Y' => {
                            __current_state = 5;
                            continue;
                        }
                        'Z' => {
                            __current_state = 5;
                            continue;
                        }
                        '_' => {
                            __current_state = 5;
                            continue;
                        }
                        'a' => {
                            __current_state = 5;
                            continue;
                        }
                        'b' => {
                            __current_state = 5;
                            continue;
                        }
                        'c' => {
                            __current_state = 5;
                            continue;
                        }
                        'd' => {
                            __current_state = 5;
                            continue;
                        }
                        'e' => {
                            __current_state = 5;
                            continue;
                        }
                        'f' => {
                            __current_state = 5;
                            continue;
                        }
                        'g' => {
                            __current_state = 5;
                            continue;
                        }
                        'h' => {
                            __current_state = 5;
                            continue;
                        }
                        'i' => {
                            __current_state = 5;
                            continue;
                        }
                        'j' => {
                            __current_state = 5;
                            continue;
                        }
                        'k' => {
                            __current_state = 5;
                            continue;
                        }
                        'l' => {
                            __current_state = 5;
                            continue;
                        }
                        'm' => {
                            __current_state = 5;
                            continue;
                        }
                        'n' => {
                            __current_state = 5;
                            continue;
                        }
                        'o' => {
                            __current_state = 5;
                            continue;
                        }
                        'p' => {
                            __current_state = 5;
                            continue;
                        }
                        'q' => {
                            __current_state = 5;
                            continue;
                        }
                        'r' => {
                            __current_state = 5;
                            continue;
                        }
                        's' => {
                            __current_state = 5;
                            continue;
                        }
                        't' => {
                            __current_state = 5;
                            continue;
                        }
                        'u' => {
                            __current_state = 5;
                            continue;
                        }
                        'v' => {
                            __current_state = 5;
                            continue;
                        }
                        'w' => {
                            __current_state = 5;
                            continue;
                        }
                        'x' => {
                            __current_state = 5;
                            continue;
                        }
                        'y' => {
                            __current_state = 5;
                            continue;
                        }
                        'z' => {
                            __current_state = 5;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                6 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        '-' => {
                            __current_match = Some((3, __index + 1));
                            __current_state = 18;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                7 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        '!' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '#' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '$' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '%' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '&' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '*' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '+' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        ',' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '-' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '.' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '/' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        ';' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '=' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '?' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '@' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '\\' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '^' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '~' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                8 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        '!' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '#' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '$' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '%' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '&' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '*' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '+' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        ',' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '-' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '.' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '/' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        ';' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '=' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '>' => {
                            __current_match = Some((5, __index + 1));
                            __current_state = 19;
                            continue;
                        }
                        '?' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '@' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '\\' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '^' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        '~' => {
                            __current_match = Some((10, __index + 1));
                            __current_state = 1;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                9 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        '0' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        '1' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        '2' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        '3' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        '4' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        '5' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        '6' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        '7' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        '8' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        '9' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        ':' => {
                            __current_match = Some((12, __index + 1));
                            __current_state = 17;
                            continue;
                        }
                        'A' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'B' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'C' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'D' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'E' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'F' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'G' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'H' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'I' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'J' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'K' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'L' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'M' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'N' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'O' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'P' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'Q' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'R' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'S' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'T' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'U' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'V' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'W' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'X' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'Y' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'Z' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        '_' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'a' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'b' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'c' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'd' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'e' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'f' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'g' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'h' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'i' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'j' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'k' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'l' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'm' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'n' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'o' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'p' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'q' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'r' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        's' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        't' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'u' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'v' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'w' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'x' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'y' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        'z' => {
                            __current_match = Some((11, __index + 1));
                            __current_state = 9;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                10 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        _ => {
                            return __current_match;
                        }
                    }
                }
                11 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        _ => {
                            return __current_match;
                        }
                    }
                }
                12 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        '0' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '1' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '2' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '3' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '4' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '5' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '6' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '7' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '8' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '9' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        ':' => {
                            __current_match = Some((12, __index + 1));
                            __current_state = 17;
                            continue;
                        }
                        'A' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'B' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'C' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'D' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'E' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'F' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'G' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'H' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'I' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'J' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'K' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'L' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'M' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'N' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'O' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'P' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'R' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'S' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'T' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'U' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'V' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'W' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'X' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '_' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'a' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'b' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'c' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'd' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'e' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'f' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'g' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'h' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'i' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'j' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'k' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'l' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'm' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'n' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'o' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'p' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'r' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        's' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        't' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'u' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'v' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'w' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'x' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                13 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        '0' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '1' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '2' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '3' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '4' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '5' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '6' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '7' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '8' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '9' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        ':' => {
                            __current_match = Some((12, __index + 1));
                            __current_state = 17;
                            continue;
                        }
                        'A' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'B' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'C' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'D' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'E' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'F' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'G' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'H' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'I' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'J' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'K' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'L' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'M' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'N' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'O' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'P' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'R' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'S' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'T' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'U' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'V' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'W' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'X' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '_' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'a' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'b' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'c' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'd' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'e' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'f' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'g' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'h' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'i' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'j' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'k' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'l' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'm' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'n' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'o' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'p' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'r' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        's' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        't' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'u' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'v' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'w' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'x' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 20;
                            continue;
                        }
                        'y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                14 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        '0' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '1' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '2' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '3' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '4' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '5' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '6' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '7' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '8' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '9' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        ':' => {
                            __current_match = Some((12, __index + 1));
                            __current_state = 17;
                            continue;
                        }
                        'A' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'B' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'C' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'D' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'E' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'F' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'G' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'H' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'I' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'J' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'K' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'L' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'M' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'N' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'O' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'P' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'R' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'S' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'T' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'U' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'V' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'W' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'X' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '_' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'a' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'b' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'c' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'd' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'e' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'f' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'g' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'h' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'i' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'j' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'k' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'l' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'm' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'n' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'o' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 21;
                            continue;
                        }
                        'p' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'r' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        's' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        't' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'u' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'v' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'w' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'x' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                15 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        _ => {
                            return __current_match;
                        }
                    }
                }
                16 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        _ => {
                            return __current_match;
                        }
                    }
                }
                17 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        _ => {
                            return __current_match;
                        }
                    }
                }
                18 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        _ => {
                            return __current_match;
                        }
                    }
                }
                19 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        _ => {
                            return __current_match;
                        }
                    }
                }
                20 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        '0' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '1' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '2' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '3' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '4' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '5' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '6' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '7' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '8' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '9' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        ':' => {
                            __current_match = Some((12, __index + 1));
                            __current_state = 17;
                            continue;
                        }
                        'A' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'B' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'C' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'D' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'E' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'F' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'G' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'H' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'I' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'J' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'K' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'L' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'M' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'N' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'O' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'P' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'R' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'S' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'T' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'U' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'V' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'W' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'X' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '_' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'a' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'b' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'c' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'd' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'e' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'f' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'g' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'h' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'i' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 22;
                            continue;
                        }
                        'j' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'k' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'l' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'm' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'n' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'o' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'p' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'r' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        's' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        't' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'u' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'v' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'w' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'x' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                21 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        '0' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '1' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '2' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '3' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '4' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '5' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '6' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '7' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '8' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '9' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        ':' => {
                            __current_match = Some((12, __index + 1));
                            __current_state = 17;
                            continue;
                        }
                        'A' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'B' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'C' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'D' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'E' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'F' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'G' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'H' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'I' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'J' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'K' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'L' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'M' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'N' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'O' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'P' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'R' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'S' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'T' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'U' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'V' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'W' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'X' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '_' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'a' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'b' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'c' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'd' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'e' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'f' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'g' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'h' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'i' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'j' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'k' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'l' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'm' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'n' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'o' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'p' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'r' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 23;
                            continue;
                        }
                        's' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        't' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'u' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'v' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'w' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'x' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                22 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        '0' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '1' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '2' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '3' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '4' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '5' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '6' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '7' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '8' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '9' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        ':' => {
                            __current_match = Some((12, __index + 1));
                            __current_state = 17;
                            continue;
                        }
                        'A' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'B' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'C' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'D' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'E' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'F' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'G' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'H' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'I' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'J' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'K' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'L' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'M' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'N' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'O' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'P' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'R' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'S' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'T' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'U' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'V' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'W' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'X' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '_' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'a' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'b' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'c' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'd' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'e' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'f' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'g' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'h' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'i' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'j' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'k' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'l' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'm' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'n' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'o' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'p' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'r' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        's' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 24;
                            continue;
                        }
                        't' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'u' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'v' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'w' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'x' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                23 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        '0' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '1' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '2' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '3' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '4' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '5' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '6' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '7' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '8' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '9' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        ':' => {
                            __current_match = Some((12, __index + 1));
                            __current_state = 17;
                            continue;
                        }
                        'A' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'B' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'C' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'D' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'E' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'F' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'G' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'H' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'I' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'J' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'K' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'L' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'M' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'N' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'O' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'P' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'R' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'S' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'T' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'U' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'V' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'W' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'X' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '_' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'a' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 25;
                            continue;
                        }
                        'b' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'c' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'd' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'e' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'f' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'g' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'h' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'i' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'j' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'k' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'l' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'm' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'n' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'o' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'p' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'r' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        's' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        't' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'u' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'v' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'w' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'x' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                24 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        '0' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '1' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '2' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '3' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '4' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '5' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '6' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '7' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '8' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '9' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        ':' => {
                            __current_match = Some((12, __index + 1));
                            __current_state = 17;
                            continue;
                        }
                        'A' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'B' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'C' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'D' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'E' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'F' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'G' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'H' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'I' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'J' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'K' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'L' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'M' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'N' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'O' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'P' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'R' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'S' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'T' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'U' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'V' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'W' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'X' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '_' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'a' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'b' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'c' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'd' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'e' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'f' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'g' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'h' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'i' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'j' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'k' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'l' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'm' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'n' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'o' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'p' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'r' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        's' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        't' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 26;
                            continue;
                        }
                        'u' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'v' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'w' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'x' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                25 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        '0' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '1' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '2' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '3' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '4' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '5' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '6' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '7' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '8' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '9' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        ':' => {
                            __current_match = Some((12, __index + 1));
                            __current_state = 17;
                            continue;
                        }
                        'A' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'B' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'C' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'D' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'E' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'F' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'G' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'H' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'I' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'J' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'K' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'L' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'M' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'N' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'O' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'P' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'R' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'S' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'T' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'U' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'V' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'W' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'X' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '_' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'a' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'b' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'c' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'd' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'e' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'f' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'g' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'h' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'i' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'j' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'k' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'l' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 27;
                            continue;
                        }
                        'm' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'n' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'o' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'p' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'r' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        's' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        't' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'u' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'v' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'w' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'x' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                26 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        '0' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '1' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '2' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '3' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '4' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '5' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '6' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '7' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '8' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '9' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        ':' => {
                            __current_match = Some((12, __index + 1));
                            __current_state = 17;
                            continue;
                        }
                        'A' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'B' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'C' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'D' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'E' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'F' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'G' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'H' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'I' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'J' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'K' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'L' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'M' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'N' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'O' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'P' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'R' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'S' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'T' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'U' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'V' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'W' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'X' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '_' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'a' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'b' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'c' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'd' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'e' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'f' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'g' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'h' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'i' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'j' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'k' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'l' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'm' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'n' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'o' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'p' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'r' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        's' => {
                            __current_match = Some((8, __index + 1));
                            __current_state = 28;
                            continue;
                        }
                        't' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'u' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'v' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'w' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'x' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                27 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        '0' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '1' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '2' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '3' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '4' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '5' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '6' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '7' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '8' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '9' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        ':' => {
                            __current_match = Some((12, __index + 1));
                            __current_state = 17;
                            continue;
                        }
                        'A' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'B' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'C' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'D' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'E' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'F' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'G' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'H' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'I' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'J' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'K' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'L' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'M' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'N' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'O' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'P' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'R' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'S' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'T' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'U' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'V' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'W' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'X' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '_' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'a' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'b' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'c' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'd' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'e' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'f' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'g' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'h' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'i' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'j' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'k' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'l' => {
                            __current_match = Some((9, __index + 1));
                            __current_state = 29;
                            continue;
                        }
                        'm' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'n' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'o' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'p' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'r' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        's' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        't' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'u' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'v' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'w' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'x' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                28 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        '0' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '1' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '2' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '3' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '4' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '5' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '6' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '7' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '8' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '9' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        ':' => {
                            __current_match = Some((12, __index + 1));
                            __current_state = 17;
                            continue;
                        }
                        'A' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'B' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'C' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'D' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'E' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'F' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'G' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'H' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'I' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'J' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'K' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'L' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'M' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'N' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'O' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'P' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'R' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'S' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'T' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'U' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'V' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'W' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'X' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '_' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'a' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'b' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'c' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'd' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'e' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'f' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'g' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'h' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'i' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'j' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'k' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'l' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'm' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'n' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'o' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'p' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'r' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        's' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        't' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'u' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'v' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'w' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'x' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        _ => {
                            return __current_match;
                        }
                    }
                }
                29 => {
                    let (__index, __ch) = match __chars.next() { Some(p) => p, None => return __current_match };
                    match __ch {
                        '0' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '1' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '2' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '3' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '4' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '5' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '6' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '7' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '8' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '9' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        ':' => {
                            __current_match = Some((12, __index + 1));
                            __current_state = 17;
                            continue;
                        }
                        'A' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'B' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'C' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'D' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'E' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'F' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'G' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'H' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'I' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'J' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'K' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'L' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'M' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'N' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'O' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'P' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'R' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'S' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'T' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'U' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'V' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'W' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'X' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'Z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        '_' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'a' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'b' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'c' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'd' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'e' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'f' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'g' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'h' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'i' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'j' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'k' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'l' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'm' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'n' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'o' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'p' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'q' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'r' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        's' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        't' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'u' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'v' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'w' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'x' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'y' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
                            continue;
                        }
                        'z' => {
                            __current_match = Some((13, __index + 1));
                            __current_state = 12;
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
        type Item = Result<(usize, (usize, &'input str), usize), __ParseError<usize,(usize, &'input str),()>>;

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
                        Some(Err(__ParseError::InvalidToken { location: __start_offset }))
                    }
                }
            }
        }
    }
}

pub fn __action0<
    'input,
>(
    input: &'input str,
    __0: Program,
    __lookbehind: &Option<usize>,
    __lookahead: &Option<(usize, (usize, &'input str), usize)>,
) -> Program
{
    (__0)
}

pub fn __action1<
    'input,
>(
    input: &'input str,
    __0: ::std::vec::Vec<Item>,
    __lookbehind: &Option<usize>,
    __lookahead: &Option<(usize, (usize, &'input str), usize)>,
) -> Program
{
    Program { items: __0 }
}

pub fn __action2<
    'input,
>(
    input: &'input str,
    a: Application,
    _: &'input str,
    __lookbehind: &Option<usize>,
    __lookahead: &Option<(usize, (usize, &'input str), usize)>,
) -> Item
{
    Item::Fact(a)
}

pub fn __action3<
    'input,
>(
    input: &'input str,
    __0: Rule,
    __lookbehind: &Option<usize>,
    __lookahead: &Option<(usize, (usize, &'input str), usize)>,
) -> Item
{
    Item::Rule(__0)
}

pub fn __action4<
    'input,
>(
    input: &'input str,
    a: Application,
    _: &'input str,
    f: Fact,
    _: &'input str,
    __lookbehind: &Option<usize>,
    __lookahead: &Option<(usize, (usize, &'input str), usize)>,
) -> Rule
{
    Rule {
        consequence: a,
        condition: f
    }
}

pub fn __action5<
    'input,
>(
    input: &'input str,
    __0: Fact,
    __lookbehind: &Option<usize>,
    __lookahead: &Option<(usize, (usize, &'input str), usize)>,
) -> Fact
{
    (__0)
}

pub fn __action6<
    'input,
>(
    input: &'input str,
    __0: Fact,
    __lookbehind: &Option<usize>,
    __lookahead: &Option<(usize, (usize, &'input str), usize)>,
) -> Fact
{
    (__0)
}

pub fn __action7<
    'input,
>(
    input: &'input str,
    l: Fact,
    _: &'input str,
    r: Fact,
    __lookbehind: &Option<usize>,
    __lookahead: &Option<(usize, (usize, &'input str), usize)>,
) -> Fact
{
    Fact { data: Box::new(FactData::And(l, r)) }
}

pub fn __action8<
    'input,
>(
    input: &'input str,
    __0: Fact,
    __lookbehind: &Option<usize>,
    __lookahead: &Option<(usize, (usize, &'input str), usize)>,
) -> Fact
{
    (__0)
}

pub fn __action9<
    'input,
>(
    input: &'input str,
    l: Fact,
    _: &'input str,
    r: Fact,
    __lookbehind: &Option<usize>,
    __lookahead: &Option<(usize, (usize, &'input str), usize)>,
) -> Fact
{
    Fact { data: Box::new(FactData::Or(l, r)) }
}

pub fn __action10<
    'input,
>(
    input: &'input str,
    l: Fact,
    _: &'input str,
    r: Fact,
    __lookbehind: &Option<usize>,
    __lookahead: &Option<(usize, (usize, &'input str), usize)>,
) -> Fact
{
    Fact { data: Box::new(FactData::Implication(l, r)) }
}

pub fn __action11<
    'input,
>(
    input: &'input str,
    _: &'input str,
    v: Variable,
    _: &'input str,
    b: Fact,
    __lookbehind: &Option<usize>,
    __lookahead: &Option<(usize, (usize, &'input str), usize)>,
) -> Fact
{
    Fact { data: Box::new(FactData::Exists(v, b)) }
}

pub fn __action12<
    'input,
>(
    input: &'input str,
    _: &'input str,
    v: Variable,
    _: &'input str,
    b: Fact,
    __lookbehind: &Option<usize>,
    __lookahead: &Option<(usize, (usize, &'input str), usize)>,
) -> Fact
{
    Fact { data: Box::new(FactData::ForAll(v, b)) }
}

pub fn __action13<
    'input,
>(
    input: &'input str,
    v: Variable,
    _: &'input str,
    b: Fact,
    __lookbehind: &Option<usize>,
    __lookahead: &Option<(usize, (usize, &'input str), usize)>,
) -> Fact
{
    Fact { data: Box::new(FactData::Lambda(v, b)) }
}

pub fn __action14<
    'input,
>(
    input: &'input str,
    __0: Application,
    __lookbehind: &Option<usize>,
    __lookahead: &Option<(usize, (usize, &'input str), usize)>,
) -> Fact
{
    Fact { data: Box::new(FactData::Apply(__0)) }
}

pub fn __action15<
    'input,
>(
    input: &'input str,
    __0: ::std::vec::Vec<Bit>,
    __lookbehind: &Option<usize>,
    __lookahead: &Option<(usize, (usize, &'input str), usize)>,
) -> Application
{
    Application { bits: __0 }
}

pub fn __action16<
    'input,
>(
    input: &'input str,
    __0: &'input str,
    __lookbehind: &Option<usize>,
    __lookahead: &Option<(usize, (usize, &'input str), usize)>,
) -> Bit
{
    Bit::Operator(Operator { id: intern(__0) })
}

pub fn __action17<
    'input,
>(
    input: &'input str,
    __0: &'input str,
    __lookbehind: &Option<usize>,
    __lookahead: &Option<(usize, (usize, &'input str), usize)>,
) -> Bit
{
    Bit::Operator(Operator { id: intern(__0) })
}

pub fn __action18<
    'input,
>(
    input: &'input str,
    __0: &'input str,
    __lookbehind: &Option<usize>,
    __lookahead: &Option<(usize, (usize, &'input str), usize)>,
) -> Bit
{
    Bit::Atom(Atom { id: intern(__0) })
}

pub fn __action19<
    'input,
>(
    input: &'input str,
    __0: Variable,
    __lookbehind: &Option<usize>,
    __lookahead: &Option<(usize, (usize, &'input str), usize)>,
) -> Bit
{
    Bit::Variable(__0)
}

pub fn __action20<
    'input,
>(
    input: &'input str,
    _: &'input str,
    __0: Fact,
    _: &'input str,
    __lookbehind: &Option<usize>,
    __lookahead: &Option<(usize, (usize, &'input str), usize)>,
) -> Bit
{
    Bit::Paren(Box::new(__0))
}

pub fn __action21<
    'input,
>(
    input: &'input str,
    __0: &'input str,
    __lookbehind: &Option<usize>,
    __lookahead: &Option<(usize, (usize, &'input str), usize)>,
) -> Variable
{
    Variable { id: intern(__0) }
}

pub fn __action22<
    'input,
>(
    input: &'input str,
    __0: Bit,
    __lookbehind: &Option<usize>,
    __lookahead: &Option<(usize, (usize, &'input str), usize)>,
) -> ::std::vec::Vec<Bit>
{
    vec![__0]
}

pub fn __action23<
    'input,
>(
    input: &'input str,
    v: ::std::vec::Vec<Bit>,
    e: Bit,
    __lookbehind: &Option<usize>,
    __lookahead: &Option<(usize, (usize, &'input str), usize)>,
) -> ::std::vec::Vec<Bit>
{
    { let mut v = v; v.push(e); v }
}

pub fn __action24<
    'input,
>(
    input: &'input str,
    __0: Item,
    __lookbehind: &Option<usize>,
    __lookahead: &Option<(usize, (usize, &'input str), usize)>,
) -> ::std::vec::Vec<Item>
{
    vec![__0]
}

pub fn __action25<
    'input,
>(
    input: &'input str,
    v: ::std::vec::Vec<Item>,
    e: Item,
    __lookbehind: &Option<usize>,
    __lookahead: &Option<(usize, (usize, &'input str), usize)>,
) -> ::std::vec::Vec<Item>
{
    { let mut v = v; v.push(e); v }
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
