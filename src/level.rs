use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Index, IndexMut};

use data::{Format, MapCell, Content, State, Pos};
use extensions::Scratch;


#[derive(Debug, Clone)]
pub struct Level {
    pub map: Map,
    pub state: State,
}

impl Level {
    pub fn new(map: Map, state: State) -> Self {
        Level { map, state }
    }

    // TODO default to XSB everywhere for Display
    pub fn to_string(&self, format: Format) -> String {
        self.map.to_string(&self.state, format)
    }
}

impl Display for Level {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string(Format::Xsb))
    }
}


#[derive(Debug, Clone)]
pub struct Map {
    pub grid: VecVec<MapCell>,
    pub goals: Vec<Pos>,
}

impl Map {
    pub fn new(grid: VecVec<MapCell>, goals: Vec<Pos>) -> Self {
        Map { grid, goals }
    }

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

        for r in 0..self.grid.0.len() {
            for c in 0..self.grid.0[r].len() {
                let pos = Pos::new(r as u8, c as u8);
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

        for r in 0..self.grid.0.len() {
            for c in 0..self.grid.0[r].len() {
                let pos = Pos::new(r as u8, c as u8);
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


// TODO would be nice to make the internal vector private
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VecVec<T>(pub Vec<Vec<T>>);

impl<T> VecVec<T> {
    pub fn new(grid: Vec<Vec<T>>) -> Self {
        VecVec(grid)
    }

    /*pub fn len(&self) -> usize {
        self.0.len()
    }*/

    /*pub fn iter(&self) -> Iter<Vec<T>> {
        self.0.iter()
    }*/
}

impl<TIn, TOut: Copy> Scratch<TOut> for VecVec<TIn> {
    type Result = VecVec<TOut>;

    fn create_scratchpad(&self, default: TOut) -> Self::Result {
        let mut scratch = Vec::new();
        for row in self.0.iter() {
            scratch.push(vec![default; row.len()]);
        }
        VecVec(scratch)
    }
}

impl<T> Index<Pos> for VecVec<T> {
    type Output = T;

    fn index(&self, index: Pos) -> &Self::Output {
        &self.0[index.r as usize][index.c as usize]
    }
}

impl<T> IndexMut<Pos> for VecVec<T> {
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        &mut self.0[index.r as usize][index.c as usize]
    }
}

impl<T> Index<(usize, usize)> for VecVec<T> {
    type Output = T;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.0[index.0][index.1]
    }
}

impl<T> IndexMut<(usize, usize)> for VecVec<T> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.0[index.0][index.1]
    }
}
