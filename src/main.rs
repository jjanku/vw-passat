use clap::Parser;
use vw_passat::{io, parallel, solver};

#[derive(Parser)]
#[command(about = "A CDCL-based SAT solver.")]
struct Args {
    /// Path to a file in DIMACS format
    input: String,

    /// Split problem into N subproblems,
    /// defaults to # available CPUs
    #[arg(short, long, value_name = "N")]
    jobs: Option<usize>,

    /// Generate a DRAT proof
    #[arg(short, long, value_name = "PATH")]
    proof: Option<String>,
    /// Format of the generated proof
    #[arg(long, value_name = "FORMAT", value_enum, default_value_t = io::drat::Format::Plain)]
    pformat: io::drat::Format,
}

fn main() {
    let args = Args::parse();

    assert!(
        !(args.proof.is_some() && args.jobs.unwrap_or_default() > 1),
        "incompatible options, proof generation cannot be split"
    );

    let mut input = std::fs::File::open(args.input).unwrap();
    let mut output = std::io::stdout();

    let problem = io::read_problem(&mut input);
    match args.proof {
        None => {
            let solution = parallel::solve(problem, args.jobs);
            io::write_solution(&mut output, &solution);
        }
        Some(path) => {
            let mut proof = std::fs::File::create(path).unwrap();
            let mut solver = solver::Solver::with_proof(problem);
            let solution = solver.solve();
            io::drat::write_proof(&mut proof, args.pformat, solver.proof().unwrap());
            io::write_solution(&mut output, &solution);
        }
    };
}
