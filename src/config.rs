use std::fmt::{self, Display, Formatter};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Custom,
    Xsb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    MoveOptimal,
    PushOptimal,
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Method::MoveOptimal => write!(f, "move-optimal"),
            Method::PushOptimal => write!(f, "push-optimal"),
        }
    }
}
