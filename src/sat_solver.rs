use std::collections::HashMap;
use std::env::{split_paths, var_os};
use std::num::NonZeroI32;
use std::path::PathBuf;
use std::process::Stdio;

use anyhow::{anyhow, Context, Result};
use tokio::fs::metadata;
use tokio::io::{stdout, AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::spawn;
use tokio::task::JoinHandle;

use crate::formula_builder::{Literal, Variable};
use crate::iter_singleton::IteratorExt;

async fn find_file_on_path(name: &str) -> Result<PathBuf> {
    let path = var_os("PATH").ok_or_else(|| anyhow!("PATH not defined in the environment"))?;

    for mut path in split_paths(&path) {
        path.push(name);
        if let Ok(metadata) = metadata(&path).await {
            if metadata.is_file() {
                return Ok(path);
            }
        }
    }

    Err(anyhow!("{} was not found on the PATH", name))
}

async fn parse_output(child_stdout: ChildStdout) -> Result<Solution> {
    // TODO: Wait a few seconds before echoing messages to stdout. That will eliminiate spam for
    // quick solves while providing a stream of status updates during long solves.
    let mut stdout = stdout();
    let mut solution = None;
    let mut lines = BufReader::new(child_stdout).lines();
    let mut variables_done = false;
    while let Some(line) = lines.next_line().await? {
        let mut suppress = false;
        if let Some(suffix) = line.strip_prefix('s') {
            match &*suffix
                .split_ascii_whitespace()
                .singleton()
                .unwrap()
                .to_lowercase()
            {
                "satisfiable" => {
                    if solution.is_some() {
                        return Err(anyhow!("DIMACS parse error: multiple solution lines"));
                    }
                    solution = Some(Solution::Satisfiable {
                        assignments: HashMap::new(),
                    });
                }
                "unsatisfiable" => {
                    if solution.is_some() {
                        return Err(anyhow!("DIMACS parse error: multiple solution lines"));
                    }
                    solution = Some(Solution::Unsatisfiable);
                }
                _ => {
                    return Err(anyhow!(
                        "DIMACS parse error: unsupported solution line: {:?}",
                        line
                    ));
                }
            }
        } else if let Some(suffix) = line.strip_prefix('v') {
            suppress = true;
            if let Some(Solution::Satisfiable { assignments }) = solution.as_mut() {
                for part in suffix.split_ascii_whitespace() {
                    if variables_done {
                        return Err(anyhow!(
                            "DIMACS parse error: variable assignments after the zero terminator",
                        ));
                    }
                    let literal = i32::from_str_radix(part, 10)
                        .with_context(|| anyhow!("DIMACS parse error: bad literal: {:?}", part))?;
                    if literal == 0 {
                        variables_done = true;
                    } else if let Some(literal) =
                        Literal::from_index(NonZeroI32::new(literal).unwrap())
                    {
                        assignments.insert(literal.variable(), literal.is_positive());
                    } else {
                        return Err(anyhow!(
                            "DIMACS parse error: literal out of range: {}",
                            literal,
                        ));
                    }
                }
            } else {
                return Err(anyhow!(
                    "DIMACS parse error: variable assignments before solution line",
                ));
            }
        }
        // Ignore all other line types.

        if !suppress {
            stdout.write_all(line.as_bytes()).await?;
            stdout.write(b"\n").await?;
        }
    }

    if let Some(Solution::Satisfiable { .. }) = solution.as_ref() {
        if !variables_done {
            return Err(anyhow!(
                "DIMACS parse error: variable assignments not terminated with a zero literal",
            ));
        }
    }

    Ok(solution.unwrap())
}

pub struct SatSolver {
    child: Child,
    input: BufWriter<ChildStdin>,
    solution: JoinHandle<Result<Solution>>,
}

impl SatSolver {
    pub async fn start() -> Result<Self> {
        let executable_path = find_file_on_path("kissat").await?;

        let mut child = Command::new(executable_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // .arg("-q")
            .spawn()
            .context("Failed to execute kissat")?;

        let input = BufWriter::new(child.stdin.take().unwrap());
        let output = child.stdout.take().unwrap();
        let solution = spawn(async move { parse_output(output).await });

        Ok(Self {
            child,
            input,
            solution,
        })
    }

    pub fn input(&mut self) -> &mut impl AsyncWrite {
        &mut self.input
    }

    pub async fn solve(self) -> Result<Solution> {
        let Self {
            mut child,
            mut input,
            solution,
        } = self;
        input.shutdown().await?;
        drop(input);

        let exit_status = child.wait().await?;
        let solution = solution.await??;
        match (exit_status.code(), &solution) {
            (Some(10), Solution::Satisfiable { .. }) | (Some(20), Solution::Unsatisfiable) => (),
            _ => {
                return Err(anyhow!(
                    "unexpected exit status from kissat ({}) with parsed solution {:?}",
                    exit_status,
                    solution,
                ));
            }
        }

        Ok(solution)
    }
}

#[derive(Debug)]
pub enum Solution {
    Satisfiable {
        assignments: HashMap<Variable, bool>,
    },
    Unsatisfiable,
}
