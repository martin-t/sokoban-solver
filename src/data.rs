use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Sub};

crate const MAX_SIZE: usize = 255;
crate const MAX_BOXES: usize = 255;

// TODO considering i made a mistake once already it might be worth
// trying to split this into two types - one for remover and one for goals
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
crate enum MapCell {
    Empty,
    Wall,
    Goal,
    Remover,
}

impl Default for MapCell {
    fn default() -> MapCell {
        // Empty imho makes slightly more sense than Wall
        // if changing this, make sure maps without complete borders are rejected properly
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

    #[cfg(test)]
    crate fn dist(self, other: Pos) -> u16 {
        ((i16::from(self.r) - i16::from(other.r)).abs()
            + (i16::from(self.c) - i16::from(other.c)).abs()) as u16
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

    crate fn dir_to(self, new_pos: Pos) -> Dir {
        if self.r - 1 == new_pos.r {
            assert_eq!(self.c, new_pos.c);
            Dir::Up
        } else if self.c + 1 == new_pos.c {
            assert_eq!(self.r, new_pos.r);
            Dir::Right
        } else if self.r + 1 == new_pos.r {
            assert_eq!(self.c, new_pos.c);
            Dir::Down
        } else if self.c - 1 == new_pos.c {
            assert_eq!(self.r, new_pos.r);
            Dir::Left
        } else {
            unreachable!("Positions are not adjacent");
        }
    }
}

impl Add<Dir> for Pos {
    type Output = Pos;

    fn add(self, dir: Dir) -> Pos {
        #![allow(clippy::suspicious_arithmetic_impl)]
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

impl Sub<Dir> for Pos {
    type Output = Pos;

    fn sub(self, dir: Dir) -> Pos {
        #![allow(clippy::suspicious_arithmetic_impl)]
        self + dir.inverse()
    }
}

crate const DIRECTIONS: [Dir; 4] = [Dir::Up, Dir::Right, Dir::Down, Dir::Left];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
crate enum Dir {
    Up,
    Right,
    Down,
    Left,
}

impl Dir {
    crate fn inverse(self) -> Self {
        match self {
            Dir::Up => Dir::Down,
            Dir::Right => Dir::Left,
            Dir::Down => Dir::Up,
            Dir::Left => Dir::Right,
        }
    }
}

impl Display for Dir {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Dir::Up => write!(f, "u"),
            Dir::Right => write!(f, "r"),
            Dir::Down => write!(f, "d"),
            Dir::Left => write!(f, "l"),
        }
    }
}
