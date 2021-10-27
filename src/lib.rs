use std::fmt;
use std::fs;
use std::io;
use std::process::Command;
use std::string;

use ansi_term::{Colour, Style};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
struct Test {
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
    failed_expectations: Vec<FailedExpectation>,
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

struct Expectation<T> {
    expected: T,
    actual: T,
}

enum FailedExpectation {
    StdOut(Expectation<String>),
    StdErr(Expectation<String>),
    ExitCode(Expectation<i32>),
}

impl fmt::Display for FailedExpectation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FailedExpectation::StdOut(ref expectation) => {
                write!(
                    f,
                    "    Unexpected output on stdout.\n\
                    \n\
                    \x20   Expected:\n\
                    \n\
                    \x20     {}\n\
                    \n\
                    \x20   Received:\n\
                    \n\
                    \x20     {}\n",
                    Colour::Green.paint(&expectation.expected),
                    Colour::Red.paint(&expectation.actual)
                )
            }
            FailedExpectation::StdErr(ref expectation) => {
                write!(
                    f,
                    "    Unexpected output on stderr.\n\
                    \n\
                    \x20   Expected:\n\
                    \n\
                    \x20     {}\n\
                    \n\
                    \x20   Received:\n\
                    \n\
                    \x20     {}\n",
                    Colour::Green.paint(&expectation.expected),
                    Colour::Red.paint(&expectation.actual)
                )
            }
            FailedExpectation::ExitCode(ref expectation) => {
                write!(
                    f,
                    "    Unexpected exit code.\n\
                    \n\
                    \x20   Expected: {}\n\
                    \n\
                    \x20   Received: {}\n\n",
                    Colour::Green.paint(expectation.expected.to_string()),
                    Colour::Red.paint(expectation.actual.to_string())
                )
            }
        }
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

pub enum ValidationError {
    MissingExitCode,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ValidationError::MissingExitCode => {
                write!(f, "expected output on stderr but no exit code specified.")
            }
        }
    }
}

pub enum CliError {
    Io(io::Error),
    Yaml(serde_yaml::Error),
    Utf8(string::FromUtf8Error),
    Validation(ValidationError),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: ")?;

        match *self {
            CliError::Io(ref err) => err.fmt(f),
            CliError::Yaml(ref err) => err.fmt(f),
            CliError::Utf8(ref err) => err.fmt(f),
            CliError::Validation(ref err) => {
                write!(f, "validation error: ")?;
                err.fmt(f)
            }
        }
    }
}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> CliError {
        CliError::Io(err)
    }
}

impl From<serde_yaml::Error> for CliError {
    fn from(err: serde_yaml::Error) -> CliError {
        CliError::Yaml(err)
    }
}

impl From<string::FromUtf8Error> for CliError {
    fn from(err: string::FromUtf8Error) -> CliError {
        CliError::Utf8(err)
    }
}

// TODO:
// - Add tests
// - Before/after each/all hooks
// - Verify that test names are unique
// - Verify that expectations are valid

pub fn run(filename: String) -> Result<TestState, CliError> {
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

fn parse(filename: &str) -> Result<Vec<Test>, CliError> {
    let contents = fs::read_to_string(filename)?;
    let tests: Vec<Test> = serde_yaml::from_str(&contents)?;

    Ok(tests)
}

fn run_test(
    test: Test,
    test_counts: &mut TestCounts,
    failures: &mut Vec<Failure>,
) -> Result<(), CliError> {
    let output = Command::new("bash").arg("-c").arg(&test.input).output()?;

    let mut failed_expectations: Vec<FailedExpectation> = Vec::new();

    // TODO: make this a failure for the individual test rather than causing all
    // tests to crash.
    validate_test(&test)?;

    let stdout = String::from_utf8(output.stdout)?;

    if let Some(expected_out) = test.out {
        if stdout.ne(&expected_out) {
            failed_expectations.push(FailedExpectation::StdOut(Expectation {
                actual: stdout,
                expected: expected_out,
            }))
        }
    }

    let stderr = String::from_utf8(output.stderr)?;

    // TODO: get expected stderr
    if let Some(expected_err) = test.err {
        if stderr.ne(&expected_err) {
            failed_expectations.push(FailedExpectation::StdErr(Expectation {
                actual: stderr,
                expected: expected_err,
            }))
        }
    }

    // TODO: get expected exit code
    if let Some(expected_exit) = test.exit_code {
        // TODO: remove unwrap
        let actual_exit_code = output.status.code().unwrap();

        if expected_exit != actual_exit_code {
            failed_expectations.push(FailedExpectation::ExitCode(Expectation {
                actual: actual_exit_code,
                expected: expected_exit,
            }))
        }
    }

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

fn validate_test(test: &Test) -> Result<(), CliError> {
    match test {
        Test {
            err: Some(_),
            exit_code: None,
            ..
        } => Err(CliError::Validation(ValidationError::MissingExitCode)),
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
