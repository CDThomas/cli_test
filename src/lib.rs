use std::fmt;
use std::fs;
use std::process::Command;

use ansi_term::{Colour, Style};
use serde::Deserialize;

mod errors;
mod expectations;

#[derive(Clone, Debug, Deserialize)]
pub struct Test {
    #[serde(rename = "test")]
    name: String,
    #[serde(rename = "in")]
    input: String,
    out: Option<String>,
    err: Option<String>,
    exit_code: Option<i32>,
}

struct Failure {
    name: String,
    failure_number: usize,
    failed_expectations: Vec<expectations::FailedExpectation>,
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "  {}) {}\n\n",
            self.failure_number,
            Colour::Red.paint(&self.name)
        )?;

        for expectation in &self.failed_expectations {
            expectation.fmt(f)?;
        }

        return Ok(());
    }
}

#[derive(Debug)]
struct TestCounts {
    passed: usize,
    failed: usize,
}

impl fmt::Display for TestCounts {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let label_text = Style::new().bold().paint("Tests:");
        let passed_text = Colour::Green.paint(format!("{} passed", self.passed));
        let failed_text = Colour::Red.paint(format!("{} failed", self.failed));
        let total_text = format!("{} total", self.passed + self.failed);

        if self.failed > 0 {
            writeln!(
                f,
                "{} {}, {}, {}",
                label_text, passed_text, failed_text, total_text
            )
        } else {
            writeln!(f, "{} {}, {}", label_text, passed_text, total_text)
        }
    }
}

pub enum TestState {
    Passed,
    Failed,
}

// TODO:
// - Add tests
// - Before/after each/all hooks
// - Verify that test names are unique
// - Verify that expectations are valid

pub fn run(filename: String) -> Result<TestState, errors::CliError> {
    let tests = parse(&filename)?;

    // TODO: validate that test names are unique

    let mut test_counts = TestCounts {
        passed: 0,
        failed: 0,
    };

    let mut failures: Vec<Failure> = Vec::new();

    for test in tests {
        run_test(test, &mut test_counts, &mut failures)?;
    }

    report_summary(&test_counts, &failures);

    match test_counts.failed {
        0 => Ok(TestState::Passed),
        _ => Ok(TestState::Failed),
    }
}

fn parse(filename: &str) -> Result<Vec<Test>, errors::CliError> {
    let contents = fs::read_to_string(filename)?;
    let tests: Vec<Test> = serde_yaml::from_str(&contents)?;

    Ok(tests)
}

fn run_test(
    test: Test,
    test_counts: &mut TestCounts,
    failures: &mut Vec<Failure>,
) -> Result<(), errors::CliError> {
    let output = Command::new("bash").arg("-c").arg(&test.input).output()?;

    // TODO: make this a failure for the individual test rather than causing all
    // tests to crash.
    validate_test(&test)?;

    let failed_expectations = expectations::verify_expectations(&test, output)?;

    if failed_expectations.len() == 0 {
        report_test_passed();
        test_counts.passed += 1;
    } else {
        report_test_failed();
        test_counts.failed += 1;

        failures.push(Failure {
            name: test.name,
            failure_number: test_counts.failed,
            failed_expectations,
        });
    }

    Ok(())
}

// TODO: move this into verify_exit_code instead
fn validate_test(test: &Test) -> Result<(), errors::CliError> {
    match test {
        Test {
            err: Some(_),
            exit_code: None,
            ..
        } => Err(errors::CliError::Validation(
            errors::ValidationError::MissingExitCode,
        )),
        _ => Ok(()),
    }
}

fn report_test_passed() {
    print!("{}", Colour::Green.paint("."));
}

fn report_test_failed() {
    print!("{}", Colour::Red.paint("F"));
}

fn report_summary(test_counts: &TestCounts, failures: &Vec<Failure>) {
    print!("\n\n{}", test_counts);

    if failures.len() > 0 {
        print!("\n{}\n\n", Style::new().bold().paint("Failures:"));

        for failure in failures.iter() {
            print!("{}", failure);
        }
    }
}
