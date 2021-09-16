use crate::formula_builder::{FormulaBuilder, Literal};

pub trait GateFormulaBuilder: FormulaBuilder {
    fn add_logical_equivalence_constraint(&mut self, a: Literal, b: Literal) {
        //  a ->  b  =>  (-a v  b)
        // -a -> -b  =>  ( a v -b)
        self.add_binary_clause(-a, b);
        self.add_binary_clause(a, -b);
    }

    fn add_logical_or_constraint(&mut self, output: Literal, inputs: &[Literal]) {
        // i0 v ... v iN = output
        // (i0 v ... v iN -> output) ^ (output -> i0 v ... v iN)
        //  [(-i0 v output) ^ ... ^ (-iN v output)] ^ (-output v i0 v ... v iN)
        let mut wide_clause = Vec::with_capacity(inputs.len() + 1);
        wide_clause.push(-output);
        for input in inputs.iter().copied() {
            self.add_binary_clause(-input, output);
            wide_clause.push(input);
        }
        self.add_clause(wide_clause);
    }

    fn add_logical_and_constraint(&mut self, output: Literal, inputs: &[Literal]) {
        // i0 ^ ... ^ iN = output
        // (i0 ^ ... ^ iN -> output) ^ (output -> i0 ^ ... ^ iN)
        // (-i0 v ... v -iN v output) ^ [(-output v i0) ^ ... ^ (-output v iN)]
        let mut wide_clause = Vec::with_capacity(inputs.len() + 1);
        for input in inputs.iter().copied() {
            self.add_binary_clause(-output, input);
            wide_clause.push(-input);
        }
        wide_clause.push(output);
        self.add_clause(wide_clause);
    }

    fn add_logical_xor_constraint(&mut self, output: Literal, a: Literal, b: Literal) {
        // -a ^ -b -> -output  =>  ( a v  b v -output)
        // -a ^  b ->  output  =>  ( a v -b v  output)
        //  a ^ -b ->  output  =>  (-a v  b v  output)
        //  a ^  b -> -output  =>  (-a v -b v -output)
        self.add_clause(vec![a, b, -output]);
        self.add_clause(vec![a, -b, output]);
        self.add_clause(vec![-a, b, output]);
        self.add_clause(vec![-a, -b, -output]);
    }
}

impl<T> GateFormulaBuilder for T where T: FormulaBuilder {}
