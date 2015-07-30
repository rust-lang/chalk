use super::*;

use std::rc::Rc;

#[cfg(test)]
mod test;

impl Environment {
    fn new() -> Environment {
        Environment {
            first_cell: None
        }
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
        let Suspension { mut term, environment, bound, lifts } = self;
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
                        match *substituted_term.data() {
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
                        }
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

            TermData::Suspension(_) => {
                panic!("nested suspensions")
            }
        }
    }
}
