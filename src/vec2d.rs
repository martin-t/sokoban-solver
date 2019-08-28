use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Index, IndexMut};

use crate::data::{MapCell, Pos};

#[derive(Clone, PartialEq, Eq)]
crate struct Vec2d<T> {
    data: Vec<T>,
    rows: u8,
    cols: u8,
}

impl<T> Vec2d<T> {
    crate fn new(grid: &[Vec<T>]) -> Self
    where
        T: Clone + Default,
    {
        let max_cols = grid.iter().map(Vec::len).max().unwrap_or(0);
        let mut data = Vec::with_capacity(grid.len() * max_cols);
        for row in grid {
            for c in row {
                data.push(c.clone());
            }
            for _ in row.len()..max_cols {
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
        U: Clone,
    {
        Vec2d {
            data: vec![default; self.data.len()],
            rows: self.rows,
            cols: self.cols,
        }
    }

    crate fn scratchpad<U>(&self) -> Vec2d<U>
    where
        U: Clone + Default,
    {
        self.scratchpad_with_default(U::default())
    }

    crate fn positions(&self) -> Positions {
        Positions {
            rows: self.rows,
            cols: self.cols,
            cur_r: 0,
            cur_c: 0,
        }
    }
}

crate struct Positions {
    rows: u8,
    cols: u8,
    cur_r: u8,
    cur_c: u8,
}

impl Iterator for Positions {
    type Item = Pos;

    fn next(&mut self) -> Option<Pos> {
        if self.cur_r == self.rows {
            return None;
        }

        let ret = Pos::new(self.cur_r, self.cur_c);

        self.cur_c += 1;
        if self.cur_c == self.cols {
            self.cur_c = 0;
            self.cur_r += 1;
        }

        Some(ret)
    }
}

impl<T: Display> Display for Vec2d<T> {
    default fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let data: Vec<String> = self.data.iter().map(|t| format!("{}", t)).collect();
        fmt_t(data, self.cols.into(), f)
    }
}

impl<T: Debug> Debug for Vec2d<T> {
    default fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let data: Vec<String> = self.data.iter().map(|t| format!("{:?}", t)).collect();
        fmt_t(data, self.cols.into(), f)
    }
}

fn fmt_t(data: Vec<String>, cols: usize, f: &mut Formatter<'_>) -> fmt::Result {
    if cols == 0 {
        // chunk size must be >0
        return Ok(());
    }
    let longest = data.iter().map(|s| s.len()).max().unwrap_or(0);
    for row in data.chunks(cols) {
        for cell in row {
            write!(f, " {:>width$}", cell, width = longest)?;
        }
        writeln!(f)?;
    }
    Ok(())
}

impl Display for Vec2d<MapCell> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.cols == 0 {
            // chunk size must be >0
            return Ok(());
        }
        for row in self.data.chunks(self.cols.into()) {
            for &cell in row {
                write!(f, "{}", cell)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Debug for Vec2d<MapCell> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for Vec2d<bool> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.cols == 0 {
            // chunk size must be >0
            return Ok(());
        }
        for row in self.data.chunks(self.cols.into()) {
            for &cell in row {
                write!(f, "{}", if cell { 1 } else { 0 })?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Debug for Vec2d<bool> {
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
    use super::*;
    use crate::level::Level;

    #[test]
    fn positions() {
        let v = Vec2d::new(&[vec![0, 1, 2], vec![3, 4, 5], vec![6, 7, 8]]);
        let nums: Vec<_> = v.positions().map(|p| v[p]).collect();
        assert_eq!(nums, &[0, 1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn formatting_map_cell() {
        let xsb_level: &str = r"
*####*
#@$.*#
*####*#
"
        .trim_start_matches('\n');
        // the `\n\` is necessary because intellij removes trailing whitespace
        let xsb_grid: &str = "
.####. \n\
#  ..# \n\
.####.#
"
        .trim_start_matches('\n');
        let level: Level = xsb_level.parse().unwrap();

        println!("{}", level.map().grid());
        println!("{:?}", level.map().grid());

        assert_eq!(format!("{}", level.map().grid()), xsb_grid);
        assert_eq!(format!("{:?}", level.map().grid()), xsb_grid);
    }

    #[test]
    fn formatting_bool() {
        let v2d = Vec2d::new(&[
            vec![true, false, true],
            vec![false, true, false],
            vec![true, false, true],
        ]);

        let expected = "101\n010\n101\n";

        assert_eq!(format!("{}", v2d), expected);
        assert_eq!(format!("{:?}", v2d), expected);
    }

    #[test]
    fn formatting_t() {
        // currently i only care about Ts which print themselves on a single line

        let v2d = Vec2d::new(&[
            vec![1, 2, 3],
            vec![42, 1337, 666],
            vec![4, 5, 6],
            vec![7, 88, 9],
        ]);

        let expected = r"
    1    2    3
   42 1337  666
    4    5    6
    7   88    9
"
        .trim_start_matches('\n');

        assert_eq!(format!("{}", v2d), expected);
        assert_eq!(format!("{:?}", v2d), expected);
    }

    #[test]
    fn formatting_empty() {
        let v2d: Vec2d<bool> = Vec2d::new(&[]);
        assert_eq!(format!("{}", v2d), "");
        assert_eq!(format!("{:?}", v2d), "");

        let v2d: Vec2d<&str> = Vec2d::new(&[]);
        assert_eq!(format!("{}", v2d), "");
        assert_eq!(format!("{:?}", v2d), "");
    }
}
