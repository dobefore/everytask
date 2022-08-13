use std::borrow::Borrow;
use std::error::Error;
use std::fmt;
use std::io;
use std::num;
use std::result;
pub type Result<T> = result::Result<T, TaskError>;
#[derive(Debug)]
pub struct TaskError {
    kind: String,
    message: String,
}
#[derive(Debug, Clone)]
pub enum CustomError {
    /// error parse string to it
    ParsePayItemError(String),
    ValueEmpty(String),
    ValueNotFound(String),
}
impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ParsePayItemError(v) => {
                write!(f, "{}", v)
            }
            Self::ValueEmpty(v) => {
                write!(f, "{}", v)
            }
            Self::ValueNotFound(v) => {
                write!(f, "{}", v)
            }
        }
    }
}
/// fix :doesn't satisfy `TaskError: std::error::Error`
impl Error for TaskError {}
impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.kind, self.message)
    }
}
impl From<CustomError> for TaskError {
    fn from(error: CustomError) -> Self {
        match error {
            CustomError::ParsePayItemError(_) => TaskError {
                kind: String::from("parsepayitem"),
                message: error.to_string(),
            },
            CustomError::ValueEmpty(_) => TaskError {
                kind: String::from("value empty"),
                message: error.to_string(),
            },
            CustomError::ValueNotFound(_) => TaskError {
                kind: String::from("value not found"),
                message: error.to_string(),
            },
        }
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
impl From<num::ParseFloatError> for TaskError {
    fn from(error: num::ParseFloatError) -> Self {
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
