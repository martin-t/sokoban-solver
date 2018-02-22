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

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn formatting_level() {
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

        for level in [xsb, custom].iter() {
            let level: Level = level.parse().unwrap();
            assert_eq!(level.to_string(), xsb);
            assert_eq!(level.xsb().to_string(), xsb);
            assert_eq!(level.format(Format::Xsb).to_string(), xsb);
            assert_eq!(format!("{}", level), xsb);
            assert_eq!(format!("{:?}", level), xsb);

            assert_eq!(level.custom().to_string(), custom);
            assert_eq!(level.format(Format::Custom).to_string(), custom);
            assert_eq!(format!("{}", level.custom()), custom);
            assert_eq!(format!("{:?}", level.custom()), custom);

            assert_eq!(level.map.format_with_state(Format::Xsb, &level.state).to_string(), xsb);
            assert_eq!(level.map.format_with_state(Format::Custom, &level.state).to_string(), custom);
        }
    }
}
