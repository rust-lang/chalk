use lalrpop_intern::InternedString;

#[derive(Debug)]
pub struct Span {
    pub lo: usize,
    pub hi: usize,
}

impl Span {
    pub fn new(lo: usize, hi: usize) -> Self {
        Span { lo: lo, hi: hi }
    }
}

#[derive(Debug)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug)]
pub enum Item {
    Fact(Application),
    Rule(Rule),
}

#[derive(Debug)]
pub struct Rule {
    pub span: Span,
    pub consequence: Application,
    pub condition: Fact,
}

#[derive(Debug)]
pub struct Fact {
    pub span: Span,
    pub data: Box<FactData>
}

// A Fact looks something like one of these things:
// - `a + b`
// - `a |- c : T`
// - `a |- c : T with: out`
#[derive(Debug)]
pub enum FactData {
    And(Fact, Fact),
    Or(Fact, Fact),

    Implication(Fact, Fact), // A => B
    Exists(Variable, Fact), // exists(x: A)
    ForAll(Variable, Fact), // forall(x: A)

    Apply(Application),
}

#[derive(Debug)]
pub struct Application {
    pub span: Span,
    pub bits: Vec<Bit>,
}

#[derive(Debug)]
pub struct Bit {
    pub span: Span,
    pub kind: BitKind
}

#[derive(Debug)]
pub enum BitKind {
    Value(Value),
    Operator(Operator),
}

// Component of a fact
#[derive(Debug)]
pub enum Value {
    Atom(Atom),
    Variable(Variable),
    Application(Application),
    Wildcard,
}

// `+`, `|-`, or `foo:`
#[derive(Debug)]
pub enum Operator {
    Colon(InternedString),
    Parens(InternedString),
    Symbols(InternedString),
}

// `foo` or `bar`
#[derive(Debug)]
pub struct Atom {
    pub id: InternedString
}

// `Foo` or `Bar`
#[derive(Debug)]
pub struct Variable {
    pub id: InternedString
}

