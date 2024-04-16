// TODO: make Lit, Var proper structs?

pub type Lit = i32;

pub type Var = usize;

pub fn to_var(lit: Lit) -> Var {
    assert_ne!(lit, 0);
    lit.unsigned_abs() as Var
}

pub type Clause = Vec<Lit>;

#[derive(Clone)]
pub struct Problem {
    pub var_count: usize,
    pub clauses: Vec<Clause>,
}

pub enum Solution {
    Sat { model: Vec<Lit> },
    Unsat,
    Unknown,
}

#[derive(Clone, Copy)]
pub enum ProofStep {
    Add,
    Delete,
}

pub type Proof = Vec<(ProofStep, Clause)>;
