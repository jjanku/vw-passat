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
