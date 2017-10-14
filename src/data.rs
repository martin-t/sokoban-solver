use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Add;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapCell {
    Wall,
    Empty,
    Goal,
    Remover,
}

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

#[derive(Debug)]
pub struct MyVec2d<T>(pub Vec<Vec<T>>); // TODO rename :)

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

use std::ops::{Index, IndexMut};

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

#[derive(Debug)]
pub struct Map {
    pub map: MyVec2d<MapCell>,
    pub goals: Vec<Pos>,
    pub dead_ends: MyVec2d<bool>,
}

impl Map {
    pub fn new(map: MyVec2d<MapCell>, goals: Vec<Pos>) -> Self {
        let dead_ends = map.create_scratch_map(false);
        Map {
            map,
            goals,
            dead_ends,
        }
    }

    pub fn print(&self, state: &State) {
        let mut state_grid = self.map.create_scratch_map(0);
        for &b in state.boxes.iter() {
            state_grid[b] = 1;
        }
        state_grid[state.player_pos] = 2;
        for r in 0..self.map.0.len() {
            for c in 0..self.map.0[r].len() {
                let pos = Pos::new(r, c);
                let cell = self.map[pos];
                if cell == MapCell::Wall {
                    print!("<>");
                    continue;
                }
                match state_grid[pos] {
                    0 => print!(" "),
                    1 => print!("B"),
                    2 => print!("P"),
                    _ => unreachable!(),
                }
                match cell {
                    MapCell::Empty => print!(" "),
                    MapCell::Goal => print!("_"),
                    MapCell::Remover => print!("R"),
                    _ => unreachable!(),
                }
            }
            println!();
        }
        println!();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct State {
    pub player_pos: Pos,
    pub boxes: Vec<Pos>,
    // TODO keep this sorted to discover duplicates
}

impl State {
    pub fn new(player_pos: Pos, boxes: Vec<Pos>) -> State {
        State { player_pos, boxes }
    }
}

#[derive(Debug)]
pub struct SearchState {
    pub state: State,
    pub prev: Option<State>,
    pub dist: i32,
    pub h: i32,
}

impl Ord for SearchState {
    fn cmp(&self, other: &Self) -> Ordering {
        // intentionally reversed for BinaryHeap
        //other.heuristic().cmp(&self.heuristic())
        (other.dist + other.h).cmp(&(self.dist + self.h))
    }
}

impl PartialOrd for SearchState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for SearchState {}

impl PartialEq for SearchState {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pos {
    pub r: i32,
    pub c: i32,
}

impl Pos {
    // TODO profile with i8
    pub fn new(r: usize, c: usize) -> Pos {
        Pos {
            r: r as i32,
            c: c as i32,
        }
    }

    pub fn dist(self, other: Pos) -> i32 {
        (self.r - other.r).abs() + (self.c - other.c).abs()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Dir {
    pub r: i32,
    pub c: i32,
}

impl Add<Dir> for Pos {
    type Output = Pos;

    fn add(self, dir: Dir) -> Pos {
        Pos { r: self.r + dir.r, c: self.c + dir.c }
    }
}
