pub mod a_star;
mod level;

use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

use data::{Pos, DIRECTIONS};
use extensions::Scratch;
use level::{Level, Map, Vec2d, MapCell, State};

use self::a_star::{SearchState, Stats};
use self::level::SolverLevel;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Moves,
    Pushes,
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Method::Moves => write!(f, "Moves"),
            Method::Pushes => write!(f, "Pushes"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SolverErr {
    TooLarge,
    IncompleteBorder,
    UnreachableBoxes,
    UnreachableGoals,
    //UnreachableRemover,
    TooMany,
    BoxesGoals,
}

impl Display for SolverErr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            SolverErr::TooLarge => write!(f, "Map larger than 255 rows/columns"),
            SolverErr::IncompleteBorder => write!(f, "Player can exit the level because of missing border"),
            SolverErr::UnreachableBoxes => write!(f, "Boxes that are not on goal but can't be reached"),
            SolverErr::UnreachableGoals => write!(f, "Goals that don't have a box but can't be reached"),
            //SolverErr::UnreachableRemover => write!(f, "Remover is not reachable"),
            SolverErr::TooMany => write!(f, "More than 254 reachable boxes or goals"),
            SolverErr::BoxesGoals => write!(f, "Different number of reachable boxes and goals"),
        }
    }
}

pub struct SolverOk {
    // TODO probably wanna use Dirs or Moves eventually
    pub path_states: Option<Vec<State>>,
    pub stats: Stats,
    pub method: Method,
}

impl SolverOk {
    fn new(path_states: Option<Vec<State>>, stats: Stats, method: Method) -> Self {
        Self { path_states, stats, method }
    }
}

impl Debug for SolverOk {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.path_states {
            None => writeln!(f, "No solution")?,
            Some(ref states) => writeln!(f, "{}: {}", self.method, states.len() - 1)?,
        }
        write!(f, "{}", self.stats)
    }
}


pub fn solve(level: &Level, method: Method, print_status: bool) -> Result<SolverOk, SolverErr> {
    let solver_level = process_map(level)?;
    match method {
        Method::Moves => Ok(search(&solver_level, method, print_status, expand_move, heuristic_move)),
        Method::Pushes => Ok(search(&solver_level, method, print_status, expand_push, heuristic_push)),
    }
}

fn process_map(level: &Level) -> Result<SolverLevel, SolverErr> {
    // Only guarantees we have here is the player exists and therefore map is at least 1x1.
    // Do some more low level checking so we can omit some checks later.

    // make sure all rows have the same length
    /*let mut grid = level.map.grid;
    let cols = grid.iter().map(|row| row.len()).max().unwrap();
    for row in grid.iter_mut() {
        while row.len() < cols {
            row.push(MapCell::Wall);
        }
    }*/

    if level.map.grid.rows() > 255 || level.map.grid.cols(0) > 255 {
        return Err(SolverErr::TooLarge);
    }

    let mut to_visit = vec![(level.state.player_pos.r as i32, level.state.player_pos.c as i32)];
    let mut visited = level.map.grid.create_scratchpad(false);

    while !to_visit.is_empty() {
        let (r, c) = to_visit.pop().unwrap();
        visited[(r, c)] = true;

        let neighbors = [(r + 1, c), (r - 1, c), (r, c + 1), (r, c - 1)];
        for &(nr, nc) in neighbors.iter() {
            // this is the only place we need to check bounds (using signed types)
            // everything after that will be surrounded by walls
            // TODO make sure we're not wasting time bounds checking anywhere else
            if nr < 0
                || nc < 0
                || nr as usize >= level.map.grid.rows()
                || nc as usize >= level.map.grid.cols(nr as usize) {
                // we got out of bounds without hitting a wall
                return Err(SolverErr::IncompleteBorder);
            }

            if !visited[(nr, nc)] && level.map.grid[(nr, nc)] != MapCell::Wall {
                to_visit.push((nr, nc));
            }
        }
    }

    // TODO move into specialized function when removers work
    /*if let Some(pos) = remover {
        if !visited.0[pos.r as usize][pos.c as usize] {
            return Err(SolverErr::UnreachableRemover);
        }
    }*/

    let mut reachable_goals = Vec::new();
    let mut reachable_boxes = Vec::new();
    for &pos in level.state.boxes.iter() {
        let (r, c) = (pos.r as usize, pos.c as usize);
        if visited[(r, c)] {
            reachable_boxes.push(pos);
        } else if !level.map.goals.contains(&pos) {
            return Err(SolverErr::UnreachableBoxes);
        }
    }
    for &pos in level.map.goals.iter() {
        let (r, c) = (pos.r as usize, pos.c as usize);
        if visited[(r, c)] {
            reachable_goals.push(pos);
        } else if !level.state.boxes.contains(&pos) {
            return Err(SolverErr::UnreachableGoals);
        }
    }

    // TODO maybe do this first and use it instead of visited when detecting reachability in specialized fns?
    // to avoid errors with some code that iterates through all non-walls
    let mut processed_grid = level.map.grid.clone();
    for r in 0..processed_grid.rows() {
        for c in 0..processed_grid.cols(r) {
            if !visited[(r, c)] {
                processed_grid[(r, c)] = MapCell::Wall;
            }
        }
    }

    if reachable_boxes.len() != reachable_goals.len() {
        return Err(SolverErr::BoxesGoals);
    }

    // only 254 because 255 is used to represent empty in expand_{move,push}
    if reachable_boxes.len() > 254 {
        return Err(SolverErr::TooMany);
    }

    let processed_map = Map::new(processed_grid, reachable_goals);
    let clean_state = State::new(level.state.player_pos, reachable_boxes);
    let dead_ends = find_dead_ends(&processed_map);
    Ok(SolverLevel::new(processed_map, clean_state, dead_ends))
}

