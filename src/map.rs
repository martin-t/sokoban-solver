use std::fmt;
use std::fmt::{Debug, Display, Formatter};

use crate::config::Format;
use crate::data::{MapCell, Pos};
use crate::formatter::MapFormatter;
use crate::state::State;
use crate::vec2d::Vec2d;

// TODO none of this should be pub probably

pub trait Map {
    fn xsb(&self) -> MapFormatter<'_>;
    fn custom(&self) -> MapFormatter<'_>;
    fn format(&self, format: Format) -> MapFormatter<'_>;
    fn xsb_with_state<'a>(&'a self, state: &'a State) -> MapFormatter<'a>;
    fn custom_with_state<'a>(&'a self, state: &'a State) -> MapFormatter<'a>;
    fn format_with_state<'a>(&'a self, format: Format, state: &'a State) -> MapFormatter<'a>;
}

#[derive(Clone)]
pub struct GoalMap {
    crate grid: Vec2d<MapCell>,
    crate goals: Vec<Pos>,
}

impl GoalMap {
    crate fn new(grid: Vec2d<MapCell>, goals: Vec<Pos>) -> Self {
        GoalMap { grid, goals }
    }
}

impl Map for GoalMap {
    fn xsb(&self) -> MapFormatter<'_> {
        MapFormatter::new(&self.grid, None, Format::Xsb)
    }

    fn custom(&self) -> MapFormatter<'_> {
        MapFormatter::new(&self.grid, None, Format::Custom)
    }

    fn format(&self, format: Format) -> MapFormatter<'_> {
        MapFormatter::new(&self.grid, None, format)
    }

    fn xsb_with_state<'a>(&'a self, state: &'a State) -> MapFormatter<'a> {
        MapFormatter::new(&self.grid, Some(state), Format::Xsb)
    }

    fn custom_with_state<'a>(&'a self, state: &'a State) -> MapFormatter<'a> {
        MapFormatter::new(&self.grid, Some(state), Format::Custom)
    }

    fn format_with_state<'a>(&'a self, format: Format, state: &'a State) -> MapFormatter<'a> {
        MapFormatter::new(&self.grid, Some(state), format)
    }
}

impl Display for GoalMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mf = MapFormatter::new(&self.grid, None, Format::Xsb);
        write!(f, "{}", mf)
    }
}

impl Debug for GoalMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[allow(unused)]
crate struct RemoverMap {
    crate grid: Vec2d<MapCell>,
    crate remover: Pos,
}

#[allow(unused)]
impl RemoverMap {
    crate fn new(grid: Vec2d<MapCell>, remover: Pos) -> Self {
        Self { grid, remover }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::Level;

    #[test]
    fn formatting_map() {
        let xsb_level: &str = r"
*####*
#@$.*#
*####*#
".trim_left_matches('\n');
        let xsb_map: &str = "
.####.
#  ..#
.####.#
".trim_left_matches('\n');
        let custom_level: &str = r"
B_<><><><>B_
<>P B  _B_<>
B_<><><><>B_<>
".trim_left_matches('\n');
        let custom_map: &str = r"
 _<><><><> _
<>     _ _<>
 _<><><><> _<>
".trim_left_matches('\n');

        let level: Level = xsb_level.parse().unwrap();
        let map = level.map;

        // default
        assert_eq!(map.to_string(), xsb_map);
        assert_eq!(format!("{}", map), xsb_map);
        assert_eq!(format!("{:?}", map), xsb_map);

        // xsb
        assert_eq!(map.xsb().to_string(), xsb_map);
        assert_eq!(map.format(Format::Xsb).to_string(), xsb_map);
        assert_eq!(format!("{}", map.xsb()), xsb_map);
        assert_eq!(format!("{:?}", map.xsb()), xsb_map);

        // custom
        assert_eq!(map.custom().to_string(), custom_map);
        assert_eq!(map.format(Format::Custom).to_string(), custom_map);
        assert_eq!(format!("{}", map.custom()), custom_map);
        assert_eq!(format!("{:?}", map.custom()), custom_map);

        // with state xsb
        assert_eq!(map.xsb_with_state(&level.state).to_string(), xsb_level);
        assert_eq!(
            map.format_with_state(Format::Xsb, &level.state).to_string(),
            xsb_level
        );

        // with state custom
        assert_eq!(
            map.custom_with_state(&level.state).to_string(),
            custom_level
        );
        assert_eq!(
            map.format_with_state(Format::Custom, &level.state)
                .to_string(),
            custom_level
        );
    }
}
