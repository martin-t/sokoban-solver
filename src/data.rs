use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Add;

crate const MAX_SIZE: usize = 255;
crate const MAX_BOXES: usize = 255;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
crate enum MapCell {
    Empty,
    Wall,
    Goal,
    Remover,
}

impl Default for MapCell {
    fn default() -> MapCell {
        MapCell::Empty
    }
}

impl Display for MapCell {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                MapCell::Empty => ' ',
                MapCell::Wall => '#',
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

impl Default for Contents {
    fn default() -> Contents {
        Contents::Empty
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

    crate fn dist(self, other: Pos) -> i16 {
        (i16::from(self.r) - i16::from(other.r)).abs()
            + (i16::from(self.c) - i16::from(other.c)).abs()
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
