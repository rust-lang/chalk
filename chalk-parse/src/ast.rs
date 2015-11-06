pub struct Program {
    items: Vec<Item>,
}

pub enum Item {
    Fact(Application),
    Rule(Rule),
}

pub struct Rule {
    consequence: Application,
    condition: Fact,
}

pub struct Fact {
    data: Box<FactData>
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
    bits: Vec<Bit>
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
    id: u32
}

// `foo` or `bar`
pub struct Atom {
    id: u32
}

// `Foo` or `Bar`
pub struct Variable {
    id: u32
}
