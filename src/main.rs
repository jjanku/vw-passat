mod io;
mod types;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: vw-passat INPUT");
        return;
    }
    let mut input = std::fs::File::open(&args[1]).unwrap();

    let problem = io::read_problem(&mut input);
}
