use std::num::NonZeroI32;
use std::ops::Neg;

use crate::formula_builder::Variable;
use crate::positive_i32::PositiveI32;

#[derive(Clone, Copy, Debug)]
pub struct Literal(NonZeroI32);

impl Literal {
    pub const fn new(variable: Variable, polarity: bool) -> Self {
        if polarity {
            Self(variable.index().as_non_zero_i32())
        } else {
            Self(variable.index().negated())
        }
    }

    pub const fn from_index(index: NonZeroI32) -> Option<Self> {
        if index.get() == i32::MIN {
            None
        } else {
            Some(Self(index))
        }
    }

    pub const fn index(self) -> NonZeroI32 {
        self.0
    }

    pub fn variable(self) -> Variable {
        Variable::from_index(PositiveI32::from_i32(self.0.get().abs()).unwrap())
    }

    pub const fn is_positive(self) -> bool {
        self.0.get() > 0
    }

    pub const fn negated(self) -> Self {
        let result = -self.0.get();
        // SAFETY: `result` is nonzero and is not i32::MIN.
        Self(unsafe { NonZeroI32::new_unchecked(result) })
    }
}

impl Neg for Literal {
    type Output = Self;

    fn neg(self) -> Self {
        self.negated()
    }
}
