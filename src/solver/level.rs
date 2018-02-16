use std::fmt;
use std::fmt::{Display, Formatter};

use data::{Format, State};
use level::{Map, Vec2d};


#[derive(Debug, Clone)]
pub struct SolverLevel {
    pub map: Map,
    pub state: State,
    pub dead_ends: Vec2d<bool>,
}

impl SolverLevel {
    pub fn new(map: Map, state: State, dead_ends: Vec2d<bool>) -> Self {
        Self { map, state, dead_ends }
    }
}

impl Display for SolverLevel {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "{}", self.map.to_string(&self.state, Format::Xsb))
    }
}
