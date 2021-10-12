use std::error::Error;
use std::fs;
use std::process::Command;

use ansi_term::{Colour, Style};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
struct Test {
    #[serde(rename = "in")]
    input: String,
    out: String,
    test: String,
}

struct Failure {
    test: Test,
    messages: Vec<String>
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

// TODO:
// - Add tests
// - Add expecting stderr and error code
// - Before/after each/all hooks
// - Verify that test names are unique

pub fn run(filename: String) -> Result<TestState, Box<dyn Error>> {
    let tests = parse(&filename)?;

    let mut summary = Summary { passed_count: 0, failed_count: 0};

    let mut failures: Vec<Failure> = Vec::new();

    for test in tests {
        run_test(&test, &mut summary, &mut failures)?;
    }

    report_failures(&failures);
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

fn run_test(test: &Test, summary: &mut Summary, failures: &mut Vec<Failure>) -> Result<(), Box<dyn Error>> {
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
        let expected_text = format!("{}", Colour::Green.paint(&test.out));
        let actual_text = format!("{}", Colour::Red.paint(&stdout));

        let message = format!(
            "Unexpected output on stdout.\n\nExpected:\n\n{}\n\nReceived:\n\n{}\n",
            pad_lines(&expected_text, 2),
            pad_lines(&actual_text, 2),
        );

        failures.push(Failure { test: test.clone(), messages: vec![message]});
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

fn report_summary(summary: &Summary) {
    let label_text = Style::new().bold().paint("Tests:");
    let passed_text = Colour::Green.paint(format!("{} passed", summary.passed_count));
    let failed_text = Colour::Red.paint(format!("{} failed", summary.failed_count));
    let total_text = format!("{} total", summary.passed_count + summary.failed_count);

    println!("");

    if summary.failed_count > 0 {
        println!("{} {}, {}, {}", label_text, passed_text, failed_text, total_text);
    } else {
        println!("{} {}, {}", label_text, passed_text, total_text);
    }
}

fn report_failures(failures: &Vec<Failure>) {
    if failures.len() == 0 {
        return;
    }

    println!("\n");
    println!("{}", Style::new().bold().paint("Failures:"));
    println!("");

    for (i, failure) in failures.iter().enumerate() {
        report_failure(i + 1, &failure);
    }
}

fn report_failure(i: usize, failure: &Failure) {
    println!("  {}) {}", i, Colour::Red.paint(&failure.test.test));

    for message in &failure.messages {
        println!("\n{}", pad_lines(message, 4));
    }
}

fn pad_lines(message: &str, padding: usize) -> String {
    let mut padded: Vec<String> = Vec::new();

    for line in message.lines() {
        padded.push(format!("{}{}", " ".repeat(padding), line));
    }

    padded.join("\n")
}
