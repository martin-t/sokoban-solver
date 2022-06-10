use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Custom,
    Xsb,
}

impl FromStr for Format {
    type Err = CliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "custom" => Ok(Self::Custom),
            "xsb" => Ok(Self::Xsb),
            _ => Err(CliError),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    MovesPushes,
    Moves,
    PushesMoves,
    Pushes,
    Any,
}

impl FromStr for Method {
    type Err = CliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "moves-pushes" => Ok(Method::MovesPushes),
            "moves" => Ok(Method::Moves),
            "pushes-moves" => Ok(Method::PushesMoves),
            "pushes" => Ok(Method::Pushes),
            "any" => Ok(Method::Any),
            _ => Err(CliError),
        }
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Method::MovesPushes => write!(f, "moves-pushes"),
            Method::Moves => write!(f, "moves"),
            Method::PushesMoves => write!(f, "pushes-moves"),
            Method::Pushes => write!(f, "pushes"),
            Method::Any => write!(f, "any"),
        }
    }
}

#[derive(Debug)]
pub struct CliError;

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "failed to parse")
    }
}

impl Error for CliError {}
