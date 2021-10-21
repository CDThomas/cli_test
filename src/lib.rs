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
    out: String,
}

struct Failure {
    name: String,
    failed_expectations: Vec<FailedExpectation>,
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for expectation in &self.failed_expectations {
            expectation.fmt(f)?;
        }

        return Ok(());
    }
}

enum FailedExpectation {
    StdOut(Expectation),
}

struct Expectation {
    expected: String,
    actual: String,
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
        }
    }
}

#[derive(Debug)]
struct Summary {
    passed_count: usize,
    failed_count: usize,
}

pub enum TestState {
    Passed,
    Failed,
}

pub enum CliError {
    Io(io::Error),
    Yaml(serde_yaml::Error),
    Utf8(string::FromUtf8Error),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CliError::Io(ref err) => err.fmt(f),
            CliError::Yaml(ref err) => err.fmt(f),
            CliError::Utf8(ref err) => err.fmt(f),
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
// - Add expecting stderr and error code
// - Before/after each/all hooks
// - Verify that test names are unique

pub fn run(filename: String) -> Result<TestState, CliError> {
    let tests = parse(&filename)?;

    let mut summary = Summary {
        passed_count: 0,
        failed_count: 0,
    };

    let mut failures: Vec<Failure> = Vec::new();

    for test in tests {
        run_test(test, &mut summary, &mut failures)?;
    }

    report_summary(&summary, &failures);

    match summary.failed_count {
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
    summary: &mut Summary,
    failures: &mut Vec<Failure>,
) -> Result<(), CliError> {
    let output = Command::new("bash").arg("-c").arg(&test.input).output()?;

    let stdout = String::from_utf8(output.stdout)?;

    let did_pass = stdout.eq(&test.out);

    report_test(&test, did_pass);

    if did_pass {
        summary.passed_count += 1;
    } else {
        failures.push(Failure {
            name: test.name,
            failed_expectations: vec![FailedExpectation::StdOut(Expectation {
                actual: stdout,
                expected: test.out,
            })],
        });

        summary.failed_count += 1;
    }

    Ok(())
}

fn report_test(_test: &Test, did_pass: bool) {
    if did_pass {
        print!("{}", Colour::Green.paint("."));
    } else {
        print!("{}", Colour::Red.paint("F"));
    }
}

fn report_summary(summary: &Summary, failures: &Vec<Failure>) {
    let label_text = Style::new().bold().paint("Tests:");
    let passed_text = Colour::Green.paint(format!("{} passed", summary.passed_count));
    let failed_text = Colour::Red.paint(format!("{} failed", summary.failed_count));
    let total_text = format!("{} total", summary.passed_count + summary.failed_count);

    print!("\n\n");

    if summary.failed_count > 0 {
        println!(
            "{} {}, {}, {}",
            label_text, passed_text, failed_text, total_text
        );

        report_failures(failures);
    } else {
        println!("{} {}, {}", label_text, passed_text, total_text);
    }
}

fn report_failures(failures: &Vec<Failure>) {
    if failures.len() == 0 {
        return;
    }

    print!("\n{}\n\n", Style::new().bold().paint("Failures:"));

    for (i, failure) in failures.iter().enumerate() {
        report_failure(i + 1, &failure);
    }
}

fn report_failure(i: usize, failure: &Failure) {
    print!("  {}) {}\n\n", i, Colour::Red.paint(&failure.name));
    print!("{}", failure);
}
