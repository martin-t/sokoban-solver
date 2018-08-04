use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Index, IndexMut};

use crate::data::Pos;

#[derive(Clone, PartialEq, Eq)]
crate struct Vec2d<T> {
    data: Vec<T>,
    rows: u8,
    cols: u8,
}

impl<T> Vec2d<T> {
    crate fn new(grid: &[Vec<T>]) -> Self
    where
        T: Copy + Default,
    {
        assert!(!grid.is_empty() && !grid[0].is_empty());

        let max_cols = grid.iter().map(|row| row.len()).max().unwrap();
        let mut data = Vec::with_capacity(grid.len() * max_cols);
        for row in grid.iter() {
            for c in row.iter() {
                data.push(*c);
            }
            for _ in row.len()..max_cols {
                // could also fill with wall but then we wouldn't detect
                // some broken maps and silently accept them instead
                data.push(T::default());
            }
        }
        Vec2d {
            data,
            rows: grid.len() as u8,
            cols: max_cols as u8,
        }
    }

    crate fn rows(&self) -> u8 {
        self.rows
    }

    crate fn cols(&self) -> u8 {
        self.cols
    }

    crate fn scratchpad_with_default<U>(&self, default: U) -> Vec2d<U>
    where
        U: Copy,
    {
        Vec2d {
            data: vec![default; self.data.len()],
            rows: self.rows,
            cols: self.cols,
        }
    }

    crate fn scratchpad<U>(&self) -> Vec2d<U>
    where
        U: Copy + Default,
    {
        self.scratchpad_with_default(U::default())
    }
}

impl<T: Display> Display for Vec2d<T> {
    default fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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

#[cfg(test)]
mod tests {
    use crate::level::Level;

    #[test]
    fn formatting_vec2d() {
        let xsb_level: &str = r"
*####*
#@$.*#
*####*#
".trim_left_matches('\n');
        // the `\n\` is necessary because intellij removes trailing whitespace
        let xsb_grid: &str = "
.####. \n\
#  ..# \n\
.####.#
".trim_left_matches('\n');
        let level: Level = xsb_level.parse().unwrap();

        assert_eq!(format!("{}", level.map.grid), xsb_grid);
        assert_eq!(format!("{:?}", level.map.grid), xsb_grid);
    }
}
