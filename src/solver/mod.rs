crate mod a_star;
mod backtracking;

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

use fnv::FnvHashMap; // using rustc-hash gives the same results, maybe bench again when able to solve levels with many boxes
use log::{debug, log};
use typed_arena::Arena;

use crate::config::Method;
use crate::data::{MapCell, Pos, DIRECTIONS, MAX_BOXES};
use crate::level::Level;
use crate::map::GoalMap;
use crate::moves::Moves;
use crate::state::State;
use crate::vec2d::Vec2d;
use crate::Solve;

use self::a_star::{SearchNode, Stats};

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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
            SolverErr::TooMany => write!(f, "More than {} reachable boxes or goals", MAX_BOXES),
            SolverErr::BoxesGoals => write!(f, "Different number of reachable boxes and goals"),
        }
    }
}

impl Error for SolverErr {}

#[derive(Debug)]
pub struct SolverOk {
    pub moves: Option<Moves>,
    pub stats: Stats,
    crate method: Method,
}

impl SolverOk {
    fn new(moves: Option<Moves>, stats: Stats, method: Method) -> Self {
        Self {
            moves,
            stats,
            method,
        }
    }
}

impl Solve for Level {
    fn solve(&self, method: Method, print_status: bool) -> Result<SolverOk, SolverErr> {
        debug!("Processing level...");
        let solver = Solver::new(self)?;
        debug!("Processed level");
        match method {
            Method::MoveOptimal => {
                Ok(solver.search(method, print_status, expand_move, heuristic_move))
            }
            Method::PushOptimal => {
                Ok(solver.search(method, print_status, expand_push, heuristic_push))
            }
        }
    }
}

#[derive(Debug)]
struct StaticData {
    map: GoalMap,
    distances: Vec2d<Option<u16>>,
}

#[derive(Debug)]
struct Solver {
    // this should remain private given i might use unsafe to optimize things
    // and some of the values must be correct to avoid out of bounds array access
    sd: StaticData,
    initial_state: State,
}

impl Solver {
    fn new(level: &Level) -> Result<Solver, SolverErr> {
        // Guarantees we have here:
        // - the player exists and therefore map is at least 1x1.
        // - rows and cols is <= 255
        // Do some more low level checking so we can omit some checks later.

        // make sure the level is surrounded by wall
        let mut to_visit = vec![level.state.player_pos];
        let mut visited = level.map.grid.scratchpad();

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

        // only 255 boxes max because 255 (index of the 256th box) is used to represent empty in expand_{move,push}
        if reachable_boxes.len() > MAX_BOXES {
            return Err(SolverErr::TooMany);
        }

        let processed_map = GoalMap::new(processed_grid, reachable_goals);
        let clean_state = State::new(level.state.player_pos, reachable_boxes);
        let distances = find_distances(&processed_map);
        Ok(Solver {
            sd: StaticData {
                map: processed_map,
                distances,
            },
            initial_state: clean_state,
        })
    }

    fn search<Expand, Heuristic>(
        &self,
        method: Method,
        print_status: bool,
        expand: Expand,
        heuristic: Heuristic,
    ) -> SolverOk
    where
        Expand: for<'a> Fn(&StaticData, &State, &'a Arena<State>) -> Vec<&'a State>,
        Heuristic: Fn(&StaticData, &State) -> u16,
    {
        debug!("Search called");

        let mut stats = Stats::new();
        let states = Arena::new();

        let mut to_visit = BinaryHeap::new();
        let mut prevs = FnvHashMap::default();

        let start = SearchNode::new(
            &self.initial_state,
            None,
            0,
            heuristic(&self.sd, &self.initial_state),
        );
        stats.add_created(&start);
        to_visit.push(Reverse(start));

        //let mut counter = 0;
        while let Some(Reverse(cur_node)) = to_visit.pop() {
            /*counter += 1;
            if counter % 10_000 == 0 {
                use crate::map::Map;
                println!("prevs: {}, to_visit: {}", prevs.len(), to_visit.len());
                println!("{}", self.map.xsb_with_state(&cur_node.state));
            }*/

            if prevs.contains_key(cur_node.state) {
                stats.add_reached_duplicate(&cur_node);
                continue;
            }
            if stats.add_unique_visited(&cur_node) && print_status {
                println!("Visited new depth: {}", cur_node.dist);
                println!("{:?}", stats);
            }

            // insert when expanding and not when generating
            // otherwise we might overwrite the shortest path with longer ones
            if let Some(p) = cur_node.prev {
                prevs.insert(cur_node.state, p);
            } else {
                // initial state has no prev - hack to avoid Option
                prevs.insert(cur_node.state, cur_node.state);
            }

            if cur_node.cost == cur_node.dist {
                // heuristic is 0 so level is solved
                debug!("Solved, backtracking path");
                return SolverOk::new(
                    Some(backtracking::reconstruct_moves(
                        &self.sd.map,
                        &prevs,
                        &cur_node.state,
                    )),
                    stats,
                    method,
                );
            }

            for neighbor_state in expand(&self.sd, &cur_node.state, &states) {
                // Insert everything and ignore duplicates when popping. This wastes memory
                // but when I filter them out here using a HashMap, push-optimal/boxxle2/4 becomes 8x slower
                // and generates much more states (although push-optimal/original/1 becomes about 2x faster).
                // I might have done something wrong, might wanna try again when i have better debugging tools
                // to look at the generated states.

                // Also might wanna try https://crates.io/crates/priority-queue for changing priorities
                // instead of adding duplicates.

                let h = heuristic(&self.sd, neighbor_state);
                let next_node =
                    SearchNode::new(neighbor_state, Some(&cur_node.state), cur_node.dist + 1, h);
                stats.add_created(&next_node);
                to_visit.push(Reverse(next_node));
            }
        }

        SolverOk::new(None, stats, method)
    }
}

