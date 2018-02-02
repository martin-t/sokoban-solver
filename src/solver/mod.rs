pub mod a_star;
mod level;

use std::fmt;
use std::fmt::{Display, Formatter};
use std::collections::{BinaryHeap, HashMap, HashSet};

use data::{Pos, Dir};
use level::{Level, Map, Vec2d, MapCell, State};

use self::a_star::{SearchState, Stats};
use self::level::SolverLevel;

#[derive(Debug, PartialEq)]
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
            SolverErr::TooLarge => write!(f, "Map oo large"),
            SolverErr::IncompleteBorder => write!(f, "Player can exit the level because of missing border"),
            SolverErr::UnreachableBoxes => write!(f, "Boxes that are not on goal but can't be reached"),
            SolverErr::UnreachableGoals => write!(f, "Goals that don't have a box but can't be reached"),
            //SolverErr::UnreachableRemover => write!(f, "Remover is not reachable"),
            SolverErr::TooMany => write!(f, "More than 255 reachable boxes or goals"),
            SolverErr::BoxesGoals => write!(f, "Different number of reachable boxes and goals"),
        }
    }
}

pub struct SolverOk {
    // TODO probably wanna use Dirs or Moves eventually
    pub path_states: Option<Vec<State>>,
    pub stats: Stats,
}

impl SolverOk {
    fn new(path_states: Option<Vec<State>>, stats: Stats) -> Self {
        Self { path_states, stats }
    }
}

const UP: Dir = Dir { r: -1, c: 0 };
const RIGHT: Dir = Dir { r: 0, c: 1 };
const DOWN: Dir = Dir { r: 1, c: 0 };
const LEFT: Dir = Dir { r: 0, c: -1 };
const DIRECTIONS: [Dir; 4] = [UP, RIGHT, DOWN, LEFT];

pub fn solve(level: &Level, print_status: bool) -> Result<SolverOk, SolverErr> {
    let solver_level = processed_map(level)?;
    Ok(search(&solver_level, print_status))
}

pub fn processed_map(level: &Level) -> Result<SolverLevel, SolverErr> {
    // Only guarantees we have here is the player exists and therefore map is at least 1x1.
    // Do some more low level checking so we can omit some checks later.

    if level.map.grid.0.len() > 255 || level.map.grid.0[0].len() > 255 {
        return Err(SolverErr::TooLarge);
    }

    let mut to_visit = vec![(level.state.player_pos.r, level.state.player_pos.c)];
    let mut visited = level.map.grid.create_scratch_map(false).0;

    while !to_visit.is_empty() {
        let (r, c) = to_visit.pop().unwrap();
        visited[r as usize][c as usize] = true;

        let neighbors = [(r + 1, c), (r - 1, c), (r, c + 1), (r, c - 1)];
        for &(nr, nc) in neighbors.iter() {
            // this is the only place we need to check bounds
            // everything after that will be surrounded by walls
            // TODO make sure we're not wasting time bounds checking anywhere else
            if nr < 0
                || nc < 0
                || nr as usize >= level.map.grid.0.len()
                || nc as usize >= level.map.grid.0[nr as usize].len() {
                // we got out of bounds without hitting a wall
                return Err(SolverErr::IncompleteBorder);
            }

            if !visited[nr as usize][nc as usize] && level.map.grid.0[nr as usize][nc as usize] != MapCell::Wall {
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
        if visited[r][c] {
            reachable_boxes.push(pos);
        } else if !level.map.goals.contains(&pos) {
            return Err(SolverErr::UnreachableBoxes);
        }
    }
    for &pos in level.map.goals.iter() {
        let (r, c) = (pos.r as usize, pos.c as usize);
        if visited[r][c] {
            reachable_goals.push(pos);
        } else if !level.state.boxes.contains(&pos) {
            return Err(SolverErr::UnreachableGoals);
        }
    }

    // FIXME maybe do this first and use it instead of visited when detecting reachability in specialized fns?
    // to avoid errors with some code that iterates through all non-walls
    let mut processed_grid = level.map.grid.clone();
    for r in 0..processed_grid.0.len() {
        for c in 0..processed_grid.0[r].len() {
            if !visited[r][c] {
                processed_grid.0[r][c] = MapCell::Wall;
            }
        }
    }

    if reachable_boxes.len() != reachable_goals.len() {
        return Err(SolverErr::BoxesGoals);
    }

    if reachable_boxes.len() > 255 {
        return Err(SolverErr::TooMany);
    }

    let processed_map = Map::new(processed_grid, reachable_goals);
    let clean_state = State::new(level.state.player_pos, reachable_boxes);
    let dead_ends = find_dead_ends(&processed_map);
    Ok(SolverLevel::new(processed_map, clean_state, dead_ends))
}

pub fn search(level: &SolverLevel, print_status: bool) -> SolverOk
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
                stats);
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

    SolverOk::new(None, stats)
}

