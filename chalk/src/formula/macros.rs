macro_rules! formula {
    ($f:tt => $g:tt) => {
        Formula::new(FormulaData {
            kind: FormulaKind::Implication()
        })
    }
}
