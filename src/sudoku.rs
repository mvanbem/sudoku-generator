#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum VariableKind {
    Placed {
        row: Row,
        col: Col,
        digit: Digit,
    },
    Given {
        row: Row,
        col: Col,
    },
    Forced {
        row: Row,
        col: Col,
        digit: Digit,
        level: usize,
    },
    Eliminated {
        row: Row,
        col: Col,
        digit: Digit,
        level: usize,
    },
}

macro_rules! bounded_integer_1_through_9 {
    ($name:ident) => {
        #[derive(::std::clone::Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name(u8);

        impl $name {
            pub const K1: Self = Self(1);
            pub const K2: Self = Self(2);
            pub const K3: Self = Self(3);
            pub const K4: Self = Self(4);
            pub const K5: Self = Self(5);
            pub const K6: Self = Self(6);
            pub const K7: Self = Self(7);
            pub const K8: Self = Self(8);
            pub const K9: Self = Self(9);

            pub fn values() -> impl Iterator<Item = Self> {
                (1..=9).into_iter().map(|x| Self(x))
            }

            pub const fn new(value: u8) -> Option<Self> {
                if value >= 1 && value <= 9 {
                    Some(Self(value))
                } else {
                    None
                }
            }

            pub const fn as_u8(self) -> u8 {
                self.0
            }

            pub const fn index(self) -> u32 {
                self.0 as u32 - 1
            }
        }
    };
}

bounded_integer_1_through_9!(Row);
bounded_integer_1_through_9!(Col);
bounded_integer_1_through_9!(Digit);

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cell {
    pub row: Row,
    pub col: Col,
}

impl Cell {
    pub fn values() -> impl Iterator<Item = Self> {
        Row::values().flat_map(|row| Col::values().map(move |col| Cell { row, col }))
    }

    pub const fn box_(self) -> Box {
        Box((self.row.0 - 1) / 3 * 3 + (self.col.0 - 1) / 3 + 1)
    }

    pub fn common_houses(self, rhs: Self) -> usize {
        (if self.row == rhs.row { 1 } else { 0 }
            + if self.col == rhs.col { 1 } else { 0 }
            + if self.box_() == rhs.box_() { 1 } else { 0 })
    }

    /// Whether this cell sees the other cell and is distinct from it. True if the cells share a
    /// row, column, and/or box and are not the same cell. False if the cells have no houses in
    /// common or are the same cell.
    ///
    /// This relation is symmetric, but not transitive or reflexive.
    pub fn sees_other(self, rhs: Self) -> bool {
        if self.row == rhs.row && self.col == rhs.col {
            // Same cell.
            false
        } else if self.row == rhs.row || self.col == rhs.col || self.box_() == rhs.box_() {
            // Different cells that share a house.
            true
        } else {
            // Unentangled cells.
            false
        }
    }
}

bounded_integer_1_through_9!(Box);

impl Box {
    pub fn rows(self) -> impl Iterator<Item = Row> {
        let base_row = (self.0 - 1) / 3 * 3 + 1;
        (base_row..base_row + 3).into_iter().map(|x| Row(x))
    }

    pub fn cols(self) -> impl Iterator<Item = Col> {
        let base_col = (self.0 - 1) % 3 * 3 + 1;
        (base_col..base_col + 3).into_iter().map(|x| Col(x))
    }

    pub fn cells(self) -> impl Iterator<Item = Cell> {
        self.rows()
            .flat_map(move |row| self.cols().map(move |col| Cell { row, col }))
    }
}

#[cfg(test)]
mod tests {
    use super::Cell;

    #[test]
    fn box_consistency() {
        for cell in Cell::values() {
            let box_ = cell.box_();
            assert_eq!(3, box_.rows().count());
            assert_eq!(3, box_.cols().count());
            assert_eq!(9, box_.cells().count());
            assert_eq!(1, box_.rows().filter(|r| *r == cell.row).count());
            assert_eq!(1, box_.cols().filter(|c| *c == cell.col).count());
            assert_eq!(1, box_.cells().filter(|c| *c == cell).count());
        }
    }
}
