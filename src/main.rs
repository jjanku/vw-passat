use vw_passat::{io, parallel};

// TODO: migrate to clap
struct Args {
    input: String,
    jobs: Option<usize>,
}

impl Args {
    const HELP: &'static str = "
A CDCL-based SAT solver.

Usage:
    vw-passat [OPTIONS] <INPUT>

Args:
    <INPUT>     Path to a file in DIMACS format

Options:
    -h, --help      Print help
    -j, --jobs <N>  Split problem into N subproblems,
                    defaults to # available CPUs
";

    fn parse() -> Self {
        let mut args = std::env::args();
        args.next();

        let mut input: Option<String> = None;
        let mut jobs: Option<usize> = None;

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
                _ => {
                    assert!(input.is_none(), "too many positional arguments");
                    input = Some(arg)
                }
            }
        }

        Self {
            input: input.expect("INPUT should be set"),
            jobs,
        }
    }
}

fn main() {
    let args = Args::parse();

    let mut input = std::fs::File::open(args.input).unwrap();
    let mut output = std::io::stdout();

    let problem = io::read_problem(&mut input);
    let solution = parallel::solve(problem, args.jobs);
    io::write_solution(&mut output, &solution);
}
