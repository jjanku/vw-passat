use std::fs;

use vw_passat::{io, parallel, solver};

enum Mode {
    Serial,
    Parallel,
}

fn test_dir(path: &str, sat: bool, mode: Mode) {
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        let mut file = fs::File::open(path).unwrap();

        let problem = io::read_problem(&mut file);
        let solution = match mode {
            Mode::Serial => solver::Solver::new(problem.clone()).solve(),
            Mode::Parallel => parallel::solve(problem.clone()),
        };
        assert!(solver::verify(&problem, sat, &solution));
    }
}

#[test]
fn sat_uniform_v50_c218() {
    test_dir("tests/data/uf50-218", true, Mode::Serial);
}

#[test]
fn sat_uniform_v100_c430() {
    test_dir("tests/data/uf100-430", true, Mode::Parallel);
}

#[test]
fn unsat_uniform_v50_c218() {
    test_dir("tests/data/uuf50-218", false, Mode::Serial);
}

#[test]
fn unsat_uniform_v100_c430() {
    test_dir("tests/data/uuf100-430", false, Mode::Parallel);
}