fn find_distances(map: &GoalMap) -> Vec2d<Option<u16>> {
    let mut distances = map.grid.scratchpad();

    // some functions don't check walls but only dead ends
    let mut fake_distances = map.grid.scratchpad();
    for r in 0..map.grid.rows() {
        for c in 0..map.grid.cols() {
            let pos = Pos::new(r, c);
            if map.grid[pos] != MapCell::Wall {
                fake_distances[pos] = Some(0);
            }
        }
    }

    // put box on every position and try to get it to the nearest goal
    for r in 0..map.grid.rows() {
        for c in 0..map.grid.cols() {
            let box_pos = Pos::new(r, c);
            if map.grid[box_pos] == MapCell::Wall {
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
                let fake_solver = Solver {
                    sd: StaticData {
                        map: map.clone(),
                        distances: fake_distances.clone(),
                    },
                    initial_state: fake_state,
                };

                // using manhattan dist here because the fake solver needs a heuristic
                // that only reports 0 when the level is solved
                let moves = fake_solver
                    .search(
                        Method::PushOptimal,
                        false,
                        expand_push,
                        heuristic_push_manhattan,
                    ).moves;
                if let Some(moves) = moves {
                    let new_dist = moves.push_cnt() as u16; // dist can't be larger than MAX_SIZE^2
                    match distances[box_pos] {
                        None => distances[box_pos] = Some(new_dist),
                        Some(cur_min_dist) => if new_dist < cur_min_dist {
                            distances[box_pos] = Some(new_dist);
                        },
                    }
                }
            }
        }
    }

    distances
}

fn heuristic_push_manhattan(sd: &StaticData, state: &State) -> u16 {
    // less is better

    let mut goal_dist_sum = 0;

    for box_pos in &state.boxes {
        let mut min = u16::max_value();
        for goal in &sd.map.goals {
            let dist = box_pos.dist(*goal);
            if dist < min {
                min = dist;
            }
        }
        goal_dist_sum += min;
    }

    goal_dist_sum
}

fn heuristic_push(sd: &StaticData, state: &State) -> u16 {
    // less is better

    let mut goal_dist_sum = 0;

    for &box_pos in &state.boxes {
        goal_dist_sum += sd.distances[box_pos].unwrap();
    }

    goal_dist_sum
}

fn heuristic_move(sd: &StaticData, state: &State) -> u16 {
    // less is better

    let mut closest_box = u16::max_value();
    for box_pos in &state.boxes {
        let dist = state.player_pos.dist(*box_pos);
        if dist < closest_box {
            closest_box = dist;
        }
    }

    // -1 because it should be the distance until being able to push the box
    // and when all boxes are on goals, the heuristic should be 0
    closest_box - 1 + heuristic_push(sd, state)
}

fn expand_push<'a>(sd: &StaticData, state: &State, arena: &'a Arena<State>) -> Vec<&'a State> {
    let mut new_states = Vec::new();

    let mut box_grid = sd.map.grid.scratchpad_with_default(255u8);
    for (i, b) in state.boxes.iter().enumerate() {
        box_grid[*b] = i as u8;
    }

    // find each box and each direction from which it can be pushed
    let mut reachable = sd.map.grid.scratchpad();
    reachable[state.player_pos] = true;

    // Vec is noticeably faster than VecDeque on some levels
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
                    // either wall or dead end
                    && sd.distances[push_dest].is_some()
                {
                    // new state to explore

                    let mut new_boxes = state.boxes.clone();
                    new_boxes[box_index as usize] = push_dest;
                    // TODO normalize player pos - detect duplicates during expansion?
                    // otherwise we'd have to generate reachable twice or save them as part of state
                    let new_state = arena.alloc(State::new(new_player_pos, new_boxes));
                    new_states.push(&*new_state);
                }
            } else if sd.map.grid[new_player_pos] != MapCell::Wall && !reachable[new_player_pos] {
                // new_pos is empty and not yet visited
                reachable[new_player_pos] = true;
                to_visit.push(new_player_pos);
            }
        }
    }

    new_states
}