fn search<Expand, Heuristic>(level: &SolverLevel, method: Method, print_status: bool,
                             expand: Expand, heuristic: Heuristic) -> SolverOk
    where Expand: Fn(&Map, &State, &Vec2d<bool>) -> Vec<State>,
          Heuristic: Fn(&Map, &State) -> i32
{
    let mut stats = Stats::new();

    let mut to_visit = BinaryHeap::new();
    let mut closed = HashSet::new();
    let mut prev = HashMap::new();

    let h = heuristic(&level.map, &level.state);
    let start = SearchState {
        state: level.state.clone(),
        prev: None,
        dist: 0,
        h: h,
    };
    stats.add_created(&start);
    to_visit.push(start);
    while let Some(current) = to_visit.pop() {
        if closed.contains(&current.state) {
            stats.add_reached_duplicate(&current);
            continue;
        }
        if stats.add_unique_visited(&current) && print_status {
            println!("Visited new depth: {}", current.dist);
            println!("{:?}", stats);
        }

        // insert here and not as soon as we discover it
        // otherwise we overwrite the shortest path with longer ones
        if let Some(p) = current.prev {
            prev.insert(current.state.clone(), p.clone());
        }

        if solved(&level.map, &current.state) {
            return SolverOk::new(
                Some(backtrack_path(&prev, &current.state)),
                stats, method);
        }

        for neighbor_state in expand(&level.map, &current.state, &level.dead_ends) {
            // TODO this could probably be optimized a bit by allocating on the heap
            // and storing references only (to current state, neighbor state is always different)

            // insert and then ignore duplicates
            let h = heuristic(&level.map, &neighbor_state);
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

    SolverOk::new(None, stats, method)
}

fn find_dead_ends(map: &Map) -> Vec2d<bool> {
    let mut dead_ends = map.grid.create_scratchpad(false);

    for r in 0..map.grid.rows() {
        'cell: for c in 0..map.grid.cols(r) {
            let box_pos = Pos::new(r, c);
            if map.grid[box_pos] == MapCell::Wall {
                //print!("w");
                continue;
            }

            for &player_pos in box_pos.neighbors().iter() {
                if map.grid[player_pos] == MapCell::Wall {
                    continue;
                }

                let fake_state = State {
                    player_pos,
                    boxes: vec![box_pos],
                };
                let fake_level = SolverLevel::new(map.clone(), fake_state, dead_ends.clone());
                if let Some(_) = search(&fake_level, Method::Pushes, false,
                                        expand_push, heuristic_push).path_states {
                    //print!("cont");
                    continue 'cell; // need to find only one solution
                }
            }
            dead_ends[(r, c)] = true; // no solution from any direction
        }
    }

    dead_ends
}

fn heuristic_push(map: &Map, state: &State) -> i32 {
    // less is better

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

fn heuristic_move(map: &Map, state: &State) -> i32 {
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

fn solved(map: &Map, state: &State) -> bool {
    // to detect dead ends, this has to test all boxes are on a goal, not that all goals have a box
    for pos in &state.boxes {
        if map.grid[*pos] != MapCell::Goal {
            return false;
        }
    }
    true
}

fn expand_push(map: &Map, state: &State, dead_ends: &Vec2d<bool>) -> Vec<State> {
    let mut new_states = Vec::new();

    let mut box_grid = map.grid.create_scratchpad(255);
    for (i, b) in state.boxes.iter().enumerate() {
        box_grid[*b] = i as u8;
    }

    // find each box and each direction from which it can be pushed
    let mut reachable = map.grid.create_scratchpad(false);
    reachable[state.player_pos] = true;
    let mut to_visit = vec![state.player_pos];

    while !to_visit.is_empty() {
        let player_pos = to_visit.pop().unwrap();
        for &dir in DIRECTIONS.iter() {
            let new_player_pos = player_pos + dir;
            let box_index = box_grid[new_player_pos];
            if box_index < 255 {
                // new_pos has a box
                let push_dest = new_player_pos + dir;
                if box_grid[push_dest] == 255
                    && map.grid[push_dest] != MapCell::Wall
                    && !dead_ends[push_dest] { // TODO could we abuse dead_ends to avoid wall detection when pushing?
                    // new state to explore

                    let mut new_boxes = state.boxes.clone();
                    new_boxes[box_index as usize] = push_dest;
                    // TODO normalize player pos
                    new_states.push(State::new(new_player_pos, new_boxes));
                }
            } else if map.grid[new_player_pos] != MapCell::Wall
                && !reachable[new_player_pos] {
                // new_pos is empty and not yet visited
                reachable[new_player_pos] = true;
                to_visit.push(new_player_pos);
            }
        }
    }

    new_states
}

fn expand_move(map: &Map, state: &State, dead_ends: &Vec2d<bool>) -> Vec<State> {
    let mut new_states = Vec::new();

    let mut box_grid = map.grid.create_scratchpad(255);
    for (i, b) in state.boxes.iter().enumerate() {
        box_grid[*b] = i as u8;
    }

    for &dir in DIRECTIONS.iter() {
        let new_player_pos = state.player_pos + dir;
        if map.grid[new_player_pos] != MapCell::Wall {
            let box_index = box_grid[new_player_pos];
            let push_dest = new_player_pos + dir;

            if box_index == 255 {
                // step
                new_states.push(State::new(new_player_pos, state.boxes.clone()));
            } else if box_grid[push_dest] == 255
                && map.grid[push_dest] != MapCell::Wall
                && dead_ends[push_dest] == false {
                // push

                let mut new_boxes = state.boxes.clone();
                new_boxes[box_index as usize] = push_dest;
                new_states.push(State::new(new_player_pos, new_boxes));
            }
        }
    }

    new_states
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unreachable_boxes() {
        let level = r"
########
#@$.#$.#
########
";
        let level = level.parse().unwrap();
        assert_eq!(process_map(&level).unwrap_err(), SolverErr::UnreachableBoxes);
    }

    #[test]
    fn test_dead_ends() {
        let level = r"
#####
##@##
##$##
#  .#
#####";
        let level = level.parse().unwrap();
        let solver_level = process_map(&level).unwrap();
        let expected = r"
00000
00100
00100
01000
00000
".trim_left();
        assert_eq!(solver_level.dead_ends.to_string(), expected);
    }

    #[test]
    fn test_expand_push() {
        // at some point expand detected some moves multiple times - should not happen again

        let level = r"
<><><><><><>
<> _B_B_ _<>
<>B_B     <>
<><>P     <>
<><>  B <><>
<>      <>
<><>    <>
<><><><><>
";
        let level = level.parse().unwrap();
        let solver_level = process_map(&level).unwrap();
        let neighbor_states = expand_push(&solver_level.map, &solver_level.state, &solver_level.dead_ends);
        assert_eq!(neighbor_states.len(), 2);
    }

    #[test]
    fn test_expand_move1() {
        let level = r"
 ####
# $  #
# @$*#
# $  #
# ...#
 ####
";
        let level = level.parse().unwrap();
        let solver_level = process_map(&level).unwrap();
        let neighbor_states = expand_move(&solver_level.map, &solver_level.state, &solver_level.dead_ends);
        assert_eq!(neighbor_states.len(), 2);
    }

    #[test]
    fn test_expand_move2() {
        let level = r"
 ####
#    #
# @ *#
# $  #
#   .#
 ####
";
        let level = level.parse().unwrap();
        let solver_level = process_map(&level).unwrap();
        let neighbor_states = expand_move(&solver_level.map, &solver_level.state, &solver_level.dead_ends);
        assert_eq!(neighbor_states.len(), 4);
    }
}
