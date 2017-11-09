use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter, Result};

use separator::Separatable;

use level::State;

pub struct Stats {
    pub created_states: Vec<i32>,
    pub duplicate_states: Vec<i32>,
    pub visited_states: Vec<i32>,
}

impl Stats {
    // TODO remove pub
    pub fn new() -> Self {
        Stats { created_states: vec![], duplicate_states: vec![], visited_states: vec![] }
    }

    pub fn total_created(&self) -> i32 {
        self.created_states.iter().sum::<i32>()
    }

    pub fn total_duplicate(&self) -> i32 {
        self.duplicate_states.iter().sum::<i32>()
    }

    pub fn total_visited(&self) -> i32 {
        self.visited_states.iter().sum::<i32>()
    }

    pub fn add_created(&mut self, state: &SearchState) -> bool {
        Self::add(&mut self.created_states, state)
    }

    pub fn add_duplicate(&mut self, state: &SearchState) -> bool {
        Self::add(&mut self.duplicate_states, state)
    }

    pub fn add_visited(&mut self, state: &SearchState) -> bool {
        Self::add(&mut self.visited_states, state)
    }

    pub fn add(counts: &mut Vec<i32>, state: &SearchState) -> bool {
        let mut ret = false;

        // while because some depths might be skipped - duplicates or tunnel optimizations (NYI)
        while state.dist as usize >= counts.len() {
            counts.push(0);
            ret = true;
        }
        counts[state.dist as usize] += 1;
        ret
    }
}

impl Debug for Stats {
    fn fmt(&self, f: &mut Formatter) -> Result {
        writeln!(f, "created by depth: {:?}", self.created_states)?;
        writeln!(f, "reached duplicates: {:?}", self.duplicate_states)?;
        writeln!(f, "visited by depth: {:?}", self.visited_states)?;
        writeln!(f, "total created: {}", self.total_created().separated_string())?;
        writeln!(f, "total reached duplicates: {}", self.total_duplicate().separated_string())?;
        writeln!(f, "total visited: {}", self.total_visited().separated_string())
    }
}

impl Display for Stats {
    fn fmt(&self, f: &mut Formatter) -> Result {
        writeln!(f, "States created total: {}", self.total_created().separated_string())?;
        writeln!(f, "Reached duplicates total: {}", self.total_duplicate().separated_string())?;
        writeln!(f, "States visited total: {}", self.total_visited().separated_string())?;
        writeln!(f, "Depth / created states:")?;
        for i in 0..self.created_states.len() {
            writeln!(f, "{}: {}", i, self.created_states[i])?;
        }
        writeln!(f, "Depth / found duplicates:")?;
        for i in 0..self.duplicate_states.len() {
            writeln!(f, "{}: {}", i, self.duplicate_states[i])?;
        }
        writeln!(f, "Depth / visited states:")?;
        for i in 0..self.visited_states.len() {
            writeln!(f, "{}: {}", i, self.visited_states[i])?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct SearchState {
    pub state: State,
    pub prev: Option<State>,
    pub dist: i32,
    pub h: i32,
}

impl Ord for SearchState {
    fn cmp(&self, other: &Self) -> Ordering {
        // intentionally reversed for BinaryHeap
        //other.heuristic().cmp(&self.heuristic())
        (other.dist + other.h).cmp(&(self.dist + self.h))
    }
}

impl PartialOrd for SearchState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for SearchState {}

impl PartialEq for SearchState {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
    }
}
