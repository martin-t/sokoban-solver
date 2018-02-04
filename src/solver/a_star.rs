use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter, Result};

use separator::Separatable;

use level::State;

#[derive(PartialEq, Eq)]
pub struct Stats {
    created_states: Vec<i32>,
    visited_states: Vec<i32>,
    duplicate_states: Vec<i32>,
}

impl Stats {
    pub fn new() -> Self {
        Stats { created_states: vec![], duplicate_states: vec![], visited_states: vec![] }
    }

    pub fn total_created(&self) -> i32 {
        self.created_states.iter().sum::<i32>()
    }

    pub fn total_unique_visited(&self) -> i32 {
        self.visited_states.iter().sum::<i32>()
    }

    pub fn total_reached_duplicates(&self) -> i32 {
        self.duplicate_states.iter().sum::<i32>()
    }

    pub fn add_created(&mut self, state: &SearchState) -> bool {
        Self::add(&mut self.created_states, state)
    }

    pub fn add_unique_visited(&mut self, state: &SearchState) -> bool {
        Self::add(&mut self.visited_states, state)
    }

    pub fn add_reached_duplicate(&mut self, state: &SearchState) -> bool {
        Self::add(&mut self.duplicate_states, state)
    }

    fn add(counts: &mut Vec<i32>, state: &SearchState) -> bool {
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
        writeln!(f, "total created / unique visited / reached duplicates:", )?;
        writeln!(f, "{:16}{:17}{}", // or "      {:12}{:19}{}" and 2 spaces around slashes
                 self.total_created().separated_string(),
                 self.total_unique_visited().separated_string(),
                 self.total_reached_duplicates().separated_string())?;

        //writeln!(f, "created by depth: {:?}", self.created_states)?;
        //writeln!(f, "unique visited by depth: {:?}", self.visited_states)?;
        //writeln!(f, "reached duplicates by depth: {:?}", self.duplicate_states)?;
        Ok(())
    }
}

impl Display for Stats {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let created = self.total_created();
        let visited = self.total_unique_visited();
        let duplicates = self.total_reached_duplicates();
        let left = created - visited - duplicates;
        writeln!(f, "States created total: {}", created.separated_string())?;
        writeln!(f, "Unique visited total: {}", visited.separated_string())?;
        writeln!(f, "Reached duplicates total: {}", duplicates.separated_string())?;
        writeln!(f, "Created but not reached total: {}", left.separated_string())?;
        writeln!(f, "")?;

        writeln!(f, "Depth / created states:")?;
        writeln!(f, "|                   Depth / unique visited:")?;
        writeln!(f, "|                   |                   Depth / reached duplicate:")?;
        writeln!(f, "|                   |                   |                   Depth / created but not reached:")?;
        for i in 0..self.created_states.len() { // created_states should be the longest vec
            let depth = format!("{}: ", i);
            let created = self.created_states[i];
            let visited = if i < self.visited_states.len() { self.visited_states[i] } else { 0 };
            let duplicates = if i < self.duplicate_states.len() { self.duplicate_states[i] } else { 0 };
            let left = created - visited - duplicates;
            writeln!(f, "{0:<5}{1:<15}{0:<5}{2:<15}{0:<5}{3:<15}{0:<5}{4}",
                     depth,
                     created.separated_string(),
                     visited.separated_string(),
                     duplicates.separated_string(),
                     left.separated_string())?;
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

impl PartialOrd for SearchState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SearchState {
    fn cmp(&self, other: &Self) -> Ordering {
        // intentionally reversed for BinaryHeap
        (other.dist + other.h).cmp(&(self.dist + self.h))
    }
}

impl PartialEq for SearchState {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
    }
}

impl Eq for SearchState {}
