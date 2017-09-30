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
pub struct Map {
    pub cells: Vec<Vec<MapCell>>,
    pub goals: Vec<Pos>,
}

impl Map {
    pub fn new(map: Vec<Vec<MapCell>>, goals: Vec<Pos>) -> Map {
        Map { cells: map, goals }
    }

    pub fn create_scratch_map<T>(original: &Vec<Vec<MapCell>>, default: T) -> Vec<Vec<T>> // TODO impl Vec instead?
        where T: Copy
    {
        let mut scratch = Vec::new();
        for row in original.iter() {
            scratch.push(vec![default; row.len()]);
        }
        scratch
    }

    pub fn empty_map_state(&self) -> MapState {
        MapState::new(
            self.cells.iter().map(|row| {
                row.iter().map(|cell| {
                    match *cell {
                        MapCell::Wall => Cell::Wall,
                        MapCell::Empty => Cell::Path(Content::Empty, Tile::Empty),
                        MapCell::Goal => Cell::Path(Content::Empty, Tile::Goal),
                        MapCell::Remover => Cell::Path(Content::Empty, Tile::Remover),
                    }
                }).collect()
            }).collect(),
            self.goals.clone()
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    Wall,
    Path(Content, Tile),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Content {
    Empty,
    Player,
    Box,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tile {
    Empty,
    Goal,
    Remover,
}

#[derive(Debug, Clone)]
pub struct MapState {
    pub map: Vec<Vec<Cell>>,
    pub goals: Vec<Pos>,
    pub dead_ends: Vec<Vec<bool>>,
}

impl MapState {
    #[allow(unused)] // TODO
    pub fn new(cells: Vec<Vec<Cell>>, goals: Vec<Pos>) -> MapState {
        MapState {
            map: cells,
            goals: goals,
            dead_ends: Vec::new(),
        }
    }

    pub fn at(&self, pos: Pos) -> &Cell {
        &self.map[pos.r as usize][pos.c as usize]
    }

    pub fn at_mut(&mut self, pos: Pos) -> &mut Cell {
        &mut self.map[pos.r as usize][pos.c as usize]
    }

    pub fn with_state(self, state: &State) -> MapState {
        self.with_boxes(state).with_player(state)
    }

    pub fn with_boxes(mut self, state: &State) -> MapState {
        for pos in &state.boxes {
            if let Cell::Path(Content::Empty, tile) = *self.at(*pos) {
                *self.at_mut(*pos) = Cell::Path(Content::Box, tile);
            } else {
                unreachable!();
            }
        }
        self
    }

    pub fn with_player(mut self, state: &State) -> MapState {
        if let Cell::Path(Content::Empty, tile) = *self.at(state.player_pos) {
            *self.at_mut(state.player_pos) = Cell::Path(Content::Player, tile);
        } else {
            unreachable!();
        }
        self
    }

    pub fn to_string(&self) -> String {
        let mut res = String::new();
        for row in &self.map {
            for cell in row {
                match *cell {
                    Cell::Wall => res += "<>",
                    Cell::Path(content, tile) => {
                        match content {
                            Content::Empty => res += " ",
                            Content::Box => res += "B",
                            Content::Player => res += "P",
                        }
                        match tile {
                            Tile::Empty => res += " ",
                            Tile::Goal => res += "_",
                            Tile::Remover => res += "R",
                        }
                    }
                }
            }
            res += "\n";
        }
        res
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct State {
    pub player_pos: Pos,
    pub boxes: Vec<Pos>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    /*pub fn neighbors(&self) -> [Pos; 4] {
        [
            Pos { r: self.r + 1, c: self.c },
            Pos { r: self.r - 1, c: self.c },
            Pos { r: self.r, c: self.c + 1 },
            Pos { r: self.r, c: self.c - 1 },
        ]
    }*/
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
