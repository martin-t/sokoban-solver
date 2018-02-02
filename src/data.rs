use std::ops::Add;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Format {
    Custom,
    Xsb,
}

// TODO profile with i8,u8,usize

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pos {
    pub r: i32,
    pub c: i32,
}

impl Pos {
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
