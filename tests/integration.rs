use std::{fs, path::Path};

use vw_passat::{
    io::{self, drat},
    parallel, solver,
    types::Proof,
};

fn verify_proof(input: impl AsRef<Path>, sat: bool, proof: &Proof, format: drat::Format) -> bool {
    use std::process::{Command, Stdio};

    let mut cmd = Command::new("drat-trim");
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::null())
        .arg(input.as_ref())
        .arg(match format {
            drat::Format::Binary => "-i",
            drat::Format::Plain => "-I",
        });
    if sat {
        cmd.arg("-S");
    }

    let mut child = cmd.spawn().expect("drat-trim should be installed");

    let mut stdin = child.stdin.take().unwrap();
    drat::write_proof(&mut stdin, format, proof);
    drop(stdin);

    let status = child.wait().unwrap();
    status.success()
}

enum Mode {
    Serial,
    Parallel,
    Prover,
}

fn test_dir(path: &str, sat: bool, mode: Mode) {
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        let mut file = fs::File::open(&path).unwrap();

        let problem = io::read_problem(&mut file);
        let solution = match mode {
            Mode::Serial => solver::Solver::new(problem.clone()).solve(),
            Mode::Parallel => parallel::solve(problem.clone(), None),
            Mode::Prover => {
                let mut solver = solver::Solver::with_proof(problem.clone());
                let solution = solver.solve();
                assert!(verify_proof(
                    &path,
                    sat,
                    solver.proof().unwrap(),
                    drat::Format::Binary
                ));
                solution
            }
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

#[test]
#[ignore = "requires drat-trim and more time"]
fn prove_sat_uniform_v125_c538() {
    test_dir("tests/data/uf125-538", true, Mode::Prover);
}

#[test]
#[ignore = "requires drat-trim and more time"]
fn prove_unsat_uniform_v125_c538() {
    test_dir("tests/data/uuf125-538", false, Mode::Prover);
}
