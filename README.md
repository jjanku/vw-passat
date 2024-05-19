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
- Simple benchmarking utility & analysis script ([benchmark](tests/benchmark.py), [analysis](tests/analysis.ipynb))

### Remarks

- Parallelization was an easy way to increase performance of the solver significantly (~100 LOC, see [parallel](src/parallel.rs)). Resource utilization could be improved further by assigning new work to threads that finish early.
- Forgetting was implemented mainly to reduce the number of cache misses. This effort seems to have paid off. Additionally, it opens new possibilities for optimization because it changes the hot path (previously, up to 90 % of the execution time was spent in unit propagation, this is now reduced to 50 % or less).
  - However, at the current state of the project, it largely means that bad code can no longer be excused with "this runs 1 % of the overall time" :/ For example, conflict analysis relies on sorting to deduplicate literals in clauses, the heap implementation performs unnecessary writes, etc. These suboptimalities seem to be no longer so insignificant.
- Proof checker, `drat-trim` specifically, is used in tests to reduce the probability of any subtle bugs hiding in the solver (see [integration](tests/integration.rs) and [CI setup](.gitlab-ci.yml)). Proofs are generated for both UNSAT as well as SAT instances.
- Low "restart sequence multiplier" (the number by which the Luby sequence is scaled) seems to hurt the performance on large instances despite the paper linked in the interactive  syllabus, [Evaluating CDCL Restart Schemes](https://easychair.org/publications/open/RdBL), suggesting otherwise. (But there is a chance that I misunderstood it, I just skimmed through it.)
- Remaining features were implemented for obvious reasons.
