use super::*;

impl Item {
    pub fn for_each_free_variable(&self, func: &mut FnMut(Span, Variable))
    {
        match *self {
            Item::Fact(ref appl) => appl.for_each_free_variable(func),
            Item::Rule(ref rule) => rule.for_each_free_variable(func),
        }
    }
}

impl Rule {
    pub fn for_each_free_variable(&self, func: &mut FnMut(Span, Variable))
    {
        self.consequence.for_each_free_variable(func);
        self.condition.for_each_free_variable(func);
    }
}

impl Application {
    pub fn values<'a>(&'a self) -> impl Iterator<Item=&'a Value> + 'a {
        self.bits
            .iter()
            .filter_map(|bit| match bit.kind {
                BitKind::Value(ref v) => Some(v),
                BitKind::Operator(..) => None,
            })
    }

    pub fn count_wildcards(&self) -> usize {
        self.values()
            .map(|value| match value.kind {
                ValueKind::Atom(_) => 0,
                ValueKind::Variable(_) => 0,
                ValueKind::Application(ref appl) => appl.count_wildcards(),
                ValueKind::Wildcard => 1,
            })
            .sum()
    }

    pub fn for_each_free_variable(&self, func: &mut FnMut(Span, Variable))
    {
        for value in self.values() {
            match value.kind {
                ValueKind::Atom(_) => (),
                ValueKind::Variable(v) => func(value.span, v),
                ValueKind::Application(ref appl) => appl.for_each_free_variable(func),
                ValueKind::Wildcard => (),
            }
        }
    }
}

impl Fact {
    pub fn for_each_free_variable(&self, func: &mut FnMut(Span, Variable))
    {
        match *self.data {
            FactData::True | FactData::False => { }
            FactData::And(ref f1, ref f2) |
            FactData::Or(ref f1, ref f2) |
            FactData::Implication(ref f1, ref f2) => {
                f1.for_each_free_variable(func);
                f2.for_each_free_variable(func);
            }
            FactData::Apply(ref appl) => {
                appl.for_each_free_variable(func);
            }
            FactData::IfThenElse(ref cond, ref then, ref otherwise) => {
                cond.for_each_free_variable(func);
                then.for_each_free_variable(func);
                otherwise.for_each_free_variable(func);
            }
            FactData::Exists(bound_v, ref f) |
            FactData::ForAll(bound_v, ref f) => {
                f.for_each_free_variable(&mut |s, free_v| {
                    if free_v != bound_v {
                        func(s, free_v);
                    }
                })
            }
        }
    }
}
