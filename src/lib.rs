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
        run_test(&test, &mut summary)?;
    }

    report_summary(&summary);

    match summary.failed_count {
        0 => Ok(TestState::Passed),
        _ => Ok(TestState::Failed),
    }
}

fn parse(filename: &str) -> Result<Vec<Test>, Box<dyn Error>> {
    let contents = fs::read_to_string(filename)?;
    let tests: Vec<Test> = serde_yaml::from_str(&contents)?;

    Ok(tests)
}

fn run_test(test: &Test, summary: &mut Summary) -> Result<(), Box<dyn Error>> {
    let output = Command::new("bash")
        .arg("-c")
        .arg(&test.input)
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;

    let did_pass = stdout.eq(&test.out);

    report_test(&test, did_pass);

    if did_pass {
        summary.passed_count += 1;
    } else {
        summary.failed_count += 1;
    }

    Ok(())
}

fn report_test(_test: &Test, did_pass: bool) {
    if did_pass {
        println!("✅ Pass");
    } else {
        println!("❌ Fail");
    }
}

fn report_summary(summary: &Summary) {
    println!("{:?}", summary);
}
