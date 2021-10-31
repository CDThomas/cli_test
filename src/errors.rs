use std::fmt;
use std::io;
use std::string;

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
