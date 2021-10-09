use std::env;
use std::fs;
use std::process;
use std::process::Command;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Test {
    #[serde(alias = "in")]
    input: String,
    out: String,
    test: String,
}

#[derive(Debug)]
struct Summary {
    passed_count: u32,
    failed_count: u32,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Not enough arguments");
        process::exit(1);
    }

    let filename = args[1].clone();

    let contents = fs::read_to_string(filename).unwrap();

    let tests: Vec<Test> =
        serde_yaml::from_str(&contents).expect("YAML was not well-formatted");

    let mut summary = Summary { passed_count: 0, failed_count: 0};

    for test in tests {
        let output = Command::new("bash")
            .arg("-c")
            .arg(test.input)
            .output()
            .unwrap();

        let stdout = String::from_utf8(output.stdout).unwrap();

        if stdout.eq(&test.out) {
            println!("Pass ✅");
            summary.passed_count += 1;
        } else {
            println!("Fail ❌");
            summary.failed_count += 1;
        }

    }

    println!("{:?}", summary);

    if summary.failed_count > 0 {
        process::exit(1);
    }
}
