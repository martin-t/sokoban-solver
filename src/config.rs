use std::fmt::{self, Display, Formatter};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Custom,
    Xsb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Moves,
    Pushes,
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Method::Moves => write!(f, "Moves"),
            Method::Pushes => write!(f, "Pushes"),
        }
    }
}
