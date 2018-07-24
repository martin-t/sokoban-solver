use std::fmt;
use std::fmt::{Debug, Display, Formatter};

use crate::config::Format;
use crate::data::State;
use crate::map::{GoalMap, MapFormatter};

#[derive(Clone)]
pub struct Level {
    pub map: GoalMap,
    crate state: State,
}

impl Level {
    crate fn new(map: GoalMap, state: State) -> Self {
        Level { map, state }
    }

    #[allow(unused)]
    crate fn xsb(&self) -> MapFormatter<'_> {
        MapFormatter::new(&self.map.grid, &self.state, Format::Xsb)
    }

    #[allow(unused)]
    crate fn custom(&self) -> MapFormatter<'_> {
        MapFormatter::new(&self.map.grid, &self.state, Format::Custom)
    }

    #[allow(unused)]
    crate fn format(&self, format: Format) -> MapFormatter<'_> {
        MapFormatter::new(&self.map.grid, &self.state, format)
    }
}

impl Display for Level {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.xsb())
    }
}

impl Debug for Level {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.xsb())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Map;

    #[test]
    fn formatting_level() {
        let xsb: &str = r"
*###*
#@$.#
*###*#
"
            .trim_left_matches('\n');
        let custom: &str = r"
B_<><><>B_
<>P B  _<>
B_<><><>B_<>
"
            .trim_left_matches('\n');

        for level in &[xsb, custom] {
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

            assert_eq!(
                level
                    .map
                    .format_with_state(Format::Xsb, &level.state)
                    .to_string(),
                xsb
            );
            assert_eq!(
                level
                    .map
                    .format_with_state(Format::Custom, &level.state)
                    .to_string(),
                custom
            );
        }
    }
}
