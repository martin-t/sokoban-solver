use std::fmt::{self, Debug, Display, Formatter};

use crate::config::Format;
use crate::data::{Contents, MapCell, Pos};
use crate::state::State;
use crate::vec2d::Vec2d;

pub struct MapFormatter<'a> {
    grid: &'a Vec2d<MapCell>,
    state: Option<&'a State>,
    format: Format,
}

impl<'a> MapFormatter<'a> {
    pub(crate) fn new(grid: &'a Vec2d<MapCell>, state: Option<&'a State>, format: Format) -> Self {
        Self {
            grid,
            state,
            format,
        }
    }

    fn write_to_formatter(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut state_grid = self.grid.scratchpad();
        if let Some(state) = self.state {
            for &b in &state.boxes {
                state_grid[b] = Contents::Box;
            }
            state_grid[state.player_pos] = Contents::Player;
        }

        for r in 0..self.grid.rows() {
            // don't print trailing empty cells to match the input level strings
            let mut last_non_empty = 0;
            for c in 0..self.grid.cols() {
                let pos = Pos::new(r, c);
                if self.grid[pos] != MapCell::Empty || state_grid[pos] != Contents::Empty {
                    last_non_empty = pos.c;
                }
            }

            for c in 0..=last_non_empty {
                let pos = Pos::new(r, c);
                let cell = self.grid[pos];

                match self.format {
                    Format::Custom => Self::write_cell_custom(cell, state_grid[pos], f)?,
                    Format::Xsb => Self::write_cell_xsb(cell, state_grid[pos], f)?,
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }

    fn write_cell_custom(cell: MapCell, contents: Contents, f: &mut Formatter<'_>) -> fmt::Result {
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
                MapCell::Wall => unreachable!("Wall again"),
            };
        }
        Ok(())
    }

    fn write_cell_xsb(cell: MapCell, contents: Contents, f: &mut Formatter<'_>) -> fmt::Result {
        match (cell, contents) {
            (MapCell::Empty, Contents::Empty) => write!(f, " "),
            (MapCell::Empty, Contents::Box) => write!(f, "$"),
            (MapCell::Empty, Contents::Player) => write!(f, "@"),
            (MapCell::Wall, Contents::Empty) => write!(f, "#"),
            (MapCell::Wall, _) => unreachable!("Wall with non-empty contents"),
            (MapCell::Goal, Contents::Empty) => write!(f, "."),
            (MapCell::Goal, Contents::Box) => write!(f, "*"),
            (MapCell::Goal, Contents::Player) => write!(f, "+"),
            (MapCell::Remover, Contents::Empty) => write!(f, "r"),
            (MapCell::Remover, Contents::Box) => unreachable!("Remover with box"),
            (MapCell::Remover, Contents::Player) => write!(f, "R"),
        }
    }
}

impl<'a> Display for MapFormatter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.write_to_formatter(f)
    }
}

impl<'a> Debug for MapFormatter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
