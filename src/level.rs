use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Index, IndexMut};

use data::{Format, MapCell, Contents, State, Pos};


pub struct MapFormatter<'a> {
    map: &'a Map,
    state: &'a State,
    format: Format,
}

impl<'a> MapFormatter<'a> {
    pub fn new(map: &'a Map, state: &'a State, format: Format) -> Self {
        Self { map, state, format }
    }
}

impl<'a> Display for MapFormatter<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.map.write_with_state(self.state, self.format, f)
    }
}

impl<'a> Debug for MapFormatter<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone)]
pub struct Level {
    pub map: Map,
    pub state: State,
}

impl Level {
    pub fn new(map: Map, state: State) -> Self {
        Level { map, state }
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


#[derive(Clone)]
pub struct Map {
    pub grid: Vec2d<MapCell>,
    pub goals: Vec<Pos>,
}

impl Map {
    pub fn new(grid: Vec2d<MapCell>, goals: Vec<Pos>) -> Self {
        Map { grid, goals }
    }

    pub fn format_with_state<'a>(&'a self, format: Format, state: &'a State) -> MapFormatter<'a> {
        MapFormatter::new(self, state, format)
    }

    fn write_with_state(&self, state: &State, format: Format, f: &mut Formatter) -> fmt::Result {
        let mut state_grid = self.grid.create_scratchpad(Contents::Empty);
        for &b in state.boxes.iter() {
            state_grid[b] = Contents::Box;
        }
        state_grid[state.player_pos] = Contents::Player;
        self.write(state_grid, format, f)
    }

    fn write(&self, state_grid: Vec2d<Contents>, format: Format, f: &mut Formatter) -> fmt::Result {
        for r in 0..self.grid.rows() {

            // don't print trailing empty cells to match the input level strings
            let mut last_non_empty = 0;
            for c in 0..self.grid.cols() {
                let pos = Pos::new(r as u8, c as u8);
                if self.grid[pos] != MapCell::Empty || state_grid[pos] != Contents::Empty {
                    last_non_empty = pos.c;
                }
            }

            for c in 0..last_non_empty + 1 {
                let pos = Pos::new(r as u8, c as u8);
                let cell = self.grid[pos];

                match format {
                    Format::Custom => Self::write_custom(cell, state_grid[pos], f)?,
                    Format::Xsb => Self::write_xsb(cell, state_grid[pos], f)?,
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }

    fn write_custom(cell: MapCell, contents: Contents, f: &mut Formatter) -> fmt::Result {
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

    fn write_xsb(cell: MapCell, contents: Contents, f: &mut Formatter) -> fmt::Result {
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
}

impl Display for Map {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let state_grid = self.grid.create_scratchpad(Contents::Empty);
        self.write(state_grid, Format::Xsb, f)
    }
}

impl Debug for Map {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}


// TODO bench a single vector as map representation
#[derive(Clone, PartialEq, Eq)]
pub struct Vec2d<T> {
    data: Vec<T>,
    rows: u8,
    cols: u8,
}

impl<T> Vec2d<T> {
    pub fn rows(&self) -> u8 {
        self.rows
    }

    pub fn cols(&self) -> u8 {
        self.cols
    }
}

impl Vec2d<MapCell> {
    pub fn new(grid: &Vec<Vec<MapCell>>) -> Self {
        assert!(grid.len() > 0 && grid[0].len() > 0);

        let max_cols = grid.iter().map(|row| row.len()).max().unwrap();
        let mut data = Vec::new(); // TODO bench reserving
        for row in grid.iter() {
            for i in 0..row.len() {
                data.push(row[i]);
            }
            for _ in row.len()..max_cols {
                data.push(MapCell::Empty);
            }
        }
        Vec2d {
            data,
            rows: grid.len() as u8,
            cols: max_cols as u8,
        }
    }

    pub fn create_scratchpad<T: Copy>(&self, default: T) -> Vec2d<T> {
        Vec2d {
            data: vec![default; self.data.len()],
            rows: self.rows,
            cols: self.cols,
        }
    }
}

impl<T: Display> Display for Vec2d<T> {
    default fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for row in self.data.chunks(self.cols.into()) {
            for cell in row {
                write!(f, "{}", cell)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Display for Vec2d<bool> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for row in self.data.chunks(self.cols.into()) {
            for &cell in row {
                write!(f, "{}", if cell { 1 } else { 0 })?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl<T: Display> Debug for Vec2d<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl<T> Index<Pos> for Vec2d<T> {
    type Output = T;

    fn index(&self, index: Pos) -> &Self::Output {
        let index = usize::from(index.r) * usize::from(self.cols) + usize::from(index.c);
        &self.data[index]
    }
}

impl<T> IndexMut<Pos> for Vec2d<T> {
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        let index = usize::from(index.r) * usize::from(self.cols) + usize::from(index.c);
        &mut self.data[index]
    }
}

#[cfg(test)]
mod tests {
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

    #[test]
    fn formatting_map() {
        let xsb_level: &str = r"
*###*
#@$.#
*###*#
".trim_left_matches('\n');
        let xsb_map: &str = "
.###.
#  .#
.###.#
".trim_left_matches('\n');
        // the `\n\` is necessary because intellij removes trailing whitespace
        let xsb_grid: &str = "
.###. \n\
#  .# \n\
.###.#
".trim_left_matches('\n');

        let level: Level = xsb_level.parse().unwrap();
        assert_eq!(format!("{}", level.map), xsb_map);
        assert_eq!(format!("{:?}", level.map), xsb_map);
        assert_eq!(format!("{}", level.map.grid), xsb_grid);
        assert_eq!(format!("{:?}", level.map.grid), xsb_grid);
    }
}
