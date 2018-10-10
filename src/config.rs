use std::fmt::{self, Display, Formatter};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Custom,
    Xsb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    MoveOptimalMinPushes,
    MoveOptimal,
    PushOptimalMinMoves,
    PushOptimal,
    Any,
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Method::MoveOptimalMinPushes => write!(f, "move-optimal-min-pushes"),
            Method::MoveOptimal => write!(f, "move-optimal"),
            Method::PushOptimalMinMoves => write!(f, "push-optimal-min-moves"),
            Method::PushOptimal => write!(f, "push-optimal"),
            Method::Any => write!(f, "any"),
        }
    }
}