fn expand(map: &Map, state: &State, dead_ends: &Vec2d<bool>) -> Vec<State> {
    //expand_move(map, state)
    expand_push(map, state, dead_ends)
}

fn heuristic(map: &Map, state: &State) -> i32 {
    //heuristic_move(map, state)
    heuristic_push(map, state)
}

fn find_dead_ends(map: &Map) -> Vec2d<bool> {
    // TODO test case
    // #####
    // ##@##
    // ##$##
    // #  .#
    // #####

    let mut dead_ends = map.grid.create_scratch_map(false);

    for r in 0..map.grid.0.len() {
        // TODO make .0 private
        'cell: for c in 0..map.grid.0[r].len() {
            let box_pos = Pos::new(r, c);
            if map.grid[box_pos] == MapCell::Wall {
                //print!("w");
                continue;
            }

            for dir in DIRECTIONS.iter() {
                let player_pos = box_pos + *dir;
                if map.grid[player_pos] == MapCell::Wall {
                    continue;
                }

                let fake_state = State {
                    player_pos,
                    boxes: vec![box_pos],
                };
                let fake_level = SolverLevel::new(map.clone(), fake_state, dead_ends.clone());
                if let Some(_) = search(&fake_level, false).path_states {
                    //print!("cont");
                    continue 'cell; // need to find only one solution
                }
            }
            dead_ends.0[r][c] = true; // no solution from any direction
        }
    }

    dead_ends
}

fn heuristic_push(map: &Map, state: &State) -> i32 {
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

/*#[allow(unused)]
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
}*/

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

    let mut box_grid = map.grid.create_scratch_map(255);
    for (i, b) in state.boxes.iter().enumerate() {
        box_grid[*b] = i as u8;
    }

    // find each box and each direction from which it can be pushed
    let mut reachable = map.grid.create_scratch_map(false);
    reachable[state.player_pos] = true;
    let mut to_visit = vec![state.player_pos];

    while !to_visit.is_empty() {
        let player_pos = to_visit.pop().unwrap();
        for &dir in DIRECTIONS.iter() {
            let move_pos = player_pos + dir;
            if map.grid[move_pos] == MapCell::Wall {
                continue;
            }

            let box_index = box_grid[move_pos];
            if box_index < 255 {
                // new_pos has a box
                let push_dest = move_pos + dir;
                if box_grid[push_dest] == 255
                    && map.grid[push_dest] != MapCell::Wall && !dead_ends[push_dest] {
                    // new state to explore

                    let mut new_boxes = state.boxes.clone();
                    new_boxes[box_index as usize] = push_dest;
                    // TODO normalize player pos
                    new_states.push(State::new(move_pos, new_boxes));
                }
            } else if !reachable[move_pos] {
                // new_pos is empty and not yet visited
                reachable[move_pos] = true;
                to_visit.push(move_pos);
            }
        }
    }

    new_states
}

/*#[allow(unused)]
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
}*/

#[cfg(test)]
mod tests {
    use super::*;
    use data::Format;
    use parser;

    #[test]
    fn test_unreachable_boxes() {
        let level = r"
########
#@$.#$.#
########
";
        let level = parser::parse(level, Format::Xsb).unwrap();
        assert_eq!(processed_map(&level).unwrap_err(), SolverErr::UnreachableBoxes);
    }

    #[test]
    fn test_expand_push() {
        // at some point expand could detect some moves multiple times
        // TODO test unsolvable map and total numbers of generated states

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
        let level = parser::parse(&level, Format::Custom).unwrap();
        let solver_level = processed_map(&level).unwrap();
        let neighbor_states = expand(&solver_level.map, &solver_level.state, &solver_level.dead_ends);
        assert_eq!(neighbor_states.len(), 2);
    }
}
