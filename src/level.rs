use std::fmt;
use std::fmt::{Debug, Display, Formatter};

use map::{GoalMap, MapFormatter};
use data::{Format, State};

#[derive(Clone)]
pub struct Level {
    pub map: GoalMap,
    pub state: State,
}

impl Level {
    pub fn new(map: GoalMap, state: State) -> Self {
        Level { map, state }
    }

    #[allow(unused)]
    pub fn xsb(&self) -> MapFormatter {
        MapFormatter::new(&self.map, &self.state, Format::Xsb)
    }

    #[allow(unused)]
    pub fn custom(&self) -> MapFormatter {
        MapFormatter::new(&self.map, &self.state, Format::Custom)
    }

    #[allow(unused)]
    pub fn format(&self, format: Format) -> MapFormatter {
        MapFormatter::new(&self.map, &self.state, format)
    }
}

impl Display for Level {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.xsb())
    }
}

impl Debug for Level {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self.xsb())
    }
}
