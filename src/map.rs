use std::fmt;
use std::fmt::{Debug, Display, Formatter};

use crate::config::Format;
use crate::data::{MapCell, Pos};
use crate::map_formatter::MapFormatter;
use crate::state::State;
use crate::vec2d::Vec2d;

crate trait Map {
    fn grid(&self) -> &Vec2d<MapCell>;

    // this is a hack for things that are not performance critical
    // like backtracking and replaying moves during formatting to share some code
    // still would be nice to get rid of it someday
    fn remover(&self) -> Option<Pos>;

    fn xsb(&self) -> MapFormatter<'_> {
        self.format(Format::Xsb)
    }

    fn custom(&self) -> MapFormatter<'_> {
        self.format(Format::Custom)
    }

    fn format(&self, format: Format) -> MapFormatter<'_> {
        MapFormatter::new(self.grid(), None, format)
    }

    fn xsb_with_state<'a>(&'a self, state: &'a State) -> MapFormatter<'a> {
        self.format_with_state(Format::Xsb, state)
    }

    fn custom_with_state<'a>(&'a self, state: &'a State) -> MapFormatter<'a> {
        self.format_with_state(Format::Custom, state)
    }

    fn format_with_state<'a>(&'a self, format: Format, state: &'a State) -> MapFormatter<'a> {
        MapFormatter::new(self.grid(), Some(state), format)
    }
}

impl Display for &dyn Map {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mf = MapFormatter::new(self.grid(), None, Format::Xsb);
        write!(f, "{}", mf)
    }
}

impl Debug for &dyn Map {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Debug, Clone)]
crate enum MapType {
    Goals(GoalMap),
    Remover(RemoverMap),
}

impl MapType {
    crate fn map(&self) -> &dyn Map {
        match self {
            MapType::Goals(ref goals_map) => goals_map,
            MapType::Remover(ref remover_map) => remover_map,
        }
    }
}

impl Map for MapType {
    fn grid(&self) -> &Vec2d<MapCell> {
        self.map().grid()
    }

    fn remover(&self) -> Option<Pos> {
        match self {
            MapType::Goals(gm) => gm.remover(),
            MapType::Remover(rm) => rm.remover(),
        }
    }
}

#[derive(Clone)]
crate struct GoalMap {
    crate grid: Vec2d<MapCell>,
    crate goals: Vec<Pos>,
}

impl GoalMap {
    crate fn new(grid: Vec2d<MapCell>, goals: Vec<Pos>) -> Self {
        GoalMap { grid, goals }
    }
}

impl Map for GoalMap {
    fn grid(&self) -> &Vec2d<MapCell> {
        &self.grid
    }

    fn remover(&self) -> Option<Pos> {
        None
    }
}

// can't impl it for M: Map to share it even though Map is crate visible only:
// https://github.com/rust-lang/rust/issues/48869
impl Display for GoalMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mf = MapFormatter::new(self.grid(), None, Format::Xsb);
        write!(f, "{}", mf)
    }
}

impl Debug for GoalMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone)]
crate struct RemoverMap {
    crate grid: Vec2d<MapCell>,
    crate remover: Pos,
}

impl RemoverMap {
    crate fn new(grid: Vec2d<MapCell>, remover: Pos) -> Self {
        Self { grid, remover }
    }
}

impl Map for RemoverMap {
    fn grid(&self) -> &Vec2d<MapCell> {
        &self.grid
    }

    fn remover(&self) -> Option<Pos> {
        Some(self.remover)
    }
}

impl Display for RemoverMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mf = MapFormatter::new(self.grid(), None, Format::Xsb);
        write!(f, "{}", mf)
    }
}

impl Debug for RemoverMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
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
"
        .trim_start_matches('\n');
        let xsb_map: &str = "
.####.
#  ..#
.####.#
"
        .trim_start_matches('\n');
        let custom_level: &str = r"
B_<><><><>B_
<>P B  _B_<>
B_<><><><>B_<>
"
        .trim_start_matches('\n');
        let custom_map: &str = r"
 _<><><><> _
<>     _ _<>
 _<><><><> _<>
"
        .trim_start_matches('\n');

        let level: Level = xsb_level.parse().unwrap();
        let map = level.map();

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
