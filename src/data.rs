use std::cmp::Ordering;
use std::ops::Add;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    Wall,
    Path(PathCell),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathCell {
    pub content: Content,
    pub tile: Tile,
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
pub struct Map {
    pub map: Vec<Vec<Cell>>,
    pub goals: Vec<Pos>,
    pub dead_ends: Vec<Vec<bool>>,
}

impl Map {
    pub fn at(&self, pos: Pos) -> &Cell {
        &self.map[pos.r as usize][pos.c as usize]
    }

    pub fn at_mut(&mut self, pos: Pos) -> &mut Cell {
        &mut self.map[pos.r as usize][pos.c as usize]
    }

    pub fn with_state(self, state: &State) -> Map {
        self.with_boxes(state).with_player(state)
    }

    pub fn with_boxes(mut self, state: &State) -> Map {
        for pos in &state.boxes {
            if let Cell::Path(ref mut pc) = *self.at_mut(*pos) {
                pc.content = Content::Box;
            } else {
                unreachable!();
            }
        }
        self
    }

    pub fn with_player(mut self, state: &State) -> Map {
        if let Cell::Path(ref mut pc) = *self.at_mut(state.player_pos) {
            pc.content = Content::Player;
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
                    Cell::Path(ref path) => {
                        match path.content {
                            Content::Empty => res += " ",
                            Content::Box => res += "B",
                            Content::Player => res += "P",
                        }
                        match path.tile {
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
