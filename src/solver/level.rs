use std::fmt;
use std::fmt::{Display, Formatter};

use data::{Format, State};
use level::{Map, MapFormatter, Vec2d};


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

    pub fn xsb(&self) -> MapFormatter {
        MapFormatter::new(&self.map, &self.state, Format::Xsb)
    }

    pub fn custom(&self) -> MapFormatter {
        MapFormatter::new(&self.map, &self.state, Format::Custom)
    }

    pub fn format(&self, format: Format) -> MapFormatter {
        MapFormatter::new(&self.map, &self.state, format)
    }
}

impl Display for SolverLevel {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.xsb().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solver;

    #[test]
    fn formatting() {
        let xsb: &str = r"
*###*
#@$.#
*###*#
".trim_left_matches('\n');
        let custom: &str = r"
B_<><><>B_
<>P B  _<>
B_<><><>B_<>
".trim_left_matches('\n');
        let processed_xsb: &str = r"
######
#@$.##
######
".trim_left_matches('\n');
        let processed_custom: &str = r"
<><><><><><>
<>P B  _<><>
<><><><><><>
".trim_left_matches('\n');

        for level in [xsb, custom].iter() {
            let level: SolverLevel = solver::process_level(&level.parse().unwrap()).unwrap();
            assert_eq!(level.to_string(), processed_xsb);
            assert_eq!(level.xsb().to_string(), processed_xsb);
            assert_eq!(level.format(Format::Xsb).to_string(), processed_xsb);
            assert_eq!(format!("{}", level), processed_xsb);

            assert_eq!(level.custom().to_string(), processed_custom);
            assert_eq!(level.format(Format::Custom).to_string(), processed_custom);
            assert_eq!(format!("{}", level.custom()), processed_custom);

            assert_eq!(level.map.format_with_state(Format::Xsb, &level.state).to_string(), processed_xsb);
            assert_eq!(level.map.format_with_state(Format::Custom, &level.state).to_string(), processed_custom);
        }
    }
}
