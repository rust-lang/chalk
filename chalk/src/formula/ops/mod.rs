use super::*;

use std::rc::Rc;

#[cfg(test)]
mod test;

impl Term {
    /// Reduce a term, in place, to head normal form. This means that
    /// all beta redexs (*) along top-most spine are contracted
    /// (**). Note that this creates suspensions where possible to
    /// avoid unnecessary work.
    ///
    /// (*) A beta redex is a term of the form `(fn x => t1) t2`.
    /// (**) A beta contraction is substituting `t2` for `x` in `t1`: `[t2 => x] t1`
    pub fn head_normal_form(self) {
        while self.head_normal_form_step() { }
    }

    fn head_normal_form_step(self) -> bool {
        let data = self.take();
        match data {
            TermData::Constant(_) |
            TermData::FreeVariable(_) |
            TermData::Lambda(_) |
            TermData::BoundVariable(_) => {
                self.replace(data);
                true
            }
            TermData::Apply(fun, argument) => {
                fun.head_normal_form_step_applied(self, argument)
            }
            TermData::Suspension(suspension) => {
                self.replace(suspension.step());
                true
            }
        }
    }

    fn head_normal_form_step_applied(self, app: Term, argument: Term) -> bool {
        let data = self.take();
        match data {
            TermData::Constant(_) |
            TermData::FreeVariable(_) |
            TermData::BoundVariable(_) => {
                app.replace(TermData::Apply(self, argument));
                self.replace(data);
                false
            }

            TermData::Apply(fun, argument) => {
                app.replace(TermData::Apply(self, argument));
                fun.head_normal_form_step_applied(self, argument)
            }

            TermData::Lambda(body) => {
                let suspension = Suspension {
                    term: body,
                    environment: Environment::with_term(argument),
                    bound: 1,
                    lifts: 0
                };
                let data = suspension.step();
                app.replace(data);
                self.free();
                true
            }

            TermData::Suspension(suspension) => {
                app.replace(TermData::Apply(self, argument));
                self.replace(suspension.step());
                true
            }
        }
    }
}

impl Environment {
    fn new() -> Environment {
        Environment {
            first_cell: None
        }
    }

    fn with_term(term: Term) -> Environment {
        Environment::new().prepend(CellData::Term(term))
    }

    fn with_cell(cell: Cell) -> Environment {
        Environment {
            first_cell: Some(Rc::new(cell))
        }
    }

    fn prepend(&self, data: CellData) -> Environment {
        Environment::with_cell(
            Cell {
                data: data,
                next: self.clone()
            })
    }

    fn get(&self, index: u32) -> &CellData {
        match self.first_cell {
            None => {
                panic!("out of bounds get in an environment")
            }

            Some(ref cell) => {
                if index == 0 {
                    &cell.data
                } else {
                    cell.next.get(index - 1)
                }
            }
        }
    }

    fn len(&self) -> usize {
        match self.first_cell {
            None => 0,
            Some(ref cell) => cell.next.len() + 1,
        }
    }
}

impl Suspension {
    pub fn new(term: Term,
               environment: Environment,
               bound: u32,
               lifts: u32)
               -> Suspension
    {
        Suspension {
            term: term,
            environment: environment,
            bound: bound,
            lifts: lifts,
        }
    }

    pub fn term(term: Term,
                environment: Environment,
                bound: u32,
                lifts: u32)
                -> Term
    {
        Term::new(Suspension::data(term, environment, bound, lifts))
    }

    pub fn data(term: Term,
                environment: Environment,
                bound: u32,
                lifts: u32)
                -> TermData
    {
        TermData::Suspension(Box::new(Suspension::new(term, environment, bound, lifts)))
    }

    pub fn step(self) -> TermData {
        let Suspension { term, environment, bound, lifts } = self;
        match term.take() {
            // go from `<λ body_term>` to `λ <body_term>`
            TermData::Lambda(body_term) => {
                // reuse `term` to hold `<body_term>`
                term.replace(Suspension::data(body_term,
                                              environment.prepend(CellData::Dummy(lifts)),
                                              bound + 1,
                                              lifts + 1));

                // and return `λ <body_term>`
                TermData::Lambda(term)
            }

            // go from `<t1 t2>` to `<t1> <t2>`
            TermData::Apply(t1, t2) => {
                // reuse `term` to hold `<t1>`
                term.replace(Suspension::data(t1, environment.clone(), bound, lifts));

                // create `<t2>`
                let t2 = Suspension::term(t2, environment, bound, lifts);

                TermData::Apply(term, t2)
            }

            // go from `<c>` to `c` and `<X>` to `X`
            data @ TermData::Constant(..) |
            data @ TermData::FreeVariable(..) => {
                data
            }

            // handle `#1` where the value for `#1` is bound in the substitution
            TermData::BoundVariable(index) if index.0 < self.bound => {
                // FIXME we could handle environments more efficiently
                // in the case that they are unaliased via Rc's
                // `try_unwrap` fns, but that would not be stable Rust
                match *environment.get(index.0) {
                    CellData::Term(ref substituted_term) => {
                        substituted_term.data(|d| match *d {
                            TermData::Suspension(ref suspension) => {
                                Suspension::data(suspension.term.clone(),
                                                 suspension.environment.clone(),
                                                 suspension.bound,
                                                 lifts)
                            }

                            _ => {
                                Suspension::data(substituted_term.clone(),
                                                 Environment::new(),
                                                 0,
                                                 lifts)
                            }
                        })
                    }

                    CellData::Dummy(lifts1) => {
                        let index = DebruijnIndex(lifts - lifts1);
                        TermData::BoundVariable(index)
                    }
                }
            }

            TermData::BoundVariable(index) => {
                let index = DebruijnIndex(index.0 - bound + lifts);
                TermData::BoundVariable(index)
            }

            TermData::Suspension(data) => {
                // this isn't in the paper, but it seems like the
                // logical thing to do --nmatsakis
                term.replace(data.step());
                Suspension::data(term, environment, bound, lifts)
            }
        }
    }
}

//impl TermData {
//    pub fn desuspend(self) -> TermData {
//        use TermData::*;
//
//        match self {
//            Constant(..) => self,
//            FreeVariable(..) => self,
//            BoundVariable(..) => self,
//            Lambda(term) => Lambda(term.desuspend()),
//            Apply(term1, term2) => Apply(term1.desuspend(), term2.desuspend()),
//            Suspension(suspension) => suspension.step().desuspend(),
//        }
//    }
//}

//impl Term {
//    pub fn desuspend(mut self) -> Term {
//        let data = self.take();
//        let data = data.desuspend();
//        self.replace(data);
//        self
//    }
//}
