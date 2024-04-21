# vw-passat

Jakub Janků (514496)

![VW meme](res/meme.gif)

## How to get it moving

First, compile the project:

```bash
cargo build --release
```

The output binary is stored in `target/release/vw-passat`. Your Passat is now ready to go!

Basic usage:

```bash
vw-passat input.cnf
```

The solution is printed to `stdout`.

For more advanced options, see `vw-passat --help`

## Implemented functionality

Required:

- Input in DIMACS format ([io](src/io/mod.rs))
- Output in SAT Competition format ([io](src/io/mod.rs))
- Unit propagation using two watched literals ([solver](src/solver/mod.rs))
- Conflict-driven clause learning (CDCL) ([solver](src/solver/mod.rs))
- EVSIDS branching heuristic ([activity](src/solver/activity.rs))
- Restarts using Luby sequence ([restart](src/solver/restart.rs))

Additional:

- Parallelization ([parallel](src/parallel.rs))
- Phase saving ([assignment](src/solver/assignment.rs))
- Clause forgetting ([activity](src/solver/activity.rs), [solver](src/solver/mod.rs))
- Basic learnt clause minimization ([solver](src/solver/mod.rs))
- DRAT proof generation (plain, binary) ([drat](src/io/drat.rs))
- Simple benchmarking utility ([benchmark](tests/benchmark.py))
