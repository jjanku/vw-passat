use vw_passat::{io, solver};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: vw-passat INPUT");
        return;
    }
    let mut input = std::fs::File::open(&args[1]).unwrap();
    let mut output = std::io::stdout();

    let problem = io::read_problem(&mut input);
    let mut solver = solver::Solver::new(problem);
    let solution = solver.solve();
    io::write_solution(&mut output, &solution);
}
