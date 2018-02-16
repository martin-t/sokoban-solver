use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Index, IndexMut};

use data::{Format, MapCell, Content, State, Pos};
use extensions::Scratch;


#[derive(Debug, Clone)]
pub struct SolverLevel {
    pub map: SolverMap,
    pub state: State,
    pub dead_ends: Vec2d<bool>,
}

impl SolverLevel {
    pub fn new(map: SolverMap, state: State, dead_ends: Vec2d<bool>) -> Self {
        Self { map, state, dead_ends }
    }
}

impl Display for SolverLevel {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "{}", self.map.to_string(&self.state, Format::Xsb))
    }
}


#[derive(Debug, Clone)]
pub struct SolverMap {
    pub grid: Vec2d<MapCell>,
    pub goals: Vec<Pos>,
}

impl SolverMap {
    pub fn new(grid: Vec2d<MapCell>, goals: Vec<Pos>) -> Self {
        SolverMap { grid, goals }
    }

    // TODO deduplicate (SolverMap vs Map)
    pub fn to_string(&self, state: &State, format: Format) -> String {
        match format {
            Format::Custom => self.to_string_custom(state),
            Format::Xsb => self.to_string_xsb(state),
        }
    }

    fn to_string_custom(&self, state: &State) -> String {
        let mut ret = String::new();

        let mut state_grid = self.grid.create_scratchpad(Content::Empty);
        for &b in state.boxes.iter() {
            state_grid[b] = Content::Box;
        }
        state_grid[state.player_pos] = Content::Player;

        for r in 0..self.grid.rows() {
            for c in 0..self.grid.cols() {
                let pos = Pos::new(r, c);
                let cell = self.grid[pos];
                if cell == MapCell::Wall {
                    ret.push_str("<>");
                    continue;
                }
                ret.push(match state_grid[pos] {
                    Content::Empty => ' ',
                    Content::Box => 'B',
                    Content::Player => 'P',
                });
                ret.push(match cell {
                    MapCell::Empty => ' ',
                    MapCell::Goal => '_',
                    MapCell::Remover => 'R',
                    _ => unreachable!(),
                });
            }
            ret.push('\n');
        }

        ret
    }

    fn to_string_xsb(&self, state: &State) -> String {
        let mut ret = String::new();

        let mut state_grid = self.grid.create_scratchpad(Content::Empty);
        for &b in state.boxes.iter() {
            state_grid[b] = Content::Box;
        }
        state_grid[state.player_pos] = Content::Player;

        for r in 0..self.grid.rows() {
            for c in 0..self.grid.cols() {
                let pos = Pos::new(r, c);
                let cell = self.grid[pos];

                ret.push(match (cell, state_grid[pos]) {
                    (MapCell::Wall, Content::Empty) => '#',
                    (MapCell::Wall, _) => unreachable!(),
                    (MapCell::Empty, Content::Empty) => ' ',
                    (MapCell::Empty, Content::Box) => '$',
                    (MapCell::Empty, Content::Player) => '@',
                    (MapCell::Goal, Content::Empty) => '.',
                    (MapCell::Goal, Content::Box) => '*',
                    (MapCell::Goal, Content::Player) => '+',
                    (MapCell::Remover, Content::Empty) => 'r',
                    (MapCell::Remover, Content::Box) => unreachable!(),
                    (MapCell::Remover, Content::Player) => 'R',
                });
            }
            ret.push('\n');
        }

        ret
    }
}


// TODO bench a single vector as map representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vec2d<T>(Vec<Vec<T>>);

impl<T> Vec2d<T> {
    pub fn rows(&self) -> u8 {
        self.0.len() as u8
    }

    pub fn cols(&self) -> u8 {
        self.0[0].len() as u8
    }
}

impl Vec2d<MapCell> {
    pub fn new(grid: &Vec<Vec<MapCell>>) -> Self {
        assert!(grid.len() > 0 && grid[0].len() > 0);

        // pad all rows to have the same length
        let max_cols = grid.iter().map(|row| row.len()).max().unwrap();
        let mut new_grid = Vec::new();
        for row in grid.iter() {
            let mut new_row = row.clone();
            while new_row.len() < max_cols {
                new_row.push(MapCell::Wall);
            }
            new_grid.push(new_row);
        }
        Vec2d(new_grid)
    }
}

impl<TIn, TOut: Copy> Scratch<TOut> for Vec2d<TIn> {
    type Result = Vec2d<TOut>;

    fn create_scratchpad(&self, default: TOut) -> Self::Result {
        let mut scratch = Vec::new();
        for row in self.0.iter() {
            scratch.push(vec![default; row.len()]);
        }
        Vec2d(scratch)
    }
}

impl<T: Display> Display for Vec2d<T> {
    default fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for row in self.0.iter() {
            for cell in row.iter() {
                write!(f, "{}", cell)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Display for Vec2d<bool> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for row in self.0.iter() {
            for &cell in row.iter() {
                write!(f, "{}", if cell { 1 } else { 0 })?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl<T> Index<Pos> for Vec2d<T> {
    type Output = T;

    fn index(&self, index: Pos) -> &Self::Output {
        &self.0[index.r as usize][index.c as usize]
    }
}

impl<T> IndexMut<Pos> for Vec2d<T> {
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        &mut self.0[index.r as usize][index.c as usize]
    }
}
