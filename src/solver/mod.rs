crate mod a_star;
mod level;

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

use fnv::FnvHashMap;

use config::Method;
use data::{MapCell, Pos, State, DIRECTIONS, MAX_BOXES};
use level::Level;
use map::GoalMap;
use vec2d::Vec2d;
use Solve;

use self::a_star::{SearchNode, Stats};
use self::level::SolverLevel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolverErr {
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
            SolverErr::IncompleteBorder => write!(f, "Incomplete border"),
            SolverErr::UnreachableBoxes => write!(
                f,
                "Unreachable boxes - some boxes are not on goal but can't be reached"
            ),
            SolverErr::UnreachableGoals => write!(
                f,
                "Unreachable goals - some goals don't have a box but can't be reached"
            ),
            //SolverErr::UnreachableRemover => write!(f, "Remover is not reachable"),
            SolverErr::TooMany => write!(f, "More than 254 reachable boxes or goals"),
            SolverErr::BoxesGoals => write!(f, "Different number of reachable boxes and goals"),
        }
    }
}

impl Error for SolverErr {}

pub struct SolverOk {
    // TODO probably wanna use Dirs or Moves eventually
    pub path_states: Option<Vec<State>>,
    pub stats: Stats,
    crate method: Method,
}

impl SolverOk {
    fn new(path_states: Option<Vec<State>>, stats: Stats, method: Method) -> Self {
        Self {
            path_states,
            stats,
            method,
        }
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

impl Solve for Level {
    fn solve(&self, method: Method, print_status: bool) -> Result<SolverOk, SolverErr> {
        solve(self, method, print_status)
    }
}

fn solve(level: &Level, method: Method, print_status: bool) -> Result<SolverOk, SolverErr> {
    debug!("Processing level...");
    let solver_level = process_level(level)?;
    debug!("Processed level");
    match method {
        Method::Moves => Ok(search(
            &solver_level,
            method,
            print_status,
            expand_move,
            heuristic_move,
        )),
        Method::Pushes => Ok(search(
            &solver_level,
            method,
            print_status,
            expand_push,
            heuristic_push,
        )),
    }
}

fn process_level(level: &Level) -> Result<SolverLevel, SolverErr> {
    // Guarantees we have here:
    // - the player exists and therefore map is at least 1x1.
    // - rows and cols is <= 255
    // Do some more low level checking so we can omit some checks later.

    // make sure the level is surrounded by wall
    let mut to_visit = vec![level.state.player_pos];
    let mut visited = level.map.grid.create_scratchpad(false);

    while !to_visit.is_empty() {
        let cur = to_visit.pop().unwrap();
        visited[cur] = true;

        let (r, c) = (i32::from(cur.r), i32::from(cur.c));
        let neighbors = [(r + 1, c), (r - 1, c), (r, c + 1), (r, c - 1)];
        for &(nr, nc) in &neighbors {
            // this is the only place we need to check bounds (using signed types)
            // everything after that will be surrounded by walls
            if nr < 0
                || nc < 0
                || nr >= i32::from(level.map.grid.rows())
                || nc >= i32::from(level.map.grid.cols())
            {
                // we got out of bounds without hitting a wall
                return Err(SolverErr::IncompleteBorder);
            }

            let new_pos = Pos::new(nr as u8, nc as u8);
            if !visited[new_pos] && level.map.grid[new_pos] != MapCell::Wall {
                to_visit.push(new_pos);
            }
        }
    }

    // TODO move into specialized function when removers work
    /*if let Some(pos) = remover {
        if !visited.0[pos.r as usize][pos.c as usize] {
            return Err(SolverErr::UnreachableRemover);
        }
    }*/

    // make sure all relevant game elements are reachable
    let mut reachable_goals = Vec::new();
    let mut reachable_boxes = Vec::new();
    for &pos in &level.state.boxes {
        if visited[pos] {
            reachable_boxes.push(pos);
        } else if !level.map.goals.contains(&pos) {
            return Err(SolverErr::UnreachableBoxes);
        }
    }
    for &pos in &level.map.goals {
        if visited[pos] {
            reachable_goals.push(pos);
        } else if !level.state.boxes.contains(&pos) {
            return Err(SolverErr::UnreachableGoals);
        }
    }

    // TODO maybe do this first and use it instead of visited when detecting reachability in specialized fns?
    // make sure all non-reachable cells are walls
    // to avoid errors with some code that iterates through all non-walls
    let mut processed_grid = level.map.grid.clone();
    for r in 0..processed_grid.rows() {
        for c in 0..processed_grid.cols() {
            let pos = Pos::new(r, c);
            if !visited[pos] {
                processed_grid[pos] = MapCell::Wall;
            }
        }
    }

    if reachable_boxes.len() != reachable_goals.len() {
        return Err(SolverErr::BoxesGoals);
    }

    // only 254 because 255 is used to represent empty in expand_{move,push}
    if reachable_boxes.len() > MAX_BOXES {
        return Err(SolverErr::TooMany);
    }

    let processed_map = GoalMap::new(processed_grid, reachable_goals);
    let clean_state = State::new(level.state.player_pos, reachable_boxes);
    let dead_ends = find_dead_ends(&processed_map);
    Ok(SolverLevel::new(processed_map, clean_state, dead_ends))
}

fn search<Expand, Heuristic>(
    level: &SolverLevel,
    method: Method,
    print_status: bool,
    expand: Expand,
    heuristic: Heuristic,
) -> SolverOk
where
    Expand: Fn(&GoalMap, &State, &Vec2d<bool>) -> Vec<State>,
    Heuristic: Fn(&GoalMap, &State) -> i16,
{
    // TODO get rid of all the cloning

    debug!("Search called");

    let mut stats = Stats::new();

    let mut to_visit = BinaryHeap::new();
    let mut prevs = FnvHashMap::default();

    let start = SearchNode::new(
        level.state.clone(),
        None,
        0,
        heuristic(&level.map, &level.state),
    );
    stats.add_created(&start);
    to_visit.push(Reverse(start));

    while let Some(Reverse(cur_node)) = to_visit.pop() {
        if prevs.contains_key(&cur_node.state) {
            stats.add_reached_duplicate(&cur_node);
            continue;
        }
        if stats.add_unique_visited(&cur_node) && print_status {
            println!("Visited new depth: {}", cur_node.dist);
            println!("{:?}", stats);
        }

        // insert here and not as soon as we discover it
        // otherwise we overwrite the shortest path with longer ones
        if let Some(p) = cur_node.prev {
            prevs.insert(cur_node.state.clone(), p.clone());
        } else {
            // initial state has no prev - hack to avoid Option
            prevs.insert(cur_node.state.clone(), cur_node.state.clone());
        }

        if solved(&level.map, &cur_node.state) {
            debug!("Solved, backtracking path");
            return SolverOk::new(Some(backtrack_path(&prevs, &cur_node.state)), stats, method);
        }

        for neighbor_state in expand(&level.map, &cur_node.state, &level.dead_ends) {
            // insert and then ignore duplicates
            let h = heuristic(&level.map, &neighbor_state);
            let next_node = SearchNode::new(
                neighbor_state,
                Some(cur_node.state.clone()),
                cur_node.dist + 1,
                h,
            );
            stats.add_created(&next_node);
            to_visit.push(Reverse(next_node));
        }
    }

    SolverOk::new(None, stats, method)
}

fn find_dead_ends(map: &GoalMap) -> Vec2d<bool> {
    let mut dead_ends = map.grid.create_scratchpad(false);

    // mark walls as dead ends first because expand_push needs it
    for r in 0..map.grid.rows() {
        for c in 0..map.grid.cols() {
            let pos = Pos::new(r, c);
            if map.grid[pos] == MapCell::Wall {
                dead_ends[pos] = true;
            }
        }
    }

    // put box on every position and try to get it to a goal
    for r in 0..map.grid.rows() {
        'cell: for c in 0..map.grid.cols() {
            let box_pos = Pos::new(r, c);
            if dead_ends[box_pos] {
                continue;
            }

            for &player_pos in &box_pos.neighbors() {
                if map.grid[player_pos] == MapCell::Wall {
                    continue;
                }

                let fake_state = State {
                    player_pos,
                    boxes: vec![box_pos],
                };
                let fake_level = SolverLevel::new(map.clone(), fake_state, dead_ends.clone());
                if search(
                    &fake_level,
                    Method::Pushes,
                    false,
                    expand_push,
                    heuristic_push,
                ).path_states
                    .is_some()
                {
                    continue 'cell; // need to find only one solution
                }
            }
            dead_ends[box_pos] = true; // no solution from any direction
        }
    }

    dead_ends
}

fn heuristic_push(map: &GoalMap, state: &State) -> i16 {
    // less is better

    let mut goal_dist_sum = 0;
    for box_pos in &state.boxes {
        let mut min = i16::max_value();
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

fn heuristic_move(map: &GoalMap, state: &State) -> i16 {
    // less is better

    let mut closest_box = i16::max_value();
    for box_pos in &state.boxes {
        let dist = state.player_pos.dist(*box_pos);
        if dist < closest_box {
            closest_box = dist;
        }
    }

    let mut goal_dist_sum = 0;
    for box_pos in &state.boxes {
        let mut min = i16::max_value();
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

fn backtrack_path(prevs: &FnvHashMap<State, State>, final_state: &State) -> Vec<State> {
    let mut ret = Vec::new();
    let mut state = final_state;
    loop {
        ret.push(state.clone());
        let prev = &prevs[&state];
        if prev == state {
            ret.reverse();
            return ret;
        }
        state = prev;
    }
}

fn solved(map: &GoalMap, state: &State) -> bool {
    // to detect dead ends, this has to test all boxes are on a goal, not that all goals have a box
    for pos in &state.boxes {
        if map.grid[*pos] != MapCell::Goal {
            return false;
        }
    }
    true
}

fn expand_push(map: &GoalMap, state: &State, dead_ends: &Vec2d<bool>) -> Vec<State> {
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
        for &dir in &DIRECTIONS {
            let new_player_pos = player_pos + dir;
            let box_index = box_grid[new_player_pos];
            if box_index < 255 {
                // new_pos has a box
                let push_dest = new_player_pos + dir;
                if box_grid[push_dest] == 255
                    // dead_end == true means either wall or dead end
                    // might not actually be faster (no diff in benches) but probably can't hurt
                    && !dead_ends[push_dest]
                {
                    // new state to explore

                    let mut new_boxes = state.boxes.clone();
                    new_boxes[box_index as usize] = push_dest;
                    // TODO normalize player pos - detect duplicates during expansion?
                    // otherwise we'd have to generate reachable twice or save them as part of state
                    new_states.push(State::new(new_player_pos, new_boxes));
                }
            } else if map.grid[new_player_pos] != MapCell::Wall && !reachable[new_player_pos] {
                // new_pos is empty and not yet visited
                reachable[new_player_pos] = true;
                to_visit.push(new_player_pos);
            }
        }
    }

    new_states
}

fn expand_move(map: &GoalMap, state: &State, dead_ends: &Vec2d<bool>) -> Vec<State> {
    let mut new_states = Vec::new();

    let mut box_grid = map.grid.create_scratchpad(255);
    for (i, b) in state.boxes.iter().enumerate() {
        box_grid[*b] = i as u8;
    }

    for &dir in &DIRECTIONS {
        let new_player_pos = state.player_pos + dir;
        if map.grid[new_player_pos] != MapCell::Wall {
            let box_index = box_grid[new_player_pos];
            let push_dest = new_player_pos + dir;

            if box_index == 255 {
                // step
                new_states.push(State::new(new_player_pos, state.boxes.clone()));
            } else if box_grid[push_dest] == 255
                && map.grid[push_dest] != MapCell::Wall
                && !dead_ends[push_dest]
            {
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
    fn unreachable_boxes() {
        let level = r"
########
#@$.#$.#
########
";
        let level = level.parse().unwrap();
        assert_eq!(
            process_level(&level).unwrap_err(),
            SolverErr::UnreachableBoxes
        );
    }

    #[test]
    fn dead_ends() {
        let level = r"
#####
##@##
##$##
#  .#
#####";
        let level = level.parse().unwrap();
        let solver_level = process_level(&level).unwrap();
        let expected = r"
11111
11111
11111
11001
11111
"
            .trim_left();
        assert_eq!(solver_level.dead_ends.to_string(), expected);
    }

    #[test]
    fn expand_push1() {
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
        let solver_level = process_level(&level).unwrap();
        let neighbor_states = expand_push(
            &solver_level.map,
            &solver_level.state,
            &solver_level.dead_ends,
        );
        assert_eq!(neighbor_states.len(), 2);
    }

    #[test]
    fn expand_move1() {
        let level = r"
 ####
# $  #
# @$*#
# $  #
# ...#
 ####
";
        let level = level.parse().unwrap();
        let solver_level = process_level(&level).unwrap();
        let neighbor_states = expand_move(
            &solver_level.map,
            &solver_level.state,
            &solver_level.dead_ends,
        );
        assert_eq!(neighbor_states.len(), 2);
    }

    #[test]
    fn expand_move2() {
        let level = r"
 ####
#    #
# @ *#
# $  #
#   .#
 ####
";
        let level = level.parse().unwrap();
        let solver_level = process_level(&level).unwrap();
        let neighbor_states = expand_move(
            &solver_level.map,
            &solver_level.state,
            &solver_level.dead_ends,
        );
        assert_eq!(neighbor_states.len(), 4);
    }
}
