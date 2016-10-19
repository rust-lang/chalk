use chalk_parse::ast;

pub struct Environment {
    bound_names: Vec<Option<ast::Variable>>,
    next_wildcard: Option<usize>,
}

impl Environment {
    pub fn new() -> Self {
        Environment { bound_names: vec![], next_wildcard: None }
    }

    pub fn push(&mut self, v: ast::Variable) {
        self.bound_names.push(Some(v));
    }

    pub fn pop(&mut self) {
        self.bound_names.pop();
    }

    pub fn lookup(&self, name: ast::Variable) -> Option<usize> {
        self.bound_names
            .iter()
            .rev()
            .position(|&x| x == Some(name))
    }

    pub fn claim_wildcard(&mut self) -> usize {
        match self.next_wildcard {
            Some(ref mut n) => {
                assert!(*n > 0, "incorrectly calculated number of wildcards");
                *n -= 1;
                *n
            }
            None => {
                panic!("did not expect wildcards")
            }
        }
    }
}

