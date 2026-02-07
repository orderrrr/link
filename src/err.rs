use crate::{Rule, B, NN};
use std::fmt::{Display, Formatter};

use std::fmt::Error as FmtError;
use std::num::{ParseFloatError, ParseIntError};

use pest::error::Error as PestError;

use std::error::Error;
use std::panic::Location;

pub type VRes = Result<Vec<NN>, LocatedError<LErrEnum>>;
pub type Res = Result<NN, LocatedError<LErrEnum>>;
pub type BRes = Result<B, LocatedError<LErrEnum>>;

pub type URes = Result<u16, LocatedError<LErrEnum>>;
pub type VURes = Result<Vec<u16>, LocatedError<LErrEnum>>;

#[derive(Debug, Clone)]
pub struct LErr {
    pub error: String,
    pub start: usize,
    pub end: usize,
}

impl Error for LErr {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self)
    }
}

impl Display for LErr {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "{}, {}.{}", self.error, self.start, self.end)
    }
}

#[derive(Debug, Clone)]
pub struct LocatedError<E: Error + 'static> {
    inner: E,
    location: &'static Location<'static>,
}

impl<E: Error + 'static> Error for LocatedError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.inner)
    }
}

impl<E: Error + 'static> std::fmt::Display for LocatedError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.inner, self.location)
    }
}

impl From<std::io::Error> for LocatedError<std::io::Error> {
    #[track_caller]
    fn from(err: std::io::Error) -> Self {
        LocatedError {
            inner: err,
            location: std::panic::Location::caller(),
        }
    }
}

impl From<LErr> for LocatedError<LErrEnum> {
    #[track_caller]
    fn from(err: LErr) -> Self {
        LocatedError {
            inner: LErrEnum::Standard(err),
            location: std::panic::Location::caller(),
        }
    }
}

impl From<LErrEnum> for LocatedError<LErrEnum> {
    #[track_caller]
    fn from(err: LErrEnum) -> Self {
        LocatedError {
            inner: err,
            location: std::panic::Location::caller(),
        }
    }
}

impl From<ParseIntError> for LocatedError<LErrEnum> {
    #[track_caller]
    fn from(err: ParseIntError) -> Self {
        LocatedError {
            inner: LErrEnum::IntExpected(err),
            location: std::panic::Location::caller(),
        }
    }
}

impl From<ParseFloatError> for LocatedError<LErrEnum> {
    #[track_caller]
    fn from(err: ParseFloatError) -> Self {
        LocatedError {
            inner: LErrEnum::FloatExpected(err),
            location: std::panic::Location::caller(),
        }
    }
}

impl From<PestError<Rule>> for LocatedError<LErrEnum> {
    #[track_caller]
    fn from(err: PestError<Rule>) -> Self {
        LocatedError {
            inner: LErrEnum::PestError(err),
            location: std::panic::Location::caller(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LErrEnum {
    Standard(LErr),
    ExprExpected,
    IntExpected(ParseIntError),
    FloatExpected(ParseFloatError),
    PestError(PestError<Rule>),
    Rule(Rule),
    None,
}

impl Error for LErrEnum {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self)
    }
}

impl From<PestError<Rule>> for LErrEnum {
    fn from(e: PestError<Rule>) -> Self {
        Self::PestError(e)
    }
}

impl From<Rule> for LErrEnum {
    fn from(e: Rule) -> Self {
        Self::Rule(e)
    }
}

impl From<ParseIntError> for LErrEnum {
    fn from(e: ParseIntError) -> Self {
        Self::IntExpected(e)
    }
}

impl From<ParseFloatError> for LErrEnum {
    fn from(e: ParseFloatError) -> Self {
        Self::FloatExpected(e)
    }
}

impl Display for LErrEnum {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            Self::Standard(e) => write!(f, "Error: {}.", e.error),
            Self::ExprExpected => write!(f, "Expected expression"),
            Self::IntExpected(e) => write!(f, "Expected int: {}", e),
            Self::FloatExpected(e) => write!(f, "Expected float: {}", e),
            Self::PestError(e) => write!(f, "Syntax error: {}", e),
            Self::Rule(e) => write!(f, "Rule error: {:?}", e),
            Self::None => write!(f, "None returned"),
        }
    }
}
