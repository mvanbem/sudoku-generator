use crate::formula_builder::{GateFormulaBuilder, Literal};

pub trait ArithmeticFormulaBuilder: GateFormulaBuilder {
    fn add_half_adder_constraint(&mut self, a: Literal, b: Literal, sum: Literal, carry: Literal) {
        self.add_logical_xor_constraint(sum, a, b);
        self.add_logical_and_constraint(carry, &[a, b]);
    }

    fn add_full_adder_constraint(
        &mut self,
        a: Literal,
        b: Literal,
        c: Literal,
        sum: Literal,
        carry: Literal,
    ) {
        // c --------------->[a  HA  s]---------------> sum
        // a -->[a  HA  s]-->[b      c]-->[b  OR  c]--> carry
        // b -->[b      c]--------------->[a       ]
        let half_adder_1_sum = self.new_variable().as_positive();
        let half_adder_1_carry = self.new_variable().as_positive();
        let half_adder_2_carry = self.new_variable().as_positive();
        self.add_half_adder_constraint(a, b, half_adder_1_sum, half_adder_1_carry);
        self.add_half_adder_constraint(c, half_adder_1_sum, sum, half_adder_2_carry);
        self.add_logical_or_constraint(carry, &[half_adder_1_carry, half_adder_2_carry]);
    }
}

impl<T> ArithmeticFormulaBuilder for T where T: GateFormulaBuilder {}
