use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Index, IndexMut};

use data::{MapCell, Pos};

#[derive(Clone, PartialEq, Eq)]
crate struct Vec2d<T> {
    data: Vec<T>,
    rows: u8,
    cols: u8,
}

impl<T> Vec2d<T> {
    crate fn rows(&self) -> u8 {
        self.rows
    }

    crate fn cols(&self) -> u8 {
        self.cols
    }
}

impl Vec2d<MapCell> {
    crate fn new(grid: &Vec<Vec<MapCell>>) -> Self {
        assert!(grid.len() > 0 && grid[0].len() > 0);

        let max_cols = grid.iter().map(|row| row.len()).max().unwrap();
        let mut data = Vec::with_capacity(grid.len() * max_cols);
        for row in grid.iter() {
            for i in 0..row.len() {
                data.push(row[i]);
            }
            for _ in row.len()..max_cols {
                data.push(MapCell::Empty);
            }
        }
        Vec2d {
            data,
            rows: grid.len() as u8,
            cols: max_cols as u8,
        }
    }

    crate fn create_scratchpad<T: Copy>(&self, default: T) -> Vec2d<T> {
        Vec2d {
            data: vec![default; self.data.len()],
            rows: self.rows,
            cols: self.cols,
        }
    }
}

impl<T: Display> Display for Vec2d<T> {
    default fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for row in self.data.chunks(self.cols.into()) {
            for cell in row {
                write!(f, "{}", cell)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Display for Vec2d<bool> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for row in self.data.chunks(self.cols.into()) {
            for &cell in row {
                write!(f, "{}", if cell { 1 } else { 0 })?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl<T: Display> Debug for Vec2d<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl<T> Index<Pos> for Vec2d<T> {
    type Output = T;

    fn index(&self, index: Pos) -> &Self::Output {
        let index = usize::from(index.r) * usize::from(self.cols) + usize::from(index.c);
        // unchecked indexing is only marginally faster (if at all) to justify unsafe
        //unsafe { self.data.get_unchecked(index) }
        &self.data[index]
    }
}

impl<T> IndexMut<Pos> for Vec2d<T> {
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        let index = usize::from(index.r) * usize::from(self.cols) + usize::from(index.c);
        //unsafe { self.data.get_unchecked_mut(index) }
        &mut self.data[index]
    }
}
