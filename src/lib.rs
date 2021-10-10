use std::error::Error;
use std::fs;
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

pub enum TestState {
    Passed,
    Failed,
}

// TODO:
// - Add tests
// - Add expecting stderr and error code
// - Prettier test output

pub fn run(filename: String) -> Result<TestState, Box<dyn Error>> {
    let tests = parse(&filename)?;

    let mut summary = Summary { passed_count: 0, failed_count: 0};

    for test in tests {
        let output = Command::new("bash")
            .arg("-c")
            .arg(test.input)
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;

        if stdout.eq(&test.out) {
            println!("✅ Pass");
            summary.passed_count += 1;
        } else {
            println!("❌ Fail");
            summary.failed_count += 1;
        }

    }

    println!("{:?}", summary);

    if summary.failed_count > 0 {
        return Ok(TestState::Failed);
    }

    Ok(TestState::Passed)
}

fn parse(filename: &str) -> Result<Vec<Test>, Box<dyn Error>> {
    let contents = fs::read_to_string(filename)?;
    let tests: Vec<Test> = serde_yaml::from_str(&contents)?;

    Ok(tests)
}
