use std::error::Error;
use std::fmt;
use std::io;
use std::num;
#[derive(Debug)]
pub struct TaskError {
    kind: String,
    message: String,
}
/// fix :doesn't satisfy `TaskError: std::error::Error`
impl Error for TaskError {}
impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.kind, self.message)
    }
}

// Implement std::convert::From for AppError; from io::Error
impl From<io::Error> for TaskError {
    fn from(error: io::Error) -> Self {
        TaskError {
            kind: String::from("io"),
            message: error.to_string(),
        }
    }
}

// Implement std::convert::From for AppError; from num::ParseIntError
impl From<num::ParseIntError> for TaskError {
    fn from(error: num::ParseIntError) -> Self {
        TaskError {
            kind: String::from("parse"),
            message: error.to_string(),
        }
    }
}

// Implement std::convert::From for AppError; from num::ParseIntError
impl From<rusqlite::Error> for TaskError {
    fn from(error: rusqlite::Error) -> Self {
        TaskError {
            kind: String::from("sqlite"),
            message: error.to_string(),
        }
    }
}
