use crate::formula_builder::{FormulaBuilder, Literal};

pub trait CardinalityFormulaBuilder: FormulaBuilder {
    fn add_at_most_one_of_constraint(&mut self, literals: &[Literal]) {
        for (i, a) in literals[..literals.len() - 1].iter().copied().enumerate() {
            for b in literals[i + 1..].iter().copied() {
                self.add_binary_clause(-a, -b);
            }
        }
    }
}

impl<T> CardinalityFormulaBuilder for T where T: FormulaBuilder {}
