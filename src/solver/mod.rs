mod assignment;
mod branching;
mod map;
mod restart;

use std::iter::Peekable;

use crate::{
    solver::assignment::Reason,
    types::{to_var, Clause, Lit, Problem, Solution},
};

use self::{assignment::Assignment, branching::Evsids, map::LitMap, restart::Luby};

pub struct Solver {
    clauses: Vec<Clause>,

    assignment: Assignment,

    watched: LitMap<Vec<usize>>,
    prop_head: usize,

    evsids: Evsids,

    conflicts: usize,
    restart_threshold: Peekable<Luby>,
}

impl Solver {
    pub fn new(problem: Problem) -> Self {
        let Problem { var_count, clauses } = problem;

        let mut solver = Solver {
            clauses: Vec::with_capacity(clauses.len()),
            assignment: Assignment::new(var_count),
            watched: LitMap::<Vec<usize>>::new(var_count),
            prop_head: 0,
            evsids: Evsids::new(var_count),
            conflicts: 0,
            restart_threshold: Luby::new(16).peekable(),
        };

        for mut clause in clauses {
            clause.sort();
            clause.dedup();
            solver.add(clause);
        }

        solver
    }

    fn add(&mut self, clause: Clause) -> usize {
        let i = self.clauses.len();
        if let [lit0, lit1, ..] = clause[..] {
            self.watched[lit0].push(i);
            self.watched[lit1].push(i);
        }
        self.clauses.push(clause);
        i
    }

    fn propagate(&mut self) -> Option<usize> {
        while let Some(lit) = self.assignment.trail().get(self.prop_head) {
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
                    self.assignment
                        .set(unit_lit, Reason::Propagation { i_clause: c });
                } else {
                    // conflict
                    return Some(c);
                }

                i += 1;
            }

            self.prop_head += 1;
        }

        None
    }

    fn analyze(&mut self, i_conflict: usize) -> (Clause, usize) {
        let mut learnt = self.clauses[i_conflict].clone();
        let last_level = self.assignment.last_level();

        let mut i_trail = self.assignment.trail().len();
        let i_assert = loop {
            // FIXME: which vars should be touched?
            for &lit in &learnt {
                self.evsids.touch(to_var(lit));
            }

            let mut iter = learnt
                .iter()
                .enumerate()
                .filter(|(_, &lit)| self.assignment.level(lit).unwrap() >= last_level);
            // FIXME: can it be None?
            let (i, _) = iter.next().unwrap();
            if iter.next().is_none() {
                break i;
            }

            i_trail -= 1;
            let on_lit = self.assignment.trail()[i_trail];

            let i_reason = match self.assignment.reason(on_lit).unwrap() {
                Reason::Propagation { i_clause } => i_clause,
                Reason::Decision => unreachable!(),
            };
            let reason = &self.clauses[i_reason];
            debug_assert!(reason.contains(&on_lit));

            let len_before = learnt.len();
            learnt.retain(|&lit| lit != -on_lit);
            if learnt.len() != len_before {
                // learnt contained -on_lit, finish the resolution
                learnt.extend(reason.iter().filter(|&&lit| lit != on_lit));
                // need to dedup to correctly determine #lits at a given level
                learnt.sort();
                learnt.dedup();
            }
        };

        let backtrack_level = if learnt.len() == 1 {
            if self.assignment.last_level() > 0 {
                1
            } else {
                self.assignment.eval(learnt[0]).unwrap() as usize
            }
        } else {
            learnt.swap(0, i_assert);

            let (i_max, _) = learnt[1..]
                .iter()
                .enumerate()
                .max_by_key(|(_, &lit)| self.assignment.level(lit).unwrap())
                .unwrap();
            learnt.swap(1, i_max + 1);

            self.assignment.level(learnt[1]).unwrap() + 1
        };

        self.evsids.rescale();

        (learnt, backtrack_level)
    }

    pub fn solve(&mut self) -> Solution {
        for (i, clause) in self.clauses.iter().enumerate() {
            match clause[..] {
                [] => return Solution::Unsat,
                [lit] => match self.assignment.eval(lit) {
                    None => self
                        .assignment
                        .set(lit, Reason::Propagation { i_clause: i }),
                    Some(false) => return Solution::Unsat,
                    Some(true) => (),
                },
                _ => (),
            }
        }

        if let Some(_i_conflict) = self.propagate() {
            return Solution::Unsat;
        }

        while let Some(var) = self.evsids.choose(&self.assignment) {
            self.assignment.set(-(var as Lit), Reason::Decision);

            while let Some(i_conflict) = self.propagate() {
                self.conflicts += 1;

                let (learnt, level) = self.analyze(i_conflict);

                if level == 0 {
                    return Solution::Unsat;
                }
                self.assignment.backtrack(level);
                self.prop_head = std::cmp::min(self.prop_head, self.assignment.trail().len());

                let lit_assert = learnt[0];
                let i_clause = self.add(learnt);
                self.assignment
                    .set(lit_assert, Reason::Propagation { i_clause });
            }

            if self.conflicts >= *self.restart_threshold.peek().unwrap() {
                self.conflicts = 0;
                self.restart_threshold.next();
                if self.assignment.last_level() >= 1 {
                    self.assignment.backtrack(1);
                    self.prop_head = std::cmp::min(self.prop_head, self.assignment.trail().len());
                }
            }
        }

        let model: Vec<Lit> = self.assignment.trail().to_vec();
        Solution::Sat { model }
    }
}

pub fn verify(problem: &Problem, sat: bool, solution: &Solution) -> bool {
    match solution {
        Solution::Sat { model } => {
            if sat {
                let mut sorted = model.to_vec();
                sorted.sort();
                problem
                    .clauses
                    .iter()
                    .all(|clause| clause.iter().any(|lit| sorted.binary_search(lit).is_ok()))
            } else {
                false
            }
        }
        Solution::Unsat => !sat,
        Solution::Unknown => false,
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{Clause, Problem};

    use super::{verify, Solver};

    fn check(clauses: Vec<Clause>, sat: bool) {
        let problem = Problem {
            var_count: clauses.iter().flatten().max().unwrap().abs() as usize,
            clauses,
        };

        let solution = Solver::new(problem.clone()).solve();
        assert!(verify(&problem, sat, &solution));
    }

    #[test]
    /// Formulas from the lecture.
    fn basic_sat() {
        let clauses = vec![vec![1, 2], vec![-1, 2], vec![-1, -2, 3], vec![-1, -2, -3]];
        check(clauses, true);

        let clauses = vec![
            vec![-1, -2, 3],
            vec![2, -1, 3],
            vec![1, -2, 3],
            vec![-3, 4, 5],
            vec![-3, 4, -5],
            vec![-3, -4, 5],
            vec![-3, -4, -5],
        ];
        check(clauses, true);
    }

    #[test]
    fn basic_unsat() {
        let clauses = vec![
            vec![1, 2],
            vec![-2, 3],
            vec![-2, -3],
            vec![-1, -2, -4],
            vec![-1, 2, -4],
            vec![-1, 2, 4],
        ];

        check(clauses, false);
    }

    #[test]
    /// Formulas with non-trivial propagation before the first decision.
    fn kickstart() {
        let clauses = vec![vec![1], vec![-1, 2], vec![-1, -2]];
        check(clauses, false);
    }
}
