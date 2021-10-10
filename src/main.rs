use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Not enough arguments");
        process::exit(1);
    }

    let filename = args[1].clone();

    match cli_test::run(filename) {
        Ok(cli_test::TestState::Passed) => (),
        Ok(cli_test::TestState::Failed) => process::exit(1),
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

