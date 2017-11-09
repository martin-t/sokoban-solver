use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Index, IndexMut};

use data::{Format, Pos};

#[derive(Debug, Clone)]
pub struct Level {
    original_map: Map,
    processed_map: Map,
    initial_state: State,
}

#[derive(Debug, Clone)]
pub struct Map {
    pub original_map: MyVec2d<MapCell>,
    pub map: MyVec2d<MapCell>,
    pub goals: Vec<Pos>,
    pub dead_ends: MyVec2d<bool>,
}

#[derive(Debug, Clone)]
pub struct MyVec2d<T>(pub Vec<Vec<T>>); // TODO rename :)

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapCell {
    Wall,
    Empty,
    Goal,
    Remover,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct State {
    pub player_pos: Pos,
    pub boxes: Vec<Pos>,
    // TODO keep this sorted to discover duplicates
}

#[derive(Debug, Clone, Copy)]
enum Content {
    Empty,
    Box,
    Player,
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

impl<T> MyVec2d<T> {
    pub fn create_scratch_map<U>(&self, default: U) -> MyVec2d<U>
        where U: Copy
    {
        let mut scratch = Vec::new();
        for row in self.0.iter() {
            scratch.push(vec![default; row.len()]);
        }
        MyVec2d(scratch)
    }
}

impl<T> Index<Pos> for MyVec2d<T> {
    type Output = T;

    fn index(&self, index: Pos) -> &Self::Output {
        &self.0[index.r as usize][index.c as usize]
    }
}

impl<T> IndexMut<Pos> for MyVec2d<T> {
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        &mut self.0[index.r as usize][index.c as usize]
    }
}

impl Map {
    pub fn new(original_map: MyVec2d<MapCell>, map: MyVec2d<MapCell>, goals: Vec<Pos>) -> Self {
        let dead_ends = map.create_scratch_map(false);
        Map {
            original_map,
            map,
            goals,
            dead_ends,
        }
    }

    pub fn to_string(&self, state: &State, format: Format) -> String {
        match format {
            Format::Custom => self.to_string_custom(state),
            Format::Xsb => self.to_string_xsb(state),
        }
    }

    fn to_string_custom(&self, state: &State) -> String {
        let mut ret = String::new();

        let mut state_grid = self.original_map.create_scratch_map(Content::Empty);
        for &b in state.boxes.iter() {
            state_grid[b] = Content::Box;
        }
        state_grid[state.player_pos] = Content::Player;

        for r in 0..self.original_map.0.len() {
            for c in 0..self.original_map.0[r].len() {
                let pos = Pos::new(r, c);
                let cell = self.original_map[pos];
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

        let mut state_grid = self.original_map.create_scratch_map(Content::Empty);
        for &b in state.boxes.iter() {
            state_grid[b] = Content::Box;
        }
        state_grid[state.player_pos] = Content::Player;

        for r in 0..self.original_map.0.len() {
            for c in 0..self.original_map.0[r].len() {
                let pos = Pos::new(r, c);
                let cell = self.original_map[pos];

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

impl State {
    pub fn new(player_pos: Pos, boxes: Vec<Pos>) -> State {
        State { player_pos, boxes }
    }
}
