use clap::{App, Arg};
use std::process;

fn main() {
    let matches = App::new("CLI Test")
        .version("0.1.0")
        .about("A tiny test framework for CLIs")
        .arg(
            Arg::with_name("file")
                .required(true)
                .help("The test file to run"),
        )
        .get_matches();

    let filename = matches.value_of("file").unwrap();

    match cli_test::run(filename) {
        Ok(cli_test::TestState::Passed) => (),
        Ok(cli_test::TestState::Failed) => process::exit(1),
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}
