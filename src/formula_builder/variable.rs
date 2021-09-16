use crate::positive_i32::PositiveI32;

use crate::formula_builder::Literal;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Variable(PositiveI32);

impl Variable {
    pub const fn from_index(index: PositiveI32) -> Self {
        Self(index)
    }

    pub const fn index(self) -> PositiveI32 {
        self.0
    }

    pub const fn as_literal(self, polarity: bool) -> Literal {
        Literal::new(self, polarity)
    }

    pub const fn as_positive(self) -> Literal {
        Literal::new(self, true)
    }

    pub const fn as_negative(self) -> Literal {
        Literal::new(self, false)
    }
}
