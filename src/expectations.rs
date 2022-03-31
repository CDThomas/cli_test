use std::fmt;

use ansi_term::Colour;

use crate::errors;

pub struct Expectation<T> {
    expected: T,
    actual: T,
}

pub enum FailedExpectation {
    StdOut(Expectation<String>),
    StdErr(Expectation<String>),
    ExitCode(Expectation<i32>),
    MissingExitCode,
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
            FailedExpectation::MissingExitCode => {
                write!(f, "    No exit code received.")
            }
        }
    }
}

pub fn verify_expectations(
    test: &super::Test,
    output: std::process::Output,
) -> Result<Vec<FailedExpectation>, errors::CliError> {
    let mut failed_expectations: Vec<FailedExpectation> = Vec::new();

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    let exit_code = output.status.code();

    if let Some(failed_expectation) = verify_stdout(test, &stdout) {
        failed_expectations.push(failed_expectation);
    }

    if let Some(failed_expectation) = verify_stderr(test, &stderr) {
        failed_expectations.push(failed_expectation);
    }

    if let Some(failed_expectation) = verify_exit_code(test, exit_code) {
        failed_expectations.push(failed_expectation);
    }

    Ok(failed_expectations)
}

fn verify_stdout(test: &super::Test, stdout: &str) -> Option<FailedExpectation> {
    match &test.out {
        Some(expected_out) if stdout.ne(expected_out) => {
            Some(FailedExpectation::StdOut(Expectation {
                actual: stdout.to_string(),
                expected: expected_out.to_string(),
            }))
        }
        _ => None,
    }
}

fn verify_stderr(test: &super::Test, stderr: &str) -> Option<FailedExpectation> {
    match &test.err {
        Some(expected_err) if stderr.ne(expected_err) => {
            Some(FailedExpectation::StdErr(Expectation {
                actual: stderr.to_string(),
                expected: expected_err.to_string(),
            }))
        }
        _ => None,
    }
}

fn verify_exit_code(test: &super::Test, exit_code: Option<i32>) -> Option<FailedExpectation> {
    match (test.exit_code, exit_code) {
        (Some(expected_exit_code), Some(exit_code)) if exit_code != expected_exit_code => {
            Some(FailedExpectation::ExitCode(Expectation {
                actual: exit_code,
                expected: expected_exit_code,
            }))
        }
        (_, None) => Some(FailedExpectation::MissingExitCode),
        _ => None,
    }
}