fn expand_move<'a>(sd: &StaticData, state: &State, arena: &'a Arena<State>) -> Vec<&'a State> {
    let mut new_states = Vec::new();

    let mut box_grid = sd.map.grid.scratchpad_with_default(255u8);
    for (i, b) in state.boxes.iter().enumerate() {
        box_grid[*b] = i as u8;
    }

    for &dir in &DIRECTIONS {
        let new_player_pos = state.player_pos + dir;
        if sd.map.grid[new_player_pos] != MapCell::Wall {
            let box_index = box_grid[new_player_pos];
            let push_dest = new_player_pos + dir;

            if box_index == 255 {
                // step
                let new_state = arena.alloc(State::new(new_player_pos, state.boxes.clone()));
                new_states.push(&*new_state);
            } else if box_grid[push_dest] == 255
                && sd.map.grid[push_dest] != MapCell::Wall
                && sd.distances[push_dest].is_some()
            {
                // push

                let mut new_boxes = state.boxes.clone();
                new_boxes[box_index as usize] = push_dest;
                let new_state = arena.alloc(State::new(new_player_pos, new_boxes));
                new_states.push(&*new_state);
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
            Solver::new(&level).unwrap_err(),
            SolverErr::UnreachableBoxes
        );
    }

    #[test]
    fn incomplete_border() {
        let level0 = r"
####
#@*
####
        ";
        let level1 = r"
####
#@ *
####
        ";
        let level2 = r"
## #
#@*#
####
        ";
        let level3 = r"
####
# *#
#@##
        ";
        let level4 = r"
####
.@$*
####
        ";
        for level in &[level0, level1, level2, level3, level4] {
            let level = level.parse().unwrap();
            assert_eq!(
                Solver::new(&level).unwrap_err(),
                SolverErr::IncompleteBorder
            );
        }
    }

    #[test]
    fn too_many() {
        let level = r"
##################
#****************#
#****************#
#****************#
#****************#
#****************#
#****************#
#****************#
#****************#
#****************#
#****************#
#****************#
#****************#
#****************#
#****************#
#****************#
#****************#
#@################
###
";
        let level = level.parse().unwrap();
        let err = Solver::new(&level).unwrap_err();
        assert_eq!(err, SolverErr::TooMany);
        assert_eq!(err.to_string(), "More than 255 reachable boxes or goals");
    }

    #[test]
    fn distances1() {
        let level = r"
#####
##@##
##$##
#  .#
#####";
        let level: Level = level.parse().unwrap();

        let expected = Vec2d::new(&[
            vec![None, None, None, None, None],
            vec![None, None, None, None, None],
            vec![None, None, None, None, None],
            vec![None, None, Some(1), Some(0), None],
            vec![None, None, None, None, None],
        ]);
        assert_eq!(find_distances(&level.map), expected);
    }

    #[test]
    fn distances2() {
        let level = r"
###########
#@$$$$$$ ##
######## ##
######...##
#      .  #
#         #
## ########
#.       ##
#        ##
##  #.#####
###########";
        let level: Level = level.parse().unwrap();

        let expected = r"
None    None    None    None    None    None    None     None     None None None 
None    None    None    None    None    None    None     None     None None None 
None    None    None    None    None    None    None     None  Some(1) None None 
None    None    None    None    None    None Some(0)  Some(0)  Some(0) None None 
None    None Some(5) Some(4) Some(3) Some(2) Some(1)  Some(0)  Some(1) None None 
None    None Some(5) Some(6) Some(7) Some(8) Some(9) Some(10) Some(11) None None 
None    None Some(4)    None    None    None    None     None     None None None 
None Some(0) Some(1) Some(2) Some(3) Some(4) Some(5)  Some(6)     None None None 
None    None Some(2) Some(3) Some(2) Some(1) Some(2)  Some(3)     None None None 
None    None    None    None    None Some(0)    None     None     None None None 
None    None    None    None    None    None    None     None     None None None 
".trim_left_matches('\n');
        assert_eq!(format!("{:?}", find_distances(&level.map)), expected);
    }

    #[test]
    fn processing() {
        let level: &str = r"
*####*
#@$.*#
*####*#
".trim_left_matches('\n');

        let solver = Solver::new(&level.parse().unwrap()).unwrap();

        let processed_empty_level: &str = r"
#######
#  ..##
#######
".trim_left_matches('\n');
        assert_eq!(solver.sd.map.to_string(), processed_empty_level);

        assert_eq!(solver.initial_state.player_pos, Pos { r: 1, c: 1 });
        assert_eq!(
            solver.initial_state.boxes,
            vec![Pos { r: 1, c: 2 }, Pos { r: 1, c: 4 }]
        );
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
        let solver = Solver::new(&level).unwrap();
        let states = Arena::new();
        let neighbor_states = expand_push(&solver.sd, &solver.initial_state, &states);
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
        let solver = Solver::new(&level).unwrap();
        let states = Arena::new();
        let neighbor_states = expand_move(&solver.sd, &solver.initial_state, &states);
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
        let solver = Solver::new(&level).unwrap();
        let states = Arena::new();
        let neighbor_states = expand_move(&solver.sd, &solver.initial_state, &states);
        assert_eq!(neighbor_states.len(), 4);
    }
}
