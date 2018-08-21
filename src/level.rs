use std::fmt;
use std::fmt::{Debug, Display, Formatter};

use crate::config::Format;
use crate::formatter::MapFormatter;
use crate::map::{GoalMap, Map};
use crate::moves::Moves;
use crate::state::State;

#[derive(Clone)]
pub struct Level {
    crate map: GoalMap,
    crate state: State,
}

impl Level {
    crate fn new(map: GoalMap, state: State) -> Self {
        Level { map, state }
    }

    pub fn xsb(&self) -> MapFormatter<'_> {
        MapFormatter::new(&self.map.grid, Some(&self.state), Format::Xsb)
    }

    pub fn custom(&self) -> MapFormatter<'_> {
        MapFormatter::new(&self.map.grid, Some(&self.state), Format::Custom)
    }

    pub fn format(&self, format: Format) -> MapFormatter<'_> {
        MapFormatter::new(&self.map.grid, Some(&self.state), format)
    }

    pub fn print_solution(&self, moves: &Moves, format: Format) {
        // TODO formating instead of printing
        // TODO verify moves (somebody could pass moves from a different level)
        // TODO unify arg order among other formatting fns

        println!("{}", self.format(format));
        let mut last_state = self.state.clone();
        for &mov in moves {
            let new_player_pos = last_state.player_pos + mov.dir;
            let new_boxes = last_state
                .boxes
                .iter()
                .cloned()
                .map(|b| if b == new_player_pos { b + mov.dir } else { b })
                .collect();
            let new_state = State::new(new_player_pos, new_boxes);
            println!("{}", self.map.format_with_state(format, &new_state));
            last_state = new_state;
        }
    }
}

impl Display for Level {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.xsb())
    }
}

impl Debug for Level {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatting_level() {
        let xsb: &str = r"
*####*
#@$.*#
*####*#
".trim_left_matches('\n');
        let custom: &str = r"
B_<><><><>B_
<>P B  _B_<>
B_<><><><>B_<>
".trim_left_matches('\n');

        for level in &[xsb, custom] {
            let level: Level = level.parse().unwrap();

            // xsb as default
            assert_eq!(level.to_string(), xsb);
            assert_eq!(format!("{}", level), xsb);
            assert_eq!(format!("{:?}", level), xsb);

            // explicit xsb
            assert_eq!(level.xsb().to_string(), xsb);
            assert_eq!(level.format(Format::Xsb).to_string(), xsb);
            assert_eq!(format!("{}", level.xsb()), xsb);
            assert_eq!(format!("{:?}", level.xsb()), xsb);

            // explicit custom
            assert_eq!(level.custom().to_string(), custom);
            assert_eq!(level.format(Format::Custom).to_string(), custom);
            assert_eq!(format!("{}", level.custom()), custom);
            assert_eq!(format!("{:?}", level.custom()), custom);
        }
    }
}
