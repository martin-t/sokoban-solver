use std::ops::Add;


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Format {
    Custom,
    Xsb,
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapCell {
    Wall,
    Empty,
    Goal,
    Remover,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Contents {
    Empty,
    Box,
    Player,
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


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pos {
    pub r: u8,
    pub c: u8,
}

impl Pos {
    pub fn new(r: u8, c: u8) -> Pos {
        Pos { r, c }
    }

    pub fn dist(self, other: Pos) -> i32 {
        (self.r as i32 - other.r as i32).abs() + (self.c as i32 - other.c as i32).abs()
    }

    pub fn neighbors(self) -> [Pos; 4] {
        [
            Pos { r: self.r - 1, c: self.c },
            Pos { r: self.r, c: self.c + 1 },
            Pos { r: self.r + 1, c: self.c },
            Pos { r: self.r, c: self.c - 1 },
        ]
    }
}

impl Add<Dir> for Pos {
    type Output = Pos;

    fn add(self, dir: Dir) -> Pos {
        match dir {
            Dir::Up => Pos { r: self.r - 1, c: self.c },
            Dir::Right => Pos { r: self.r, c: self.c + 1 },
            Dir::Down => Pos { r: self.r + 1, c: self.c },
            Dir::Left => Pos { r: self.r, c: self.c - 1 },
        }
    }
}


pub const DIRECTIONS: [Dir; 4] = [Dir::Up, Dir::Right, Dir::Down, Dir::Left];

#[derive(Debug, Clone, Copy)]
pub enum Dir {
    Up,
    Right,
    Down,
    Left,
}
