use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Not enough arguments");
        process::exit(1);
    }

    let filename = args[1].clone();

    if let Err(e) = cli_test::run(filename) {
        // How can I match on the boxed error here?
        // Is it better to return a specific error type rather than Box<dyn Error>
        // if I want to match on a specific error type?
        eprintln!("{}", e);
        process::exit(1);
    }
}

