use std::collections::VecDeque;
use std::ops::Range;

use crate::formula_builder::{ArithmeticFormulaBuilder, FormulaBuilder, Literal};

#[derive(Clone, Debug)]
pub struct BitVector {
    range: Range<u32>,
    bits: Vec<Literal>,
}

impl BitVector {
    pub fn range(&self) -> Range<u32> {
        self.range.clone()
    }

    pub fn bits(&self) -> &[Literal] {
        self.bits.as_slice()
    }

    pub fn len(&self) -> usize {
        self.bits.len()
    }

    pub fn from_literal(literal: Literal) -> Self {
        BitVector {
            range: 0..2,
            bits: vec![literal],
        }
    }

    pub fn add(formula: &mut impl FormulaBuilder, a: &Self, b: &Self) -> Self {
        // Compute the range of the resulting bit vector.
        let c_range = a.range.start + b.range.start..(a.range.end - 1) + (b.range.end - 1) + 1;

        // Compute the width of the resulting bit vector.
        //
        // Worked example: With an exclusive upper bound of 256, the next power of two is just
        // 256, which has 8 trailing zeros. The highest member of that range, 255, can be stored
        // in 8 bits. The nearby exclusive upper bounds values 254 and 256 result in 8 and 9,
        // respectively. 254 also needs 8 bits, while 256 needs one more bit to be represented.
        let c_len = c_range.end.next_power_of_two().trailing_zeros() as usize;

        // Add bits from `a` and `b` until `c` is wide enough.
        let mut a_bits = a.bits.iter().copied();
        let mut b_bits = b.bits.iter().copied();
        let mut c_bits = Vec::new();
        let mut prev_carry = None;
        while c_bits.len() < c_len {
            // Fetch bits from `a`, `b`, and the previous bit's carry.
            let mut bits = Vec::with_capacity(3);
            if let Some(a) = a_bits.next() {
                bits.push(a);
            }
            if let Some(b) = b_bits.next() {
                bits.push(b);
            }
            if let Some(c) = prev_carry {
                bits.push(c);
            }

            // Add those bits.
            match &*bits {
                // An empty slice will never happen because the bound imposed by `c_bits` is exact.

                // One bit doesn't require addition.
                &[x] => {
                    c_bits.push(x);
                    prev_carry = None;
                }

                // Two bits need a half adder.
                &[x, y] => {
                    let sum = formula.new_variable().as_positive();
                    let carry = formula.new_variable().as_positive();
                    c_bits.push(sum);
                    prev_carry = Some(carry);
                    formula.add_half_adder_constraint(x, y, sum, carry);
                }

                // Three bits need a full adder.
                &[x, y, z] => {
                    let sum = formula.new_variable().as_positive();
                    let carry = formula.new_variable().as_positive();
                    c_bits.push(sum);
                    prev_carry = Some(carry);
                    formula.add_full_adder_constraint(x, y, z, sum, carry);
                }
                _ => unreachable!(),
            }
        }
        BitVector {
            range: c_range,
            bits: c_bits,
        }
    }

    pub fn add_tree(formula: &mut impl FormulaBuilder, bit_vectors: Vec<Self>) -> Self {
        let mut bit_vectors: VecDeque<_> = bit_vectors.into();
        while bit_vectors.len() > 1 {
            let a = bit_vectors.pop_front().unwrap();
            let b = bit_vectors.pop_front().unwrap();
            bit_vectors.push_back(BitVector::add(formula, &a, &b));
        }
        bit_vectors.pop_front().unwrap()
    }
}
