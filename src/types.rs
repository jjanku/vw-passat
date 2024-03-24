pub type Lit = i32;

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
