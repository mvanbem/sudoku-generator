use std::time::Duration;

use anyhow::{anyhow, Result};
use clap::{clap_app, ArgMatches};
use sat_solver::SatSolver;
use tokio::io::{stdout, AsyncWriteExt, BufWriter};
use tokio::time::timeout;

use crate::emit_problem::{build_formula, Inferences, Parameters};
use crate::visualize_solution::visualize_solution;

mod emit_problem;
pub mod formula_builder;
mod iter_singleton;
mod positive_i32;
mod sat_solver;
pub mod sudoku;
mod visualize_solution;

fn get_bool_arg(matches: &ArgMatches, name: &str) -> Result<Option<bool>> {
    match matches.value_of(name) {
        Some(value) => match &*value.to_lowercase() {
            "true" => Ok(Some(true)),
            "false" => Ok(Some(false)),
            x => Err(anyhow!("expected true or false in --{} {}", name, x)),
        },
        None => Ok(None),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = clap_app!(myapp =>
        (@arg givens: --givens +takes_value "Require this many givens (default 40)")
        (@arg max_inference_levels: --max_inference_levels +takes_value "Instantiate the inference circuit to this depth (default 25)")
        (@arg naked_single: --naked_single +takes_value "Allow the solution to require naked single inference (default true)")
        (@arg hidden_single: --hidden_single +takes_value "Allow the solution to require hidden single inference (default true)")
        (@arg timeout_seconds: --timeout_seconds +takes_value "Seconds to search before giving up (default unbounded)")
        (@arg print_formula: --print_formula "Print the SAT formula to stdout and exit")
    )
    .get_matches();

    let params = Parameters {
        givens: matches
            .value_of("givens")
            .map(|s| usize::from_str_radix(s, 10))
            .transpose()?
            .unwrap_or(40),
        inference_levels: {
            let value = matches
                .value_of("max_inference_levels")
                .map(|s| usize::from_str_radix(s, 10))
                .transpose()?
                .unwrap_or(25);
            if value < 1 {
                return Err(anyhow!("--max_inference_levels must be at least 1"));
            }
            value
        },
        allowed_inferences: Inferences {
            naked_single: get_bool_arg(&matches, "naked_single")?.unwrap_or(true),
            hidden_single: get_bool_arg(&matches, "hidden_single")?.unwrap_or(true),
        },
    };
    let timeout_duration = matches
        .value_of("timeout_seconds")
        .map(|s| -> Result<Duration> { Ok(Duration::from_secs(u64::from_str_radix(s, 10)?)) })
        .transpose()?;

    if matches.is_present("print_formula") {
        let mut w = BufWriter::new(stdout());
        build_formula(&mut w, &params).await?;
        w.shutdown().await?;
        return Ok(());
    }

    let mut solver = SatSolver::start().await?;
    let variables = build_formula(solver.input(), &params).await?;

    let solution = if let Some(duration) = timeout_duration {
        timeout(duration, solver.solve()).await??
    } else {
        solver.solve().await?
    };

    visualize_solution(&variables, &solution).await?;

    Ok(())
}
