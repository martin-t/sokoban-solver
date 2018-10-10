use std::cmp::Ordering;
use std::fmt::{self, Debug, Display, Formatter, Result};
use std::hash::Hash;
use std::ops::{Add, Sub};

use separator::Separatable;

use crate::state::State;

#[derive(PartialEq, Eq)]
pub struct Stats {
    created_states: Vec<i32>,
    visited_states: Vec<i32>,
    duplicate_states: Vec<i32>,
}

impl Stats {
    crate fn new() -> Self {
        Stats {
            created_states: vec![],
            duplicate_states: vec![],
            visited_states: vec![],
        }
    }

    crate fn total_created(&self) -> i32 {
        self.created_states.iter().sum::<i32>()
    }

    crate fn total_unique_visited(&self) -> i32 {
        self.visited_states.iter().sum::<i32>()
    }

    crate fn total_reached_duplicates(&self) -> i32 {
        self.duplicate_states.iter().sum::<i32>()
    }

    pub(super) fn add_created(&mut self, depth: u16) -> bool {
        Self::add(&mut self.created_states, depth)
    }

    pub(super) fn add_unique_visited(&mut self, depth: u16) -> bool {
        Self::add(&mut self.visited_states, depth)
    }

    pub(super) fn add_reached_duplicate(&mut self, depth: u16) -> bool {
        Self::add(&mut self.duplicate_states, depth)
    }

    fn add(counts: &mut Vec<i32>, depth: u16) -> bool {
        let mut ret = false;

        // `while` because some depths might be skipped - duplicates or tunnel optimizations (NYI)
        let depth: usize = depth.into();
        while depth >= counts.len() {
            counts.push(0);
            ret = true;
        }
        counts[depth] += 1;
        ret
    }
}

impl Debug for Stats {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "total created / unique visited / reached duplicates:",)?;
        writeln!(
            f,
            "{:16}{:17}{}", // or "      {:12}{:19}{}" and 2 spaces around slashes
            self.total_created().separated_string(),
            self.total_unique_visited().separated_string(),
            self.total_reached_duplicates().separated_string()
        )?;

        //writeln!(f, "created by depth: {:?}", self.created_states)?;
        //writeln!(f, "unique visited by depth: {:?}", self.visited_states)?;
        //writeln!(f, "reached duplicates by depth: {:?}", self.duplicate_states)?;
        Ok(())
    }
}

impl Display for Stats {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let created = self.total_created();
        let visited = self.total_unique_visited();
        let duplicates = self.total_reached_duplicates();
        let left = created - visited - duplicates;

        #[cfg_attr(rustfmt, rustfmt_skip)]
        {
            writeln!(f, "States created total: {}", created.separated_string())?;
            writeln!(f, "Unique visited total: {}", visited.separated_string())?;
            writeln!(f, "Reached duplicates total: {}", duplicates.separated_string())?;
            writeln!(f, "Created but not reached total: {}",left.separated_string())?;
            writeln!(f)?;
            writeln!(f, "Depth          Created        Unique         Duplicates     Unknown (not reached)")?;
        }

        for i in 0..self.created_states.len() {
            // created_states should be the longest vec
            let depth = format!("{}: ", i);
            let created = self.created_states[i];
            let visited = if i < self.visited_states.len() {
                self.visited_states[i]
            } else {
                0
            };
            let duplicates = if i < self.duplicate_states.len() {
                self.duplicate_states[i]
            } else {
                0
            };
            let left = created - visited - duplicates;
            writeln!(
                f,
                "{:<15}{:<15}{:<15}{:<15}{}",
                depth,
                created.separated_string(),
                visited.separated_string(),
                duplicates.separated_string(),
                left.separated_string()
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
crate struct SearchNode<'a, C: Cost + Add<Output = C>> {
    crate state: &'a State,
    crate prev: Option<&'a State>,
    crate dist: C,
    crate cost: C,
}

impl<'a, C: Cost + Add<Output = C>> SearchNode<'a, C> {
    crate fn new(state: &'a State, prev: Option<&'a State>, dist: C, heuristic: C) -> Self {
        Self {
            state,
            prev,
            dist,
            cost: dist + heuristic,
        }
    }
}

crate trait Cost:
    Sized + Display + Copy + Ord + Eq + Hash + Add<Output = Self> + Sub<Output = Self>
{
    fn zero() -> Self;
    fn one() -> Self;
    fn depth(&self) -> u16;
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
crate struct SimpleCost(crate u16);

impl Display for SimpleCost {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Debug for SimpleCost {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Add for SimpleCost {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        SimpleCost(self.0 + other.0)
    }
}

impl Sub for SimpleCost {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        SimpleCost(self.0 - other.0)
    }
}

impl Cost for SimpleCost {
    fn zero() -> Self {
        SimpleCost(0)
    }

    fn one() -> Self {
        SimpleCost(1)
    }

    fn depth(&self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
crate struct ComplexCost(crate u16, crate u16);

impl Display for ComplexCost {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.0, self.1)
    }
}

impl Debug for ComplexCost {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Add for ComplexCost {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        ComplexCost(self.0 + other.0, self.1 + other.1)
    }
}

impl Sub for ComplexCost {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        ComplexCost(self.0 - other.0, self.1 - other.1)
    }
}

impl Cost for ComplexCost {
    fn zero() -> Self {
        ComplexCost(0, 0)
    }

    fn one() -> Self {
        ComplexCost(1, 0)
    }

    fn depth(&self) -> u16 {
        self.0
    }
}

crate struct CostComparator<'a, C: Cost + Add<Output = C>>(crate SearchNode<'a, C>);

impl<'a, C: Cost + Add<Output = C>> PartialOrd for CostComparator<'a, C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, C: Cost + Add<Output = C>> Ord for CostComparator<'a, C> {
    fn cmp(&self, other: &Self) -> Ordering {
        // orders acording to cost lowest to highest
        // needs std::cmp::Reverse when using BinaryHeap (it's a max heap)
        // according to Criterion, the difference between Reversed and actually reversing the order
        // (if any) is usually within noise threshold
        (self.0.cost).cmp(&(other.0.cost))
    }
}

impl<'a, C: Cost + Add<Output = C>> PartialEq for CostComparator<'a, C> {
    fn eq(&self, other: &Self) -> bool {
        self.0.cost == other.0.cost
    }
}

impl<'a, C: Cost + Add<Output = C>> Eq for CostComparator<'a, C> {}
