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


/*const UP: Dir = Dir { r: -1, c: 0 };
const RIGHT: Dir = Dir { r: 0, c: 1 };
const DOWN: Dir = Dir { r: 1, c: 0 };
const LEFT: Dir = Dir { r: 0, c: -1 };
pub const DIRECTIONS: [Dir; 4] = [UP, RIGHT, DOWN, LEFT];

#[derive(Debug, Clone, Copy)]
pub struct Dir {
    pub r: i32,
    pub c: i32,
}*/

pub const DIRECTIONS: [Dir; 4] = [Dir::Up, Dir::Right, Dir::Down, Dir::Left];

#[derive(Debug, Clone, Copy)]
pub enum Dir {
    Up,
    Right,
    Down,
    Left,
}
