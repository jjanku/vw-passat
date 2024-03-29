use std::{
    cmp::Ordering,
    iter::Peekable,
    ops::{Index, IndexMut},
};

use crate::types::{Clause, Lit, Problem, Solution};

type Var = usize;

// TODO: move to impl Lit?
fn to_var(lit: Lit) -> Var {
    assert_ne!(lit, 0);
    lit.unsigned_abs() as Var
}

#[derive(Clone, Copy, Debug)]
enum Reason {
    Decision,
    Propagation { i_clause: usize },
}

#[derive(Clone)]
struct VarData {
    value: bool,
    level: usize,
    reason: Reason,
}

struct Assignment {
    data: Vec<Option<VarData>>,
    trail: Vec<Lit>,
    levels: Vec<usize>,
}

impl Assignment {
    fn new(var_count: usize) -> Self {
        Self {
            data: vec![None; var_count + 1],
            trail: vec![],
            levels: vec![],
        }
    }

    fn eval(&self, lit: Lit) -> Option<bool> {
        self.data[to_var(lit)]
            .as_ref()
            .map(|data| data.value == lit.is_positive())
    }

    fn set(&mut self, lit: Lit, reason: Reason) {
        self.trail.push(lit);

        if let Reason::Decision = reason {
            self.levels.push(self.trail.len() - 1);
        }

        let data = VarData {
            value: lit.is_positive(),
            level: self.last_level(),
            reason,
        };
        self.data[to_var(lit)] = Some(data);
    }

    fn level(&self, lit: Lit) -> Option<usize> {
        self.data[to_var(lit)].as_ref().map(|data| data.level)
    }

    fn reason(&self, lit: Lit) -> Option<Reason> {
        self.data[to_var(lit)].as_ref().map(|data| data.reason)
    }

    fn last_level(&self) -> usize {
        self.levels.len()
    }

    /// Revert all changes at `level` (incl.) and above.
    fn backtrack(&mut self, level: usize) {
        self.levels.drain(level..);
        let i = self.levels.pop().unwrap_or(0);
        for lit in self.trail.drain(i..) {
            self.data[to_var(lit)] = None;
        }
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

#[derive(Clone, Copy, PartialEq)]
struct OrdF64(f64);

impl Eq for OrdF64 {}

impl PartialOrd for OrdF64 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrdF64 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}

impl OrdF64 {
    fn new(value: f64) -> Self {
        assert!(!value.is_nan());
        Self(value)
    }
}

struct VarHeap<T> {
    heap: Vec<(T, Var)>,
    index: Vec<usize>,
    size: usize,
}

impl<T: Ord + Copy> VarHeap<T> {
    fn new(var_count: usize, default: T) -> Self {
        let mut heap = vec![];
        // 0 inserted for indexing by variables
        let mut index = vec![0];

        for var in 1..=var_count {
            index.push(heap.len());
            heap.push((default, var));
        }

        let size = var_count;

        Self { heap, index, size }
    }

    fn swap(&mut self, i: usize, j: usize) {
        self.heap.swap(i, j);
        self.index.swap(self.heap[i].1, self.heap[j].1);
    }

    fn sift_up(&mut self, mut pos: usize) {
        while pos > 0 {
            let parent = (pos - 1) / 2;
            if self.heap[pos].0 <= self.heap[parent].0 {
                break;
            }
            self.swap(pos, parent);
            pos = parent;
        }
    }

    fn sift_down(&mut self, mut pos: usize) {
        loop {
            let mut max = pos;
            let left = 2 * pos + 1;
            if left < self.size && self.heap[left].0 > self.heap[max].0 {
                max = left;
            }
            let right = left + 1;
            if right < self.size && self.heap[right].0 > self.heap[max].0 {
                max = right;
            }

            if max != pos {
                self.swap(pos, max);
                pos = max;
            } else {
                break;
            }
        }
    }

    fn set(&mut self, var: Var, val: T) {
        let pos = self.index[var];
        self.heap[pos] = (val, var);

        self.sift_up(pos);
        self.sift_down(pos);
    }

    fn get(&self, var: Var) -> T {
        self.heap[self.index[var]].0
    }

    /// Applies `f` to every value in the heap.
    /// The supplied function must preserve the ordering of the items!
    fn transform(&mut self, mut f: impl FnMut(T) -> T) {
        for (val, _) in &mut self.heap {
            *val = f(*val);
        }
    }

    fn max(&self) -> Option<Var> {
        if self.size != 0 {
            Some(self.heap[0].1)
        } else {
            None
        }
    }

