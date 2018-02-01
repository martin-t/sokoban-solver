use level::{Map, State, Vec2d};

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
