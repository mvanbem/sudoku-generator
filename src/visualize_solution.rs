use std::collections::HashMap;
use std::fmt::Write;
use std::process::exit;

use anyhow::Result;

use crate::formula_builder::Variable;
use crate::sat_solver::Solution;
use crate::sudoku::{Cell, Col, Digit, Row, VariableKind};

pub async fn visualize_solution(
    variables: &HashMap<VariableKind, Variable>,
    solution: &Solution,
) -> Result<()> {
    let assignments = match solution {
        Solution::Satisfiable { assignments } => assignments,
        Solution::Unsatisfiable => {
            println!("UNSATISFIABLE");
            exit(1);
        }
    };

    let mut digits: HashMap<Cell, Digit> = Default::default();
    for cell in Cell::values() {
        for digit in Digit::values() {
            if assignments[&variables[&VariableKind::Placed {
                row: cell.row,
                col: cell.col,
                digit,
            }]] {
                digits.insert(cell, digit);
            }
        }
    }

    let mut given: HashMap<Cell, bool> = Default::default();
    for cell in Cell::values() {
        given.insert(
            cell,
            assignments[&variables[&VariableKind::Given {
                row: cell.row,
                col: cell.col,
            }]],
        );
    }

    const BORDER: &str = "+-------+-------+-------+";
    for row in Row::values() {
        if row.index() % 3 == 0 {
            println!("{}", BORDER);
        }
        let mut line = "| ".to_string();
        for col in Col::values() {
            let cell = Cell { row, col };
            if col.index() > 0 {
                if col.index() % 3 == 0 {
                    line += " | ";
                } else {
                    line += " ";
                }
            }
            if given[&cell] {
                write!(&mut line, "{}", digits[&cell].as_u8())?;
            } else {
                line += " ";
            }
        }
        line += " |";
        println!("{}", line);
    }
    println!("{}", BORDER);

    Ok(())
}
