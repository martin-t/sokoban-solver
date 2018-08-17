use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter, Result};

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

    pub(super) fn add_created(&mut self, state: &SearchNode<'_>) -> bool {
        Self::add(&mut self.created_states, state)
    }

    pub(super) fn add_unique_visited(&mut self, state: &SearchNode<'_>) -> bool {
        Self::add(&mut self.visited_states, state)
    }

    pub(super) fn add_reached_duplicate(&mut self, state: &SearchNode<'_>) -> bool {
        Self::add(&mut self.duplicate_states, state)
    }

    fn add(counts: &mut Vec<i32>, state: &SearchNode<'_>) -> bool {
        let mut ret = false;

        // `while` because some depths might be skipped - duplicates or tunnel optimizations (NYI)
        while state.dist as usize >= counts.len() {
            counts.push(0);
            ret = true;
        }
        counts[state.dist as usize] += 1;
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

#[derive(Debug)]
crate struct SearchNode<'a> {
    crate state: &'a State,
    crate prev: Option<&'a State>,
    crate dist: u16,
    crate cost: u16,
}

impl<'a> SearchNode<'a> {
    crate fn new(state: &'a State, prev: Option<&'a State>, dist: u16, heuristic: u16) -> Self {
        Self {
            state,
            prev,
            dist,
            cost: dist + heuristic,
        }
    }
}

impl<'a> PartialOrd for SearchNode<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for SearchNode<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        // orders acording to cost lowest to highest
        // needs std::cmp::Reverse when using BinaryHeap (it's a max heap)
        // according to Criterion, the difference between Reversed and actually reversing the order
        // (if any) is usually within noise threshold
        (self.cost).cmp(&(other.cost))
    }
}

impl<'a> PartialEq for SearchNode<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
    }
}

impl<'a> Eq for SearchNode<'a> {}
