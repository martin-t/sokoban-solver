use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Add;

crate const MAX_SIZE: usize = 255;
crate const MAX_BOXES: usize = 254;

#[derive(Clone, Copy, Debug, PartialEq)]
crate enum Format {
    Custom,
    Xsb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
crate enum MapCell {
    Wall,
    Empty,
    Goal,
    Remover,
}

impl Display for MapCell {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                MapCell::Wall => '#',
                MapCell::Empty => ' ',
                MapCell::Goal => '.',
                MapCell::Remover => 'r',
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
crate enum Contents {
    Empty,
    Box,
    Player,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
crate struct State {
    crate player_pos: Pos,
    crate boxes: Vec<Pos>,
}

impl State {
    crate fn new(player_pos: Pos, mut boxes: Vec<Pos>) -> State {
        boxes.sort(); // sort to detect equal states when we reorder boxes
        State { player_pos, boxes }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
crate struct Pos {
    crate r: u8,
    crate c: u8,
}

impl Pos {
    crate fn new(r: u8, c: u8) -> Pos {
        Pos { r, c }
    }

    crate fn dist(self, other: Pos) -> i32 {
        (i32::from(self.r) - i32::from(other.r)).abs()
            + (i32::from(self.c) - i32::from(other.c)).abs()
    }

    crate fn neighbors(self) -> [Pos; 4] {
        [
            Pos {
                r: self.r - 1,
                c: self.c,
            },
            Pos {
                r: self.r,
                c: self.c + 1,
            },
            Pos {
                r: self.r + 1,
                c: self.c,
            },
            Pos {
                r: self.r,
                c: self.c - 1,
            },
        ]
    }
}

impl Add<Dir> for Pos {
    type Output = Pos;

    fn add(self, dir: Dir) -> Pos {
        #![allow(suspicious_arithmetic_impl)]
        match dir {
            Dir::Up => Pos {
                r: self.r - 1,
                c: self.c,
            },
            Dir::Right => Pos {
                r: self.r,
                c: self.c + 1,
            },
            Dir::Down => Pos {
                r: self.r + 1,
                c: self.c,
            },
            Dir::Left => Pos {
                r: self.r,
                c: self.c - 1,
            },
        }
    }
}

crate const DIRECTIONS: [Dir; 4] = [Dir::Up, Dir::Right, Dir::Down, Dir::Left];

#[derive(Debug, Clone, Copy)]
crate enum Dir {
    Up,
    Right,
    Down,
    Left,
}
