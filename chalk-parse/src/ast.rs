use lalrpop_intern::InternedString;

pub struct Program {
    pub items: Vec<Item>,
}

pub enum Item {
    Fact(Application),
    Rule(Rule),
}

pub struct Rule {
    pub consequence: Application,
    pub condition: Fact,
}

pub struct Fact {
    pub data: Box<FactData>
}

// A Fact looks something like one of these things:
// - `a + b`
// - `a |- c : T`
// - `a |- c : T with: out`
pub enum FactData {
    And(Fact, Fact),
    Or(Fact, Fact),

    Implication(Fact, Fact), // A => B
    Lambda(Variable, Fact),
    Exists(Variable, Fact),
    ForAll(Variable, Fact),

    Apply(Application),
}

pub struct Application {
    pub bits: Vec<Bit>
}

// Component of a fact
pub enum Bit {
    Operator(Operator),
    Atom(Atom),
    Variable(Variable),
    Paren(Box<Fact>),
}

// `+`, `|-`, or `foo:`
pub struct Operator {
    pub id: InternedString
}

// `foo` or `bar`
pub struct Atom {
    pub id: InternedString
}

// `Foo` or `Bar`
pub struct Variable {
    pub id: InternedString
}
