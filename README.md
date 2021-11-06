# CLI Test

A tiny test framework for CLIs.

## Usage

This project currently isn't published. To run it, you'll need to clone the repo and build it (assumes that the Rust toolchain is installed):
```
$ git clone git@github.com:CDThomas/cli_test.git
$ cd cli_test
$ cargo build --release
$ ./target/release/cli_test test.yml
```

### Test Format

Tests are specified in YAML.

Example:
```
- test: Test output on stdout
  in: echo "Hello world"
  out: |
    Hello world
- test: Test output on stderr
  in: ">&2 echo \"error\""
  err: |
    error
- test: Test exit code
  in: exit 1
  exit_code: 1
- test: Test stderr and exit code
  in: ">&2 echo \"error\" && exit 1"
  err: |
    error
  exit_code: 1
```

Test properties:
* `test` (required): the name of the test (must be unique)
* `in` (required): the command to run for the test
* `out`: output to expect on stdout (if any)
* `err`: output to expect on stderr (if any)
* `exit_code`: expected exit code

`out`, `err`, and `exit_code` are optional. These properties are ignored (and not asserted against) when omitted.

## Credits

* [shrun](https://github.com/rylandg/shrun): the CLI test runner that inspired this project
