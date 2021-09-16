use std::collections::HashMap;
use std::fmt::Write;
use std::hash::Hash;

use anyhow::Result;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::positive_i32::PositiveI32;

pub use arithmetic::ArithmeticFormulaBuilder;
pub use bit_vector::BitVector;
pub use cardinality::CardinalityFormulaBuilder;
pub use gate::GateFormulaBuilder;
pub use literal::Literal;
pub use variable::Variable;

mod arithmetic;
mod bit_vector;
mod cardinality;
mod gate;
mod literal;
mod variable;

pub trait FormulaBuilder {
    fn new_variable(&mut self) -> Variable;
    fn add_clause(&mut self, literals: Vec<Literal>);
    fn add_unit_clause(&mut self, literal: Literal);
    fn add_binary_clause(&mut self, a: Literal, b: Literal);

    fn variable_count(&self) -> usize;
    fn clause_count(&self) -> usize;
}

struct UnitClause(Literal);

impl UnitClause {
    async fn write_dimacs_fragment<W: AsyncWrite + Unpin>(&self, w: &mut W) -> Result<()> {
        let mut buf = String::new();
        writeln!(&mut buf, "{} 0", self.0.index())?;
        w.write_all(buf.as_bytes()).await?;
        Ok(())
    }
}

struct BinaryClause([Literal; 2]);

impl BinaryClause {
    async fn write_dimacs_fragment<W: AsyncWrite + Unpin>(&self, w: &mut W) -> Result<()> {
        let mut buf = String::new();
        writeln!(&mut buf, "{} {} 0", self.0[0].index(), self.0[1].index())?;
        w.write_all(buf.as_bytes()).await?;
        Ok(())
    }
}

struct WideClause(Vec<Literal>);

impl WideClause {
    async fn write_dimacs_fragment<W: AsyncWrite + Unpin>(&self, w: &mut W) -> Result<()> {
        let mut buf = String::new();
        for (i, literal) in self.0.iter().copied().enumerate() {
            if i > 0 {
                write!(&mut buf, " ")?;
            }
            write!(&mut buf, "{}", literal.index())?;
        }
        writeln!(&mut buf, " 0")?;
        w.write_all(buf.as_bytes()).await?;
        Ok(())
    }
}

struct VariableCounter {
    highest_variable_index: u32,
}

impl VariableCounter {
    fn new_variable(&mut self) -> Variable {
        self.highest_variable_index += 1;
        Variable::from_index(PositiveI32::from_u32(self.highest_variable_index).unwrap())
    }
}

pub struct TaggedVariableFormulaBuilder<T> {
    variable_counter: VariableCounter,
    tagged_variables: HashMap<T, Variable>,
    unit: Vec<UnitClause>,
    binary: Vec<BinaryClause>,
    wide: Vec<WideClause>,
}

impl<T> TaggedVariableFormulaBuilder<T> {
    pub fn new() -> Self {
        Self {
            variable_counter: VariableCounter {
                highest_variable_index: 0,
            },
            tagged_variables: Default::default(),
            unit: Default::default(),
            binary: Default::default(),
            wide: Default::default(),
        }
    }

    pub fn tagged_variables(&self) -> &HashMap<T, Variable> {
        &self.tagged_variables
    }

    pub fn into_tagged_variables(self) -> HashMap<T, Variable> {
        self.tagged_variables
    }

    pub async fn write_dimacs<W: AsyncWrite + Unpin>(&self, w: &mut W) -> Result<()> {
        let mut buf = String::new();
        writeln!(
            &mut buf,
            "p cnf {} {}",
            self.variable_counter.highest_variable_index,
            self.clause_count()
        )?;
        w.write_all(buf.as_bytes()).await?;

        for clause in &self.unit {
            clause.write_dimacs_fragment(w).await?;
        }
        for clause in &self.binary {
            clause.write_dimacs_fragment(w).await?;
        }
        for clause in &self.wide {
            clause.write_dimacs_fragment(w).await?;
        }
        Ok(())
    }
}

impl<T> TaggedVariableFormulaBuilder<T>
where
    T: Eq + Hash,
{
    pub fn get_variable(&mut self, tag: T) -> Variable {
        let variable_counter = &mut self.variable_counter;
        *self
            .tagged_variables
            .entry(tag)
            .or_insert_with(move || variable_counter.new_variable())
    }
}

impl<T> FormulaBuilder for TaggedVariableFormulaBuilder<T> {
    fn new_variable(&mut self) -> Variable {
        self.variable_counter.new_variable()
    }

    fn add_clause(&mut self, literals: Vec<Literal>) {
        match &*literals {
            [] => panic!(),
            &[a] => self.add_unit_clause(a),
            &[a, b] => self.add_binary_clause(a, b),
            _ => self.wide.push(WideClause(literals)),
        }
    }

    fn add_unit_clause(&mut self, literal: Literal) {
        self.unit.push(UnitClause(literal));
    }

    fn add_binary_clause(&mut self, a: Literal, b: Literal) {
        self.binary.push(BinaryClause([a, b]));
    }

    fn variable_count(&self) -> usize {
        self.variable_counter.highest_variable_index as usize
    }

    fn clause_count(&self) -> usize {
        self.unit.len() + self.binary.len() + self.wide.len()
    }
}

impl<T> Default for TaggedVariableFormulaBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}