    fn extract(&mut self) -> Option<Var> {
        if self.size != 0 {
            let var = self.heap[0].1;
            self.swap(0, self.size - 1);
            self.size -= 1;
            self.sift_down(0);
            Some(var)
        } else {
            None
        }
    }

    fn restore(&mut self) -> Option<Var> {
        if self.size != self.heap.len() {
            self.size += 1;
            let var = self.heap[self.size - 1].1;
            self.sift_up(self.size - 1);
            Some(var)
        } else {
            None
        }
    }
}

struct Chooser {
    k: f64,
    seen: Vec<bool>,
    heap: VarHeap<OrdF64>,
}

impl Chooser {
    fn new(var_count: usize) -> Self {
        Self {
            k: 1.0,
            seen: vec![false; var_count + 1],
            heap: VarHeap::new(var_count, OrdF64::new(0.0)),
        }
    }

    fn touch(&mut self, var: Var) {
        if !self.seen[var] {
            self.seen[var] = true;

            let val = self.heap.get(var);
            self.heap.set(var, OrdF64::new(val.0 + self.k));
        }
    }

    fn rescale(&mut self) {
        self.k *= 1.01;

        const THRESHOLD: f64 = 10e100;
        if self.k > THRESHOLD {
            self.heap
                .transform(|OrdF64(val)| OrdF64::new(val / THRESHOLD));
            self.k /= THRESHOLD;
        }

        for var_seen in &mut self.seen {
            *var_seen = false;
        }
    }

    fn choose(&mut self, assignment: &Assignment) -> Option<Var> {
        let mut res = None;

        while let Some(var) = self.heap.max() {
            if assignment.eval(var as Lit).is_none() {
                res = Some(var);
                break;
            }
            self.heap.extract();
        }
        while self.heap.restore().is_some() {}

        res
    }
}

struct Luby {
    base: usize,
    uv: (isize, isize),
}

impl Luby {
    fn new(base: usize) -> Self {
        Self { base, uv: (1, 1) }
    }
}

impl Iterator for Luby {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let (u, v) = self.uv;
        // Based on Knuth's formula, see https://oeis.org/A182105.
        self.uv = if u & -u == v { (u + 1, 1) } else { (u, 2 * v) };
        Some(self.base * v as usize)
    }
}

pub struct Solver {
    clauses: Vec<Clause>,

    assignment: Assignment,

    watched: LitMap<Vec<usize>>,
    prop_head: usize,

    chooser: Chooser,

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
            chooser: Chooser::new(var_count),
            conflicts: 0,
            restart_threshold: Luby::new(16).peekable(),
        };

        for clause in clauses {
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
        while let Some(lit) = self.assignment.trail.get(self.prop_head) {
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

        let mut i_trail = self.assignment.trail.len();
        let i_assert = loop {
            // FIXME: which vars should be touched?
            for &lit in &learnt {
                self.chooser.touch(to_var(lit));
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
            let on_lit = self.assignment.trail[i_trail];

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

        self.chooser.rescale();

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

        while let Some(var) = self.chooser.choose(&self.assignment) {
            self.assignment.set(-(var as Lit), Reason::Decision);

            while let Some(i_conflict) = self.propagate() {
                self.conflicts += 1;

                let (learnt, level) = self.analyze(i_conflict);

                if level == 0 {
                    return Solution::Unsat;
                }
                self.assignment.backtrack(level);
                self.prop_head = std::cmp::min(self.prop_head, self.assignment.trail.len());

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
                    self.prop_head = std::cmp::min(self.prop_head, self.assignment.trail.len());
                }
            }
        }

        let model: Vec<Lit> = self.assignment.trail.clone();
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

    use super::{verify, Assignment, Luby, Reason, Solver};

    #[test]
    fn assignment() {
        let mut ass = Assignment::new(2);

        assert_eq!(ass.last_level(), 0);

        ass.set(1, Reason::Decision);
        ass.set(-2, Reason::Propagation { i_clause: 0 });

        assert_eq!(ass.last_level(), 1);
        assert_eq!(ass.level(1), Some(1));
        assert_eq!(ass.level(2), Some(1));

        ass.backtrack(1);
        assert_eq!(ass.eval(2), None);
        assert_eq!(ass.eval(1), None);
    }

    #[test]
    fn luby() {
        let expected = vec![1, 1, 2, 1, 1, 2, 4, 1, 1, 2, 1, 1, 2, 4, 8, 1, 1, 2, 1, 1];
        let actual: Vec<usize> = Luby::new(1).take(20).collect();
        assert_eq!(expected, actual);
    }

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
