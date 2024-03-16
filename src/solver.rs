use std::ops::{Index, IndexMut};

use crate::types::{Clause, Lit, Problem, Solution};

type Var = usize;

// TODO: move to impl Lit?
fn to_var(lit: Lit) -> Var {
    assert_ne!(lit, 0);
    lit.abs() as Var
}

struct Assignment(Vec<Option<bool>>);

impl Assignment {
    fn new(var_count: usize) -> Self {
        Self(vec![None; var_count + 1])
    }

    fn set(&mut self, lit: Lit) {
        self.0[to_var(lit)] = Some(lit.is_positive());
    }

    fn unset(&mut self, lit: Lit) {
        self.0[to_var(lit)] = None;
    }

    fn eval(&self, lit: Lit) -> Option<bool> {
        self.0[to_var(lit)].map(|val| val == lit.is_positive())
    }
}

struct LitMap<T>(Vec<T>);

impl<T> LitMap<T> {
    fn position(lit: Lit) -> usize {
        2 * to_var(lit) - (lit.is_negative() as usize)
    }
}

impl<T: Clone + Default> LitMap<T> {
    fn new(var_count: usize) -> Self {
        Self(vec![Default::default(); 2 * var_count + 1])
    }
}

impl<T> Index<Lit> for LitMap<T> {
    type Output = T;

    fn index(&self, index: Lit) -> &Self::Output {
        &self.0[LitMap::<T>::position(index)]
    }
}

impl<T> IndexMut<Lit> for LitMap<T> {
    fn index_mut(&mut self, index: Lit) -> &mut Self::Output {
        &mut self.0[LitMap::<T>::position(index)]
    }
}

pub struct Solver {
    clauses: Vec<Clause>,

    // TODO: move to a Tracker struct?
    assignment: Assignment,
    trail: Vec<Lit>,
    decisions: Vec<usize>,

    watched: LitMap<Vec<usize>>,
    prop_head: usize,
}

impl Solver {
    pub fn new(problem: Problem) -> Self {
        let Problem { var_count, clauses } = problem;

        let mut watched = LitMap::<Vec<usize>>::new(var_count);
        for (i, clause) in clauses.iter().enumerate() {
            if let [lit0, lit1, ..] = clause[..] {
                watched[lit0].push(i);
                watched[lit1].push(i);
            }
        }

        Solver {
            clauses,
            assignment: Assignment::new(var_count),
            trail: vec![],
            decisions: vec![],
            watched,
            prop_head: 0,
        }
    }

    fn assign(&mut self, lit: Lit) {
        self.assignment.set(lit);
        self.trail.push(lit);
    }

    fn decide(&mut self, lit: Lit) {
        self.assign(lit);
        self.decisions.push(self.trail.len() - 1);
    }

    fn backtrack(&mut self) {
        let dec = self.decisions.pop().unwrap();
        for lit in self.trail.drain(dec + 1..) {
            self.assignment.unset(lit);
        }
        let lit = self.trail[dec];
        self.trail[dec] = -lit;
        self.assignment.set(-lit);
        self.prop_head = std::cmp::min(self.prop_head, self.trail.len() - 1);
    }

    fn choose_var(&self) -> Option<Var> {
        // TODO: add iter() to Assignment?
        self.assignment
            .0
            .iter()
            .enumerate()
            .skip(1)
            .find(|(_, val)| val.is_none())
            .map(|(var, _)| var)
    }

    fn propagate(&mut self) -> bool {
        while let Some(lit) = self.trail.get(self.prop_head) {
            let lit = -lit;

            let mut i = 0;
            'clause: while i < self.watched[lit].len() {
                let c = self.watched[lit][i];
                let clause = &mut self.clauses[c];

                // Uses "implicit" watches, i.e., the two watched literals
                // are always stored at index 0 and 1. (Borrowed from ministat.)

                if clause[1] != lit {
                    clause.swap(0, 1);
                }
                debug_assert_eq!(clause[1], lit);

                for j in 0..clause.len() {
                    match self.assignment.eval(clause[j]) {
                        Some(true) => {
                            i += 1;
                            continue 'clause;
                        }
                        None if j != 0 => {
                            clause.swap(1, j);
                            // TODO: ensure distinct literals in each clause?
                            debug_assert_ne!(clause[0], clause[1]);

                            self.watched[lit].swap_remove(i);
                            debug_assert!(!self.watched[clause[1]].contains(&c));
                            self.watched[clause[1]].push(c);

                            continue 'clause;
                        }
                        _ => (),
                    }
                }

                if self.assignment.eval(clause[0]).is_none() {
                    // unit clause
                    let unit_lit = clause[0];
                    self.assign(unit_lit);
                } else {
                    // conflict
                    return false;
                }

                i += 1;
            }

            self.prop_head += 1;
        }

        true
    }

    pub fn solve(&mut self) -> Solution {
        // FIXME: decompose properly so that we can use iter()
        for i in 0..self.clauses.len() {
            match self.clauses[i][..] {
                [] => return Solution::Unsat,
                [lit] => match self.assignment.eval(lit) {
                    None => self.assign(lit),
                    Some(false) => return Solution::Unsat,
                    Some(true) => (),
                },
                _ => (),
            }
        }

        if !self.propagate() {
            return Solution::Unsat;
        }

        while let Some(var) = self.choose_var() {
            self.decide(var as Lit);

            while !self.propagate() {
                if self.decisions.is_empty() {
                    return Solution::Unsat;
                }
                self.backtrack();
            }
        }

        let model: Vec<Lit> = self.trail.clone();
        Solution::Sat { model }
    }
}

pub fn verify(clauses: &[Clause], model: &[Lit]) -> bool {
    let mut sorted = model.to_vec();
    sorted.sort();
    clauses
        .iter()
        .all(|clause| clause.iter().any(|lit| sorted.binary_search(lit).is_ok()))
}

#[cfg(test)]
mod tests {
    use crate::types::{Clause, Problem, Solution};

    use super::{verify, Solver};

    fn check(clauses: &[Clause], sat: bool) {
        let problem = Problem {
            var_count: clauses.iter().flatten().max().unwrap().abs() as usize,
            clauses: clauses.to_vec(),
        };

        match Solver::new(problem).solve() {
            Solution::Sat { model } => {
                assert!(sat);
                assert!(verify(&clauses, &model));
            }
            Solution::Unsat => assert!(!sat),
            Solution::Unknown => assert!(false),
        }
    }

    #[test]
    /// Formulas from the lecture.
    fn basic() {
        let clauses = vec![vec![1, 2], vec![-1, 2], vec![-1, -2, 3], vec![-1, -2, -3]];
        check(&clauses, true);
    }

    #[test]
    /// Formulas with non-trivial propagation before the first decision.
    fn kickstart() {
        let clauses = vec![vec![1], vec![-1, 2], vec![-1, -2]];
        check(&clauses, false);
    }
}
