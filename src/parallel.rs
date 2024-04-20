use std::{collections::BinaryHeap, iter::zip, sync::mpsc::channel, thread};

use crate::{
    solver::Solver,
    types::{to_var, Lit, Problem, Solution, Var},
};

struct FrequentVars {
    heap: BinaryHeap<(usize, Var)>,
}

impl Iterator for FrequentVars {
    type Item = Var;

    fn next(&mut self) -> Option<Self::Item> {
        self.heap.pop().map(|(_, var)| var)
    }
}

fn frequent_vars(problem: &Problem) -> FrequentVars {
    let mut var_freq: Vec<usize> = vec![0; problem.var_count + 1];

    for &lit in problem.clauses.iter().flatten() {
        var_freq[to_var(lit)] += 1;
    }

    FrequentVars {
        heap: var_freq
            .iter()
            .enumerate()
            .skip(1)
            .map(|(var, &freq)| (freq, var))
            .collect(),
    }
}

type Cube = Vec<Lit>;

fn cubes(vars: &[Var], n: usize) -> Vec<Cube> {
    assert!(n <= 2usize.pow(vars.len() as u32));

    match n {
        0 => vec![],
        1 => vec![vec![]],
        _ => {
            let m = n / 2;
            let lit = vars[0] as Lit;

            let mut res1 = cubes(&vars[1..], m);
            for cube in &mut res1 {
                cube.push(lit)
            }

            let mut res2 = cubes(&vars[1..], n - m);
            for cube in &mut res2 {
                cube.push(-lit);
            }

            res1.extend(res2);
            res1
        }
    }
}

/// Splits `problem` into `n` subproblems such that
/// the original problem is SAT iff at least one of the subproblems is.
fn split(problem: Problem, n: usize) -> Vec<Problem> {
    // TODO: any better heuristics?
    let vars: Vec<Var> = frequent_vars(&problem).take(n).collect();
    let cubes = cubes(&vars, n);
    let mut subproblems = vec![problem; n];

    for (subproblem, cube) in zip(subproblems.iter_mut(), cubes) {
        for lit in cube {
            subproblem.clauses.push(vec![lit]);
        }
    }

    subproblems
}

pub fn solve(problem: Problem, n: Option<usize>) -> Solution {
    let n = n.unwrap_or(
        thread::available_parallelism()
            .map(|val| val.get())
            .unwrap_or(2),
    );

    let subproblems = split(problem, n);

    let (tx, rx) = channel::<Solution>();

    for subproblem in subproblems {
        let thread_tx = tx.clone();
        thread::spawn(move || {
            let mut solver = Solver::new(subproblem);
            let solution = solver.solve();
            let _ = thread_tx.send(solution);
        });
    }

    // receiver blocks as long as some tranmitter is alive
    drop(tx);

    let mut solution = Solution::Unsat;
    for subsolution in rx {
        if let Solution::Sat { .. } = subsolution {
            solution = subsolution;
            break;
        }
    }
    solution
}
