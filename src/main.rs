use vw_passat::{io, parallel, solver};

// TODO: migrate to clap
struct Args {
    input: String,
    jobs: Option<usize>,
    proof: Option<String>,
    pformat: io::drat::Format,
}

impl Args {
    const HELP: &'static str = "
A CDCL-based SAT solver.

Usage:
    vw-passat [OPTIONS] <INPUT>

Args:
    <INPUT>     Path to a file in DIMACS format

Options:
    -h, --help          Print help
    -j, --jobs <N>      Split problem into N subproblems,
                        defaults to # available CPUs

    -p, --proof <PATH>  Generate a DRAT proof
    --pformat <FORMAT>  Format of the generated proof,
                        'binary' or 'plain' (default)
";

    fn parse() -> Self {
        let mut args = std::env::args();
        args.next();

        let mut input: Option<String> = None;
        let mut jobs: Option<usize> = None;
        let mut proof: Option<String> = None;
        let mut pformat: io::drat::Format = io::drat::Format::Plain;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => {
                    println!("{}", Self::HELP);
                    std::process::exit(0);
                }
                "-j" | "--jobs" => {
                    let n = args
                        .next()
                        .expect("a number N should follow after -j and --jobs")
                        .parse()
                        .unwrap();
                    jobs = Some(n);
                }
                "-p" | "--proof" => {
                    let path = args
                        .next()
                        .expect("a path should follow after -p and --proof");
                    proof = Some(path);
                }
                "--pformat" => {
                    let format = args.next().expect("a format should be specified");
                    pformat = match format.as_str() {
                        "binary" => io::drat::Format::Binary,
                        "plain" => io::drat::Format::Plain,
                        _ => panic!("format should be 'binary' or 'plain'"),
                    }
                }
                _ => {
                    assert!(input.is_none(), "too many positional arguments");
                    input = Some(arg)
                }
            }
        }

        Self {
            input: input.expect("INPUT should be set"),
            jobs,
            proof,
            pformat,
        }
    }
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
