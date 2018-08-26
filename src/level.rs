use std::fmt;
use std::fmt::{Debug, Display, Formatter};

use crate::config::Format;
use crate::map::{Map, MapType};
use crate::map_formatter::MapFormatter;
use crate::moves::Moves;
use crate::solution_formatter::SolutionFormatter;
use crate::state::State;

#[derive(Clone)]
pub struct Level {
    crate map: MapType,
    crate state: State,
}

impl Level {
    crate fn new(map: MapType, state: State) -> Self {
        Level { map, state }
    }

    crate fn map(&self) -> &dyn Map {
        self.map.map()
    }

    pub fn xsb(&self) -> MapFormatter<'_> {
        self.format(Format::Xsb)
    }

    pub fn custom(&self) -> MapFormatter<'_> {
        self.format(Format::Custom)
    }

    pub fn format(&self, format: Format) -> MapFormatter<'_> {
        MapFormatter::new(&self.map().grid(), Some(&self.state), format)
    }

    pub fn xsb_solution<'a>(
        &'a self,
        moves: &'a Moves,
        include_steps: bool,
    ) -> SolutionFormatter<'_> {
        self.format_solution(Format::Xsb, moves, include_steps)
    }

    pub fn custom_solution<'a>(
        &'a self,
        moves: &'a Moves,
        include_steps: bool,
    ) -> SolutionFormatter<'_> {
        self.format_solution(Format::Custom, moves, include_steps)
    }

    pub fn format_solution<'a>(
        &'a self,
        format: Format,
        moves: &'a Moves,
        include_steps: bool,
    ) -> SolutionFormatter<'a> {
        SolutionFormatter::new(self.map(), &self.state, moves, include_steps, format)
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

    use crate::data::Dir;
    use crate::moves::Move;

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

    #[test]
    fn formatting_solution() {
        let level = r"
*####*
#@ $.#
*####*";
        let expected_with_steps = r"
*####*
#@ $.#
*####*

*####*
# @$.#
*####*

*####*
#  @*#
*####*

".trim_left_matches('\n');
        let expected_without_steps = r"
*####*
#@ $.#
*####*

*####*
#  @*#
*####*

".trim_left_matches('\n');

        let level: Level = level.parse().unwrap();
        let moves = Moves::new(vec![
            Move::new(Dir::Right, false),
            Move::new(Dir::Right, true),
        ]);

        assert_eq!(
            level.xsb_solution(&moves, true).to_string(),
            expected_with_steps
        );
        assert_eq!(
            level.xsb_solution(&moves, false).to_string(),
            expected_without_steps
        );
    }
}
