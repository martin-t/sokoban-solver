use std::fmt;
use std::fmt::{Debug, Display, Formatter};

use data::{Contents, Format, MapCell, Pos, State};
use vec2d::Vec2d;

crate struct MapFormatter<'a> {
    grid: &'a Vec2d<MapCell>,
    state: &'a State,
    format: Format,
}

impl<'a> MapFormatter<'a> {
    crate fn new(grid: &'a Vec2d<MapCell>, state: &'a State, format: Format) -> Self {
        Self {
            grid,
            state,
            format,
        }
    }
}

impl<'a> Display for MapFormatter<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write_with_state(&self.grid, self.state, self.format, f)
    }
}

impl<'a> Debug for MapFormatter<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

crate trait Map {
    fn format_with_state<'a>(&'a self, format: Format, state: &'a State) -> MapFormatter<'a>;
}

fn write_with_state(
    grid: &Vec2d<MapCell>,
    state: &State,
    format: Format,
    f: &mut Formatter,
) -> fmt::Result {
    let mut state_grid = grid.create_scratchpad(Contents::Empty);
    for &b in &state.boxes {
        state_grid[b] = Contents::Box;
    }
    state_grid[state.player_pos] = Contents::Player;
    write(grid, &state_grid, format, f)
}

fn write(
    grid: &Vec2d<MapCell>,
    state_grid: &Vec2d<Contents>,
    format: Format,
    f: &mut Formatter,
) -> fmt::Result {
    for r in 0..grid.rows() {
        // don't print trailing empty cells to match the input level strings
        let mut last_non_empty = 0;
        for c in 0..grid.cols() {
            let pos = Pos::new(r as u8, c as u8);
            if grid[pos] != MapCell::Empty || state_grid[pos] != Contents::Empty {
                last_non_empty = pos.c;
            }
        }

        for c in 0..last_non_empty + 1 {
            let pos = Pos::new(r as u8, c as u8);
            let cell = grid[pos];

            match format {
                Format::Custom => write_cell_custom(cell, state_grid[pos], f)?,
                Format::Xsb => write_cell_xsb(cell, state_grid[pos], f)?,
            }
        }
        writeln!(f)?;
    }
    Ok(())
}

fn write_cell_custom(cell: MapCell, contents: Contents, f: &mut Formatter) -> fmt::Result {
    if cell == MapCell::Wall {
        write!(f, "<>")?;
    } else {
        match contents {
            Contents::Empty => write!(f, " ")?,
            Contents::Box => write!(f, "B")?,
            Contents::Player => write!(f, "P")?,
        };
        match cell {
            MapCell::Empty => write!(f, " ")?,
            MapCell::Goal => write!(f, "_")?,
            MapCell::Remover => write!(f, "R")?,
            _ => unreachable!(),
        };
    }
    Ok(())
}

fn write_cell_xsb(cell: MapCell, contents: Contents, f: &mut Formatter) -> fmt::Result {
    match (cell, contents) {
        (MapCell::Wall, Contents::Empty) => write!(f, "#"),
        (MapCell::Wall, _) => unreachable!(),
        (MapCell::Empty, Contents::Empty) => write!(f, " "),
        (MapCell::Empty, Contents::Box) => write!(f, "$"),
        (MapCell::Empty, Contents::Player) => write!(f, "@"),
        (MapCell::Goal, Contents::Empty) => write!(f, "."),
        (MapCell::Goal, Contents::Box) => write!(f, "*"),
        (MapCell::Goal, Contents::Player) => write!(f, "+"),
        (MapCell::Remover, Contents::Empty) => write!(f, "r"),
        (MapCell::Remover, Contents::Box) => unreachable!(),
        (MapCell::Remover, Contents::Player) => write!(f, "R"),
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
    fn format_with_state<'a>(&'a self, format: Format, state: &'a State) -> MapFormatter<'a> {
        MapFormatter::new(&self.grid, state, format)
    }
}

impl Display for GoalMap {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let state_grid = self.grid.create_scratchpad(Contents::Empty);
        write(&self.grid, &state_grid, Format::Xsb, f)
    }
}

impl Debug for GoalMap {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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
    use level::Level;

    #[test]
    fn formatting_map() {
        let xsb_level: &str = r"
*###*
#@$.#
*###*#
"
            .trim_left_matches('\n');
        let xsb_map: &str = "
.###.
#  .#
.###.#
"
            .trim_left_matches('\n');
        // the `\n\` is necessary because intellij removes trailing whitespace
        let xsb_grid: &str = "
.###. \n\
#  .# \n\
.###.#
"
            .trim_left_matches('\n');

        let level: Level = xsb_level.parse().unwrap();
        assert_eq!(format!("{}", level.map), xsb_map);
        assert_eq!(format!("{:?}", level.map), xsb_map);
        assert_eq!(format!("{}", level.map.grid), xsb_grid);
        assert_eq!(format!("{:?}", level.map.grid), xsb_grid);
    }
}
