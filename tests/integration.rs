use std::fs;

use vw_passat::{io, solver};

fn test_dir(path: &str, sat: bool) {
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        let mut file = fs::File::open(path).unwrap();

        let problem = io::read_problem(&mut file);
        let solution = solver::Solver::new(problem.clone()).solve();
        assert!(solver::verify(&problem, sat, &solution));
    }
}

#[test]
fn sat_uniform_v50_c218() {
    test_dir("tests/data/uf50-218", true);
}

#[test]
fn unsat_uniform_v50_c218() {
    test_dir("tests/data/uuf50-218", false);
}
