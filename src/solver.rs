use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter, Result};

use separator::Separatable;

use data::*; // TODO pick

const UP: Dir = Dir { r: -1, c: 0 };
const RIGHT: Dir = Dir { r: 0, c: 1 };
const DOWN: Dir = Dir { r: 1, c: 0 };
const LEFT: Dir = Dir { r: 0, c: -1 };
const DIRECTIONS: [Dir; 4] = [UP, RIGHT, DOWN, LEFT];

pub struct Stats {
    pub created_states: Vec<i32>,
    pub duplicate_states: Vec<i32>,
    pub visited_states: Vec<i32>,
}

impl Stats {
    fn new() -> Self {
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

    fn add_created(&mut self, state: &SearchState) -> bool {
        Self::add(&mut self.created_states, state)
    }

    fn add_duplicate(&mut self, state: &SearchState) -> bool {
        Self::add(&mut self.duplicate_states, state)
    }

    fn add_visited(&mut self, state: &SearchState) -> bool {
        Self::add(&mut self.visited_states, state)
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

pub fn search(map: &MapState, initial_state: &State, print_status: bool)
              -> (Option<Vec<State>>, Stats)
{
    let mut stats = Stats::new();

    let mut to_visit = BinaryHeap::new();
    let mut closed = HashSet::new();
    let mut prev = HashMap::new();

    let h = heuristic(&map, &initial_state);
    let start = SearchState {
        state: initial_state.clone(),
        prev: None,
        dist: 0,
        h: h,
    };
    stats.add_created(&start);
    to_visit.push(start);
    while let Some(current) = to_visit.pop() {
        if closed.contains(&current.state) {
            stats.add_duplicate(&current);
            continue;
        }
        if stats.add_visited(&current) && print_status {
            println!("Visited new depth: {}", current.dist);
            println!("{:?}", stats);
        }

        // insert here and not as soon as we discover it
        // otherwise we overwrite the shortest path with longer ones
        if let Some(p) = current.prev {
            prev.insert(current.state.clone(), p.clone());
        }

        if solved(map, &current.state) {
            return (Some(backtrack_path(&prev, &current.state)), stats);
        }

        for neighbor_state in expand(&map, &current.state) {
            // TODO this could probably be optimized a bit by allocating on the heap
            // and storing references only (to current state, neighbor state is always different)
            //prev.insert(neighbor_state.clone(), current.state.clone());

            // insert and then ignore duplicates
            let h = heuristic(&map, &neighbor_state);
            let next = SearchState {
                state: neighbor_state,
                prev: Some(current.state.clone()),
                dist: current.dist + 1,
                h: h,
            };
            stats.add_created(&next);
            to_visit.push(next);
        }

        closed.insert(current.state);
    }

    (None, stats)
}

fn expand(map: &MapState, state: &State) -> Vec<State> {
    //expand_move(map, state)
    expand_push(map, state)
}

fn heuristic(map: &MapState, state: &State) -> i32 {
    //heuristic_move(map, state)
    heuristic_push(map, state)
}

pub fn mark_dead_ends(map: &mut MapState) {
    // TODO test case
    // #####
    // ##@##
    // ##$##
    // #  .#
    // #####

    // init first since otherwise we would use this partially initialized in search()
    for r in 0..map.map.len() {
        map.dead_ends.push(Vec::new());
        for _ in &map.map[r] {
            map.dead_ends[r].push(false);
        }
    }

    for r in 0..map.map.len() {
        'cell: for c in 0..map.map[r].len() {
            let box_pos = Pos { r: r as i32, c: c as i32 };
            if let &Cell::Wall = map.at(box_pos) {
                //print!("w");
                continue;
            }

            for dir in DIRECTIONS.iter() {
                let player_pos = box_pos + *dir;
                if let &Cell::Wall = map.at(player_pos) { continue; }

                let fake_state = State {
                    player_pos: player_pos,
                    boxes: vec![box_pos],
                };
                if let Some(_) = search(map, &fake_state, false).0 {
                    //print!("cont");
                    continue 'cell; // need to find only one solution
                }
            }
            map.dead_ends[r][c] = true; // no solution from any direction
        }
    }
}

fn heuristic_push(map: &MapState, state: &State) -> i32 {
    let mut goal_dist_sum = 0;
    for box_pos in &state.boxes {
        let mut min = i32::max_value();
        for goal in &map.goals {
            let dist = box_pos.dist(*goal);
            if dist < min {
                min = dist;
            }
        }
        goal_dist_sum += min;
    }
    goal_dist_sum
}

#[allow(unused)]
fn heuristic_move(map: &MapState, state: &State) -> i32 {
    // less is better

    let mut closest_box = i32::max_value();
    for box_pos in &state.boxes {
        let dist = state.player_pos.dist(*box_pos);
        if dist < closest_box {
            closest_box = dist;
        }
    }

    let mut goal_dist_sum = 0;
    for box_pos in &state.boxes {
        let mut min = i32::max_value();
        for goal in &map.goals {
            let dist = box_pos.dist(*goal);
            if dist < min {
                min = dist;
            }
        }
        goal_dist_sum += min;
    }

    closest_box + goal_dist_sum
}

fn backtrack_path(prev: &HashMap<State, State>, final_state: &State) -> Vec<State> {
    let mut ret = Vec::new();
    let mut state = final_state;
    loop {
        ret.push(state.clone());
        if let Some(prev) = prev.get(state) {
            state = prev;
        } else {
            ret.reverse();
            return ret;
        }
    }
}

fn solved(map: &MapState, state: &State) -> bool {
    // to detect dead ends, this has to test all boxes are on a goal, not that all goals have a box
    for pos in &state.boxes {
        if let &Cell::Path(_, Tile::Goal) = map.at(*pos) {} else {
            return false;
        }
    }
    true
}

fn expand_push(map: &MapState, state: &State) -> Vec<State> {
    let mut new_states = Vec::new();

    let map_state = map.clone().with_boxes(&state);

    let mut reachable = Vec::new();
    for r in 0..map.map.len() {
        reachable.push(Vec::new());
        for _ in 0..map.map[r].len() {
            reachable[r].push(false)
        }
    }

    mark_reachable(&map_state, &mut reachable, state.player_pos, state, &mut new_states);

    new_states
}

fn mark_reachable(map_state: &MapState, reachable: &mut Vec<Vec<bool>>,
                  pos: Pos, state: &State, new_states: &mut Vec<State>) {
    let r = pos.r as usize;
    let c = pos.c as usize;
    reachable[r][c] = true;
    for dir in DIRECTIONS.iter() {
        let new_pos = pos + *dir;
        if let Cell::Path(Content::Empty, _) = *map_state.at(new_pos) {
            if !reachable[new_pos.r as usize][new_pos.c as usize] {
                mark_reachable(map_state, reachable, new_pos, state, new_states);
            }
        } else if let Cell::Path(Content::Box, _) = *map_state.at(new_pos) {
            let behind_box = new_pos + *dir;
            if let Cell::Path(Content::Empty, _) = *map_state.at(behind_box) {
                if !map_state.dead_ends[behind_box.r as usize][behind_box.c as usize] {
                    let mut new_boxes = state.boxes.clone();
                    for box_pos in &mut new_boxes {
                        if *box_pos == new_pos {
                            *box_pos = behind_box;
                        }
                    }
                    let new_state = State {
                        player_pos: new_pos,
                        boxes: new_boxes,
                    };
                    new_states.push(new_state);
                }
            }
        }
    }
}

#[allow(unused)]
fn expand_move(map: &MapState, state: &State) -> Vec<State> {
    let mut new_states = Vec::new();

    let map_state = map.clone().with_boxes(&state);
    for dir in DIRECTIONS.iter() {
        let new_pos = state.player_pos + *dir;
        if let Cell::Path(Content::Empty, _) = *map_state.at(new_pos) {
            let new_state = State {
                player_pos: new_pos,
                boxes: state.boxes.clone(),
            };
            new_states.push(new_state);
        } else if let Cell::Path(Content::Box, _) = *map_state.at(new_pos) {
            let behind_box = new_pos + *dir;
            if let Cell::Path(Content::Empty, _) = *map_state.at(behind_box) {
                if !map.dead_ends[behind_box.r as usize][behind_box.c as usize] {
                    // goal will never be a dead end - no need to check
                    let mut new_boxes = state.boxes.clone();
                    for box_pos in &mut new_boxes {
                        if *box_pos == new_pos {
                            *box_pos = behind_box;
                        }
                    }
                    let new_state = State {
                        player_pos: new_pos,
                        boxes: new_boxes,
                    };
                    new_states.push(new_state);
                }
            }
        }
    }

    new_states
}
