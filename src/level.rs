use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Index, IndexMut};

use data::{Format, Pos};


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
    pub grid: Vec2d<MapCell>,
    pub goals: Vec<Pos>,
}

impl Map {
    pub fn new(grid: Vec2d<MapCell>, goals: Vec<Pos>) -> Self {
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

        let mut state_grid = self.grid.create_scratch_map(Content::Empty);
        for &b in state.boxes.iter() {
            state_grid[b] = Content::Box;
        }
        state_grid[state.player_pos] = Content::Player;

        for r in 0..self.grid.0.len() {
            for c in 0..self.grid.0[r].len() {
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

        let mut state_grid = self.grid.create_scratch_map(Content::Empty);
        for &b in state.boxes.iter() {
            state_grid[b] = Content::Box;
        }
        state_grid[state.player_pos] = Content::Player;

        for r in 0..self.grid.0.len() {
            for c in 0..self.grid.0[r].len() {
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
// TODO rename / unify with Vec2d trait :)
// TODO don't allow creating empty (if possible without a perf hit)
#[derive(Debug, Clone)]
pub struct Vec2d<T>(pub Vec<Vec<T>>);

impl<T> Vec2d<T> {
    pub fn create_scratch_map<U>(&self, default: U) -> Vec2d<U>
        where U: Copy
    {
        let mut scratch = Vec::new();
        for row in self.0.iter() {
            scratch.push(vec![default; row.len()]);
        }
        Vec2d(scratch)
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


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapCell {
    Wall,
    Empty,
    Goal,
    Remover,
}

// TODO unify with print_empty
impl Display for MapCell {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            MapCell::Wall => '#',
            MapCell::Empty => ' ',
            MapCell::Goal => '.',
            MapCell::Remover => 'r',
        })
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct State {
    pub player_pos: Pos,
    pub boxes: Vec<Pos>,
}

impl State {
    pub fn new(player_pos: Pos, mut boxes: Vec<Pos>) -> State {
        boxes.sort(); // sort to detect equal states when we reorder boxes
        State { player_pos, boxes }
    }
}


#[derive(Debug, Clone, Copy)]
enum Content {
    Empty,
    Box,
    Player,
}
