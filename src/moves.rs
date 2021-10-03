use std::fmt::{self, Debug, Display, Formatter};

use crate::data::Dir;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move {
    crate dir: Dir,
    crate is_push: bool,
}

impl Move {
    crate fn new(dir: Dir, is_push: bool) -> Self {
        Move { dir, is_push }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.is_push {
            write!(f, "{}", self.dir.to_string().to_uppercase())?;
        } else {
            write!(f, "{}", self.dir)?;
        }
        Ok(())
    }
}

impl Debug for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct Moves(Vec<Move>);

impl Moves {
    #[cfg(test)]
    crate fn new(moves: Vec<Move>) -> Self {
        Moves(moves)
    }

    pub fn move_cnt(&self) -> usize {
        self.0.len()
    }

    pub fn push_cnt(&self) -> usize {
        self.0.iter().filter(|m| m.is_push).count()
    }

    crate fn add(&mut self, mov: Move) {
        self.0.push(mov);
    }

    crate fn extend(&mut self, moves: &Moves) {
        self.0.extend_from_slice(&moves.0);
    }

    #[allow(unused)]
    crate fn into_iter(self) -> ::std::vec::IntoIter<Move> {
        self.0.into_iter()
    }

    #[allow(unused)]
    crate fn iter(&self) -> ::std::slice::Iter<'_, Move> {
        self.0.iter()
    }
}

impl IntoIterator for Moves {
    type Item = Move;
    type IntoIter = ::std::vec::IntoIter<Move>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Moves {
    type Item = &'a Move;
    type IntoIter = ::std::slice::Iter<'a, Move>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Display for Moves {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for mov in self {
            write!(f, "{}", mov)?;
        }
        Ok(())
    }
}

impl Debug for Moves {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatting_moves() {
        let moves = Moves::new(vec![
            Move::new(Dir::Up, false),
            Move::new(Dir::Right, false),
            Move::new(Dir::Down, false),
            Move::new(Dir::Left, false),
            Move::new(Dir::Up, true),
            Move::new(Dir::Right, true),
            Move::new(Dir::Down, true),
            Move::new(Dir::Left, true),
        ]);
        assert_eq!(moves.to_string(), "urdlURDL");
    }

    #[test]
    fn extending_and_counting() {
        let mut moves1 = Moves::new(vec![
            Move::new(Dir::Up, true),
            Move::new(Dir::Right, true),
            Move::new(Dir::Down, true),
            Move::new(Dir::Left, true),
        ]);

        let moves2 = Moves::new(vec![
            Move::new(Dir::Up, false),
            Move::new(Dir::Right, false),
            Move::new(Dir::Down, false),
            Move::new(Dir::Left, false),
        ]);

        assert_eq!(moves1.move_cnt(), 4);
        assert_eq!(moves1.push_cnt(), 4);
        assert_eq!(moves2.move_cnt(), 4);
        assert_eq!(moves2.push_cnt(), 0);

        moves1.extend(&moves2);

        assert_eq!(moves1.move_cnt(), 8);
        assert_eq!(moves1.push_cnt(), 4);
    }

    #[test]
    fn iterating() {
        let v = vec![
            Move::new(Dir::Up, false),
            Move::new(Dir::Right, false),
            Move::new(Dir::Down, false),
            Move::new(Dir::Left, false),
            Move::new(Dir::Up, true),
            Move::new(Dir::Right, true),
            Move::new(Dir::Down, true),
            Move::new(Dir::Left, true),
        ];
        let moves = Moves::new(v.clone());

        let mut v2 = Vec::new();
        for &m in &moves {
            v2.push(m);
        }
        for &m in moves.iter() {
            v2.push(m);
        }
        for m in moves.clone() {
            v2.push(m);
        }
        for m in moves.into_iter() {
            v2.push(m);
        }

        assert_eq!(v2.len(), 32);
        for chunk in v2.chunks(8) {
            assert_eq!(chunk, &v[..]);
        }
    }
}
