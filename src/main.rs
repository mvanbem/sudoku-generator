use anyhow::Result;
use clap::clap_app;
use sat_solver::SatSolver;
use tokio::io::{stdout, AsyncWriteExt, BufWriter};

use crate::emit_problem::emit_problem;
use crate::visualize_solution::visualize_solution;

mod emit_problem;
pub mod formula_builder;
mod iter_singleton;
mod positive_i32;
mod sat_solver;
pub mod sudoku;
mod visualize_solution;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = clap_app!(myapp =>
        (@arg print_formula: --print_formula "Print the SAT formula to stdout and exit")
    )
    .get_matches();

    if matches.is_present("print_formula") {
        let mut w = BufWriter::new(stdout());
        emit_problem(&mut w).await?;
        w.shutdown().await?;
        return Ok(());
    }

    let mut solver = SatSolver::start().await?;
    let variables = emit_problem(solver.input()).await?;
    let solution = solver.solve().await?;

    visualize_solution(&variables, &solution).await?;

    Ok(())
}
