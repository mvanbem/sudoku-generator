use std::collections::HashMap;

use anyhow::Result;
use tokio::io::AsyncWrite;

use crate::formula_builder::{
    BitVector, CardinalityFormulaBuilder, FormulaBuilder, GateFormulaBuilder,
    TaggedVariableFormulaBuilder, Variable,
};
use crate::sudoku::{Box, Cell, Col, Digit, Row, VariableKind};

pub struct Parameters {
    pub givens: usize,
    pub inference_levels: usize,
    pub allowed_inferences: Inferences,
}

pub struct Inferences {
    pub naked_single: bool,
    pub hidden_single: bool,
}

pub async fn build_formula<W: AsyncWrite + Unpin>(
    w: &mut W,
    params: &Parameters,
) -> Result<HashMap<VariableKind, Variable>> {
    let mut formula = TaggedVariableFormulaBuilder::default();

    // One digit per cell.
    for row in Row::values() {
        for col in Col::values() {
            let literals: Vec<_> = Digit::values()
                .map(|digit| {
                    formula
                        .get_variable(VariableKind::Placed { row, col, digit })
                        .as_positive()
                })
                .collect();
            formula.add_at_most_one_of_constraint(&literals);
            formula.add_clause(literals);
        }
    }

    // Each digit appears once in a row.
    for row in Row::values() {
        for digit in Digit::values() {
            let literals: Vec<_> = Col::values()
                .map(|col| {
                    formula
                        .get_variable(VariableKind::Placed { row, col, digit })
                        .as_positive()
                })
                .collect();
            formula.add_at_most_one_of_constraint(&literals);
            formula.add_clause(literals);
        }
    }

    // Each digit appears once in a column.
    for col in Col::values() {
        for digit in Digit::values() {
            let literals: Vec<_> = Row::values()
                .map(|row| {
                    formula
                        .get_variable(VariableKind::Placed { row, col, digit })
                        .as_positive()
                })
                .collect();
            formula.add_at_most_one_of_constraint(&literals);
            formula.add_clause(literals);
        }
    }

    // Each digit appears once in a box.
    for box_ in Box::values() {
        for digit in Digit::values() {
            let literals: Vec<_> = box_
                .cells()
                .map(|cell| {
                    formula
                        .get_variable(VariableKind::Placed {
                            row: cell.row,
                            col: cell.col,
                            digit,
                        })
                        .as_positive()
                })
                .collect();
            formula.add_at_most_one_of_constraint(&literals);
            formula.add_clause(literals);
        }
    }

    // Count the given digits.
    let given_bits = Cell::values()
        .map(|cell| {
            BitVector::from_literal(
                formula
                    .get_variable(VariableKind::Given {
                        row: cell.row,
                        col: cell.col,
                    })
                    .as_positive(),
            )
        })
        .collect();
    let given_count = BitVector::add_tree(&mut formula, given_bits);

    // Fix the number of given digits.
    assert_eq!(7, given_count.len());
    for bit in 0..7 {
        let mut literal = given_count.bits()[bit];
        if (params.givens >> bit) & 1 == 0 {
            literal = -literal;
        }
        formula.add_unit_clause(literal);
    }

    // At level 0, the given placements are forced and nothing is eliminated.
    for cell in Cell::values() {
        for digit in Digit::values() {
            let placed = formula
                .get_variable(VariableKind::Placed {
                    row: cell.row,
                    col: cell.col,
                    digit,
                })
                .as_positive();
            let given = formula
                .get_variable(VariableKind::Given {
                    row: cell.row,
                    col: cell.col,
                })
                .as_positive();
            let forced = formula
                .get_variable(VariableKind::Forced {
                    row: cell.row,
                    col: cell.col,
                    digit,
                    level: 0,
                })
                .as_positive();
            formula.add_logical_and_constraint(forced, &[placed, given]);

            let eliminated = formula
                .get_variable(VariableKind::Eliminated {
                    row: cell.row,
                    col: cell.col,
                    digit,
                    level: 0,
                })
                .as_positive();
            formula.add_unit_clause(-eliminated);
        }
    }

    // Model bounded iteration of forced and eliminated placements in accordance with a rule set.
    for cell in Cell::values() {
        for digit in Digit::values() {
            for level in 1..params.inference_levels {
                let prev_level = level - 1;

                // Build up lists of justifications for forcing or eliminating this placement. The
                // variables for forcing and eliminating this placement will be equated to the
                // logical OR of these justifications.
                let mut forcing_justifications = Vec::new();
                let mut eliminating_justifications = Vec::new();

                // Forced or eliminated placements propagate from the previous level.
                forcing_justifications.push(
                    formula
                        .get_variable(VariableKind::Forced {
                            row: cell.row,
                            col: cell.col,
                            digit,
                            level: prev_level,
                        })
                        .as_positive(),
                );
                eliminating_justifications.push(
                    formula
                        .get_variable(VariableKind::Eliminated {
                            row: cell.row,
                            col: cell.col,
                            digit,
                            level: prev_level,
                        })
                        .as_positive(),
                );

                // RULE: NAKED SINGLE
                //
                // This placement is forced if all other placements in its cell are eliminated on
                // the previous level.
                if params.allowed_inferences.naked_single {
                    forcing_justifications.push({
                        let mut literals = Vec::new();
                        for other_digit in Digit::values() {
                            if digit != other_digit {
                                literals.push(
                                    formula
                                        .get_variable(VariableKind::Eliminated {
                                            row: cell.row,
                                            col: cell.col,
                                            digit: other_digit,
                                            level: prev_level,
                                        })
                                        .as_positive(),
                                );
                            }
                        }
                        let justification = formula.new_variable().as_positive();
                        formula.add_logical_and_constraint(justification, &literals);
                        justification
                    });
                }

                // RULE: HIDDEN SINGLE
                //
                // This placement is forced if, within one of its houses, all other placements for
                // this digit are eliminated.
                if params.allowed_inferences.hidden_single {
                    forcing_justifications.push({
                        let mut literals = Vec::new();
                        for other_col in Col::values() {
                            if cell.col != other_col {
                                literals.push(
                                    formula
                                        .get_variable(VariableKind::Eliminated {
                                            row: cell.row,
                                            col: other_col,
                                            digit,
                                            level: prev_level,
                                        })
                                        .as_positive(),
                                );
                            }
                        }
                        let justification = formula.new_variable().as_positive();
                        formula.add_logical_and_constraint(justification, &literals);
                        justification
                    });
                    forcing_justifications.push({
                        let mut literals = Vec::new();
                        for other_row in Row::values() {
                            if cell.row != other_row {
                                literals.push(
                                    formula
                                        .get_variable(VariableKind::Eliminated {
                                            row: other_row,
                                            col: cell.col,
                                            digit,
                                            level: prev_level,
                                        })
                                        .as_positive(),
                                );
                            }
                        }
                        let justification = formula.new_variable().as_positive();
                        formula.add_logical_and_constraint(justification, &literals);
                        justification
                    });
                    forcing_justifications.push({
                        let mut literals = Vec::new();
                        for other_cell in cell.box_().cells() {
                            if cell != other_cell {
                                literals.push(
                                    formula
                                        .get_variable(VariableKind::Eliminated {
                                            row: other_cell.row,
                                            col: other_cell.col,
                                            digit,
                                            level: prev_level,
                                        })
                                        .as_positive(),
                                );
                            }
                        }
                        let justification = formula.new_variable().as_positive();
                        formula.add_logical_and_constraint(justification, &literals);
                        justification
                    });
                }

                // This placement is eliminated by any other forced placement in its cell on the
                // previous level.
                for other_digit in Digit::values() {
                    if digit != other_digit {
                        eliminating_justifications.push(
                            formula
                                .get_variable(VariableKind::Forced {
                                    row: cell.row,
                                    col: cell.col,
                                    digit: other_digit,
                                    level: prev_level,
                                })
                                .as_positive(),
                        );
                    }
                }

                // This placement is eliminated by any other forced placement it sees for the same
                // digit on the previous level.
                for other_cell in Cell::values() {
                    if cell.sees_other(other_cell) {
                        eliminating_justifications.push(
                            formula
                                .get_variable(VariableKind::Forced {
                                    row: other_cell.row,
                                    col: other_cell.col,
                                    digit,
                                    level: prev_level,
                                })
                                .as_positive(),
                        );
                    }
                }

                // Tie whether this placement is forced to the logical OR of the justifications.
                let forced = formula
                    .get_variable(VariableKind::Forced {
                        row: cell.row,
                        col: cell.col,
                        digit,
                        level,
                    })
                    .as_positive();
                formula.add_logical_or_constraint(forced, &forcing_justifications);

                // Tie whether this placement is eliminated to the logical OR of the justifications.
                let eliminated = formula
                    .get_variable(VariableKind::Eliminated {
                        row: cell.row,
                        col: cell.col,
                        digit,
                        level,
                    })
                    .as_positive();
                formula.add_logical_or_constraint(eliminated, &eliminating_justifications);
            }
        }
    }

    // The last iteration of forced and eliminated placements must match the board.
    for cell in Cell::values() {
        for digit in Digit::values() {
            let forced = formula
                .get_variable(VariableKind::Forced {
                    row: cell.row,
                    col: cell.col,
                    digit,
                    level: params.inference_levels - 1,
                })
                .as_positive();
            let eliminated = formula
                .get_variable(VariableKind::Eliminated {
                    row: cell.row,
                    col: cell.col,
                    digit,
                    level: params.inference_levels - 1,
                })
                .as_positive();
            let placed = formula
                .get_variable(VariableKind::Placed {
                    row: cell.row,
                    col: cell.col,
                    digit,
                })
                .as_positive();
            formula.add_logical_equivalence_constraint(forced, placed);
            formula.add_logical_equivalence_constraint(eliminated, -placed);
        }
    }

    formula.write_dimacs(w).await?;

    Ok(formula.into_tagged_variables())
}
