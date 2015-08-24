use super::{Term, TermData};
use std::cell::RefCell;
use std::mem;

///////////////////////////////////////////////////////////////////////////
// Arena used to store terms. This is stored in TLS to avoid the pain of
// threading it everywhere.

#[derive(Clone)]
pub struct Arena {
    terms: Vec<Option<TermData>>,
    free_list: Vec<Term>,
}

impl Arena {
    pub fn new() -> Arena {
        Arena { terms: vec![], free_list: vec![] }
    }

    pub fn push(&mut self, data: TermData) -> Term {
        let index = self.terms.len();
        self.terms.push(Some(data));
        Term::from_index(index)
    }

    pub fn free(&mut self, term: Term) {
        assert!(self.terms[term.index()].is_none());
        self.free_list.push(term);
    }

    pub fn data(&self, term: Term) -> &TermData {
        self.terms[term.index()].as_ref().unwrap()
    }

    pub fn data_mut(&mut self, term: Term) -> &mut TermData {
        self.terms[term.index()].as_mut().unwrap()
    }

    pub fn take(&mut self, term: Term) -> TermData {
        self.terms[term.index()].take().unwrap()
    }

    pub fn replace(&mut self, term: Term, data: TermData) {
        let r = mem::replace(&mut self.terms[term.index()], Some(data));
        assert!(r.is_none());
    }

    pub fn swap<F>(&mut self, term: Term, func: F)
        where F: FnOnce(TermData) -> TermData
    {
        let data = self.take(term);
        let data = func(data);
        self.replace(term, data);
    }
}

thread_local! {
    static THE_ARENA: RefCell<Arena> = RefCell::new(Arena::new())
}

pub fn write<FUNC,R>(func: FUNC) -> R
    where FUNC: FnOnce(&mut Arena) -> R
{
    THE_ARENA.with(|t| func(&mut *t.borrow_mut()))
}

pub fn read<FUNC,R>(func: FUNC) -> R
    where FUNC: FnOnce(&Arena) -> R
{
    THE_ARENA.with(|t| func(&*t.borrow()))
}
