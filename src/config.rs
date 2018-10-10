use std::fmt::{self, Display, Formatter};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Custom,
    Xsb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    MovesPushes,
    Moves,
    PushesMoves,
    Pushes,
    Any,
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
