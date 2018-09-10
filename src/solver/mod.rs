crate mod a_star;
mod backtracking;

use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

use fnv::FnvHashMap; // using rustc-hash gives the same results, maybe bench again when able to solve levels with many boxes
use log::{debug, log};
use typed_arena::Arena;

use crate::config::Method;
use crate::data::{Dir, MapCell, Pos, DIRECTIONS, MAX_BOXES};
use crate::level::Level;
use crate::map::{GoalMap, Map, MapType, RemoverMap};
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
    UnreachableRemover,
    TooMany,
    NoBoxesGoals,
    DiffBoxesGoals,
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
            SolverErr::UnreachableRemover => write!(f, "Remover is not reachable"),
            SolverErr::TooMany => write!(f, "More than {} reachable boxes or goals", MAX_BOXES),
            SolverErr::NoBoxesGoals => write!(f, "No reachable boxes or goals"),
            SolverErr::DiffBoxesGoals => write!(f, "Different number of reachable boxes and goals"),
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

        match self.map {
            MapType::Goals(ref goals_map) => {
                let solver = Solver::new_with_goals(goals_map, &self.state)?;

                debug!("Processed level");

                match method {
                    Method::MoveOptimal => Ok(solver.search(
                        method,
                        print_status,
                        Solver::<GoalMap>::expand_move,
                        heuristic_move_goals,
                    )),
                    Method::PushOptimal => Ok(solver.search(
                        method,
                        print_status,
                        Solver::<GoalMap>::expand_push,
                        heuristic_push,
                    )),
                }
            }
            MapType::Remover(ref remover_map) => {
                let solver = Solver::new_with_remover(remover_map, &self.state)?;

                debug!("Processed level");

                match method {
                    Method::MoveOptimal => Ok(solver.search(
                        method,
                        print_status,
                        Solver::<RemoverMap>::expand_move,
                        heuristic_move_remover,
                    )),
                    Method::PushOptimal => Ok(solver.search(
                        method,
                        print_status,
                        Solver::<RemoverMap>::expand_push,
                        heuristic_push,
                    )),
                }
            }
        }
    }
}

#[derive(Debug)]
struct StaticData<M: Map> {
    map: M,
    push_dists: Vec2d<[Vec2d<Option<u16>>; 4]>,
    closest_push_dists: Vec2d<Option<u16>>,
}

trait SolverTrait {
    type M: Map + Clone;

    fn sd(&self) -> &StaticData<Self::M>;

    fn initial_state(&self) -> &State;

    fn push_box(sd: &StaticData<Self::M>, state: &State, box_index: u8, push_dest: Pos)
        -> Vec<Pos>;

    fn push_dists(map: &Self::M) -> Vec2d<[Vec2d<Option<u16>>; 4]>
    where
        Solver<Self::M>: SolverTrait<M = Self::M>,
    {
        // I don't think distances per direction can be used as a heuristic - example:
        // Center box is pushable only from bottom but shortest solution first pushes the bottom box
        // which would lower the heuristic of the center box by 2 -> the push distance depends
        // on the directions available *ignoring other boxes*.
        // ##########
        // #######  #
        // ####     #
        // ####  ##.#
        // ##### ## #
        // ##   $   #
        // ## ##@#  #
        // ## ## ####
        // #.$   ####
        // ##########
        // The only thing directions can probably prevent is pushing boxes into dead end tunnels.

        // this wastes some memory given
        // a) for one cell many directions likely have the same distances
        // b) there's a lot of cells and directions that have all dest cells None
        // 16x16 map: 2^4 * 2^4 (src size) * 4 (dirs) * 2^4 * 2^4 (dest size) * 4 B (size of Option<u16>)
        //            = 2^20 B = 1 MiB
        // 32x32 map: 2^24 B = 16 MiB
        // 64x64 map: 2^28 B = 256 MiB
        // 128x128 map: 2^32 B = 4 GiB
        // 256x256 map: 2^36 B = 64 GiB
        // also it's very slow for large open areas
        let mut push_dists: Vec2d<[Vec2d<Option<u16>>; 4]> = map.grid().scratchpad_with_default([
            map.grid().scratchpad(),
            map.grid().scratchpad(),
            map.grid().scratchpad(),
            map.grid().scratchpad(),
        ]);

        for box_start_pos in map.grid().positions() {
            if map.grid()[box_start_pos] == MapCell::Wall {
                continue;
            }
            for &initial_dir in &DIRECTIONS {
                let player_start_pos = box_start_pos + initial_dir.inverse();
                if map.grid()[player_start_pos] == MapCell::Wall {
                    continue;
                }

                // BFS of pushes fanning out from the box position.
                // `visited` must be per direction because going back to the same cell from a different direction
                // means different areas are accessible.
                let mut visited = map.grid().scratchpad_with_default([false; 4]);
                let mut to_visit = VecDeque::new();
                to_visit.push_back((box_start_pos, player_start_pos, 0));

                while let Some((cur_box_pos, cur_player_pos, cur_dist)) = to_visit.pop_front() {
                    let player_to_box = cur_player_pos.dir_to(cur_box_pos);
                    if visited[cur_box_pos][player_to_box as usize] {
                        continue;
                    }
                    visited[cur_box_pos][player_to_box as usize] = true;

                    let old_dist =
                        &mut push_dists[box_start_pos][initial_dir as usize][cur_box_pos];
                    if old_dist.is_none() {
                        // given this is BFS, the old value, if there is any, is always better
                        *old_dist = Some(cur_dist);
                    }

                    for push_dir in Self::dfs_one_box(map, cur_box_pos, cur_player_pos) {
                        to_visit.push_back((cur_box_pos + push_dir, cur_box_pos, cur_dist + 1));
                    }
                }
            }
        }

        /*for box_start_pos in map.grid().positions() {
            for &initial_dir in &DIRECTIONS {
                println!(
                    "box_start_pos: {:?}, initial_dir: {:?}",
                    box_start_pos, initial_dir
                );
                println!("{:?}", push_dists[box_start_pos][initial_dir as usize]);
            }
        }*/

        push_dists
    }

    /// Finds in which directions the box is pushable
    fn dfs_one_box(map: &Self::M, box_pos: Pos, player_start_pos: Pos) -> Vec<Dir> {
        let mut ret = Vec::new();

        let mut touched = map.grid().scratchpad();
        touched[player_start_pos] = true;

        let mut to_visit = Vec::new();
        to_visit.push(player_start_pos);

        while let Some(cur_pos) = to_visit.pop() {
            for &dir in &DIRECTIONS {
                let next_pos = cur_pos + dir;
                if next_pos == box_pos {
                    // can't step on this pos (so `else if` is no taken) but can we actually push?
                    if map.grid()[next_pos + dir] != MapCell::Wall {
                        // don't set touched here
                        // box pos can be touched multiple times - that's the whole point
                        ret.push(dir);
                    }
                } else if map.grid()[next_pos] != MapCell::Wall && !touched[next_pos] {
                    touched[next_pos] = true;
                    to_visit.push(next_pos);
                }
            }
        }

        ret
    }

    fn closest_push_dists(
        map: &Self::M,
        push_dists: &Vec2d<[Vec2d<Option<u16>>; 4]>,
    ) -> Vec2d<Option<u16>>
    where
        Solver<Self::M>: SolverTrait<M = Self::M>,
    {
        let mut closest_push_dists = map.grid().scratchpad();

        for src_pos in closest_push_dists.positions() {
            let mut best = None;
            for dests in &push_dists[src_pos] {
                for dest_pos in dests.positions() {
                    if map.grid()[dest_pos] != MapCell::Goal
                        && map.grid()[dest_pos] != MapCell::Remover
                    {
                        continue;
                    }

                    let cur = dests[dest_pos];
                    match best {
                        None => best = cur,
                        Some(best_dist) => if let Some(cur_dist) = cur {
                            if cur_dist < best_dist {
                                best = cur;
                            }
                        },
                    }
                }
            }
            closest_push_dists[src_pos] = best;
        }

        closest_push_dists
    }

    fn search<Expand, Heuristic>(
        &self,
        method: Method,
        print_status: bool,
        expand: Expand,
        heuristic: Heuristic,
    ) -> SolverOk
    where
        Expand: for<'a> Fn(&StaticData<Self::M>, &State, &'a Arena<State>) -> Vec<&'a State>,
        Heuristic: Fn(&StaticData<Self::M>, &State) -> u16,
        Solver<Self::M>: SolverTrait,
    {
        debug!("Search called");

        let mut stats = Stats::new();

        // normally such states would not be generated at all but the first one is not generated so needs to be checked
        for &box_pos in &self.initial_state().boxes {
            if self.sd().closest_push_dists[box_pos].is_none() {
                return SolverOk::new(None, stats, method);
            }
        }

        let states = Arena::new();

        let mut to_visit = BinaryHeap::new();
        let mut prevs = FnvHashMap::default();

        let start = SearchNode::new(
            self.initial_state(),
            None,
            0,
            heuristic(self.sd(), self.initial_state()),
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
                        &self.sd().map,
                        &prevs,
                        &cur_node.state,
                    )),
                    stats,
                    method,
                );
            }

            for neighbor_state in expand(self.sd(), &cur_node.state, &states) {
                // Insert everything and ignore duplicates when popping. This wastes memory
                // but when I filter them out here using a HashMap, push-optimal/boxxle2/4 becomes 8x slower
                // and generates much more states (although push-optimal/original/1 becomes about 2x faster).
                // I might have done something wrong, might wanna try again when i have better debugging tools
                // to look at the generated states.

                // Also might wanna try https://crates.io/crates/priority-queue for changing priorities
                // instead of adding duplicates.

                let h = heuristic(self.sd(), neighbor_state);
                let next_node =
                    SearchNode::new(neighbor_state, Some(&cur_node.state), cur_node.dist + 1, h);
                stats.add_created(&next_node);
                to_visit.push(Reverse(next_node));
            }
        }

        SolverOk::new(None, stats, method)
    }

    fn expand_move<'a>(
        sd: &StaticData<Self::M>,
        state: &State,
        arena: &'a Arena<State>,
    ) -> Vec<&'a State> {
        let mut new_states = Vec::new();

        let mut box_grid = sd.map.grid().scratchpad_with_default(255u8);
        for (i, b) in state.boxes.iter().enumerate() {
            box_grid[*b] = i as u8;
        }

        for &dir in &DIRECTIONS {
            let new_player_pos = state.player_pos + dir;
            if sd.map.grid()[new_player_pos] != MapCell::Wall {
                let box_index = box_grid[new_player_pos];
                let push_dest = new_player_pos + dir;

                if box_index == 255 {
                    // step
                    let new_state = arena.alloc(State::new(new_player_pos, state.boxes.clone()));
                    new_states.push(&*new_state);
                } else if box_grid[push_dest] == 255
                    && sd.map.grid()[push_dest] != MapCell::Wall
                    && sd.closest_push_dists[push_dest].is_some()
                {
                    // push
                    let new_boxes = Self::push_box(sd, state, box_index, push_dest);
                    let new_state = arena.alloc(State::new(new_player_pos, new_boxes));
                    new_states.push(&*new_state);
                }
            }
        }

        new_states
    }

    fn expand_push<'a>(
        sd: &StaticData<Self::M>,
        state: &State,
        arena: &'a Arena<State>,
    ) -> Vec<&'a State> {
        let mut new_states = Vec::new();

        let mut box_grid = sd.map.grid().scratchpad_with_default(255u8);
        for (i, b) in state.boxes.iter().enumerate() {
            box_grid[*b] = i as u8;
        }

        // find each box and each direction from which it can be pushed
        let mut reachable = sd.map.grid().scratchpad();
        reachable[state.player_pos] = true;

        // Vec is noticeably faster than VecDeque on some levels
        let mut to_visit = vec![state.player_pos];

        while let Some(player_pos) = to_visit.pop() {
            for &dir in &DIRECTIONS {
                let new_player_pos = player_pos + dir;
                let box_index = box_grid[new_player_pos];
                if box_index < 255 {
                    // new_pos has a box
                    let push_dest = new_player_pos + dir;
                    if box_grid[push_dest] == 255 && sd.closest_push_dists[push_dest].is_some() {
                        // new state to explore
                        let new_boxes = Self::push_box(sd, state, box_index, push_dest);

                        // TODO normalize player pos
                        // note that pushing a box can reveal new areas on both goal and remover maps
                        let new_state = arena.alloc(State::new(new_player_pos, new_boxes));
                        new_states.push(&*new_state);
                    }
                } else if sd.map.grid()[new_player_pos] != MapCell::Wall
                    && !reachable[new_player_pos]
                {
                    // new_pos is empty and not yet visited
                    reachable[new_player_pos] = true;
                    to_visit.push(new_player_pos);
                }
            }
        }

        new_states
    }
}

impl SolverTrait for Solver<GoalMap> {
    type M = GoalMap;

    fn sd(&self) -> &StaticData<Self::M> {
        &self.sd
    }

    fn initial_state(&self) -> &State {
        &self.initial_state
    }

    fn push_box(
        _sd: &StaticData<Self::M>,
        state: &State,
        box_index: u8,
        push_dest: Pos,
    ) -> Vec<Pos> {
        let mut new_boxes = state.boxes.clone();
        new_boxes[box_index as usize] = push_dest;
        new_boxes
    }
}

impl SolverTrait for Solver<RemoverMap> {
    type M = RemoverMap;

    fn sd(&self) -> &StaticData<Self::M> {
        &self.sd
    }

    fn initial_state(&self) -> &State {
        &self.initial_state
    }

    fn push_box(
        sd: &StaticData<Self::M>,
        state: &State,
        box_index: u8,
        push_dest: Pos,
    ) -> Vec<Pos> {
        let mut new_boxes = state.boxes.clone();
        if sd.map.grid()[push_dest] == MapCell::Remover {
            new_boxes.remove(box_index as usize);
        } else {
            new_boxes[box_index as usize] = push_dest;
        }
        new_boxes
    }
}

#[derive(Debug)]
struct Solver<M: Map> {
    // this should remain private given i might use unsafe to optimize things
    // and some of the values must be correct to avoid out of bounds array access
    sd: StaticData<M>,
    initial_state: State,
}

impl Solver<GoalMap> {
    fn new_with_goals(map: &GoalMap, state: &State) -> Result<Solver<GoalMap>, SolverErr> {
        // Guarantees we have here:
        // - the player exists and therefore map is at least 1x1.
        // - rows and cols is <= 255
        // Do some more low level checking so we can omit some checks later.

        let processed_grid = Self::check_reachability(map, state)?;

        // make sure all relevant game elements are reachable
        let mut reachable_boxes = Vec::new();
        for &pos in &state.boxes {
            if processed_grid[pos] != MapCell::Wall {
                reachable_boxes.push(pos);
            } else if !map.goals.contains(&pos) {
                return Err(SolverErr::UnreachableBoxes);
            }
        }

        let mut reachable_goals = Vec::new();
        for &pos in &map.goals {
            if processed_grid[pos] != MapCell::Wall {
                reachable_goals.push(pos);
            } else if !state.boxes.contains(&pos) {
                return Err(SolverErr::UnreachableGoals);
            }
        }

        // technically, one could argue such a level is solved
        // but it creates an annyoing edge case for some heuristics
        if reachable_boxes.is_empty() || reachable_goals.is_empty() {
            return Err(SolverErr::NoBoxesGoals);
        }

        if reachable_boxes.len() != reachable_goals.len() {
            return Err(SolverErr::DiffBoxesGoals);
        }

        // only 255 boxes max because 255 (index of the 256th box) is used to represent empty in expand_{move,push}
        if reachable_boxes.len() > MAX_BOXES {
            return Err(SolverErr::TooMany);
        }

        let processed_map = GoalMap::new(processed_grid, reachable_goals);
        let clean_state = State::new(state.player_pos, reachable_boxes);
        let push_dists = Self::push_dists(&processed_map);
        let closest_push_dists = Self::closest_push_dists(&processed_map, &push_dists);
        Ok(Solver {
            sd: StaticData {
                map: processed_map,
                push_dists,
                closest_push_dists,
            },
            initial_state: clean_state,
        })
    }
}

impl Solver<RemoverMap> {
    fn new_with_remover(map: &RemoverMap, state: &State) -> Result<Solver<RemoverMap>, SolverErr> {
        // Guarantees we have here:
        // - the player exists and therefore map is at least 1x1.
        // - rows and cols is <= 255
        // Do some more low level checking so we can omit some checks later.

        let processed_grid = Self::check_reachability(map, state)?;

        if processed_grid[map.remover] == MapCell::Wall {
            return Err(SolverErr::UnreachableRemover);
        }

        for &pos in &state.boxes {
            if processed_grid[pos] == MapCell::Wall {
                return Err(SolverErr::UnreachableBoxes);
            }
        }

        // Note that a level with 0 boxes is valid (and already solved).
        // This should not upset the heuristics (since they already have to handle that case on remover maps)
        // or backtracking (since there are no moves).

        // only 255 boxes max because 255 (index of the 256th box) is used to represent empty in expand_{move,push}
        if state.boxes.len() > MAX_BOXES {
            return Err(SolverErr::TooMany);
        }

        let processed_map = RemoverMap::new(processed_grid, map.remover);
        let push_dists = Self::push_dists(&processed_map);
        let closest_push_dists = Self::closest_push_dists(&processed_map, &push_dists);
        Ok(Solver {
            sd: StaticData {
                map: processed_map,
                push_dists,
                closest_push_dists,
            },
            initial_state: state.clone(),
        })
    }
}

impl<M: Map> Solver<M> {
    fn check_reachability(map: &M, state: &State) -> Result<Vec2d<MapCell>, SolverErr> {
        // make sure the level is surrounded by wall
        let mut visited = map.grid().scratchpad();

        let mut to_visit = vec![state.player_pos];
        while let Some(cur) = to_visit.pop() {
            visited[cur] = true;

            let (r, c) = (i32::from(cur.r), i32::from(cur.c));
            let neighbors = [(r + 1, c), (r - 1, c), (r, c + 1), (r, c - 1)];
            for &(nr, nc) in &neighbors {
                // this is the only place in the solver where we need to check bounds (using signed types)
                // everything after that will be surrounded by walls
                if nr < 0
                    || nc < 0
                    || nr >= i32::from(map.grid().rows())
                    || nc >= i32::from(map.grid().cols())
                {
                    // we got out of bounds without hitting a wall
                    return Err(SolverErr::IncompleteBorder);
                }

                let new_pos = Pos::new(nr as u8, nc as u8);
                if !visited[new_pos] && map.grid()[new_pos] != MapCell::Wall {
                    to_visit.push(new_pos);
                }
            }
        }

        // make sure all non-reachable cells are walls
        // to avoid errors with some code that iterates through all non-walls
        let mut processed_grid = map.grid().clone();
        for pos in processed_grid.positions() {
            if !visited[pos] {
                processed_grid[pos] = MapCell::Wall;
            }
        }

        Ok(processed_grid)
    }
}

fn heuristic_push<M: Map>(sd: &StaticData<M>, state: &State) -> u16 {
    // thanks to precomputed distances, this is the same for goals and remover
    let mut goal_dist_sum = 0;

    for &box_pos in &state.boxes {
        goal_dist_sum += sd.closest_push_dists[box_pos].expect("Box on unreachable cell");
    }

    goal_dist_sum
}

fn heuristic_move_goals(sd: &StaticData<GoalMap>, state: &State) -> u16 {
    let mut closest_box = u16::max_value();
    for box_pos in &state.boxes {
        let dist = state.player_pos.dist(*box_pos);
        if dist < closest_box {
            closest_box = dist;
        }
    }

    // -1 because it is the distance until being able to push the box
    // and when all boxes are on goals, the heuristic must be 0
    closest_box - 1 + heuristic_push(sd, state)
}

fn heuristic_move_remover(sd: &StaticData<RemoverMap>, state: &State) -> u16 {
    if state.boxes.is_empty() {
        return 0;
    }

    let mut closest_box = u16::max_value();
    for box_pos in &state.boxes {
        let dist = state.player_pos.dist(*box_pos);
        if dist < closest_box {
            closest_box = dist;
        }
    }

    // -1 because it is the distance until being able to push the box
    // and when all boxes are on goals, the heuristic must be 0
    closest_box - 1 + heuristic_push(sd, state)
}

#[cfg(test)]
mod tests {
    use super::*;

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
            let level: Level = level.parse().unwrap();
            assert_eq!(
                //Solver::new(&level).unwrap_err(),
                Solver::new_with_goals(level.goal_map(), &level.state).unwrap_err(),
                SolverErr::IncompleteBorder
            );
        }
    }

    #[test]
    fn unreachable_boxes_goals() {
        let level = r"
########
#@$.#$.#
########
";
        let level: Level = level.parse().unwrap();
        assert_eq!(
            //Solver::new(&level).unwrap_err(),
            Solver::new_with_goals(level.goal_map(), &level.state).unwrap_err(),
            SolverErr::UnreachableBoxes
        );
    }

    #[test]
    fn unreachable_boxes_remover() {
        let level = r"
########
# $ #$R#
########
";
        let level: Level = level.parse().unwrap();
        assert_eq!(
            //Solver::new(&level).unwrap_err(),
            Solver::new_with_remover(level.remover_map(), &level.state).unwrap_err(),
            SolverErr::UnreachableBoxes
        );
    }

    #[test]
    fn unreachable_goals() {
        let level = r"
########
#@$ # .#
########
";
        let level: Level = level.parse().unwrap();
        assert_eq!(
            //Solver::new(&level).unwrap_err(),
            Solver::new_with_goals(level.goal_map(), &level.state).unwrap_err(),
            SolverErr::UnreachableGoals
        );
    }

    #[test]
    fn unreachable_remover() {
        let level = r"
########
#@$$# r#
########
";
        let level: Level = level.parse().unwrap();
        assert_eq!(
            //Solver::new(&level).unwrap_err(),
            Solver::new_with_remover(level.remover_map(), &level.state).unwrap_err(),
            SolverErr::UnreachableRemover
        );
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
        let level: Level = level.parse().unwrap();
        //let err = Solver::new(&level).unwrap_err();
        let err = Solver::new_with_goals(level.goal_map(), &level.state).unwrap_err();
        assert_eq!(err, SolverErr::TooMany);
        assert_eq!(err.to_string(), "More than 255 reachable boxes or goals");
    }

    #[test]
    fn no_boxes_or_goals() {
        let level = r"
###
#@#
###
";
        let level: Level = level.parse().unwrap();
        //assert_eq!(Solver::new(&level).unwrap_err(), SolverErr::NoBoxesGoals);
        assert_eq!(
            Solver::new_with_goals(level.goal_map(), &level.state).unwrap_err(),
            SolverErr::NoBoxesGoals
        );
    }

    #[test]
    fn diff_boxes_or_goals() {
        let level = r"
####
#@.*#
####
";
        let level: Level = level.parse().unwrap();
        //assert_eq!(Solver::new(&level).unwrap_err(), SolverErr::DiffBoxesGoals);
        assert_eq!(
            Solver::new_with_goals(level.goal_map(), &level.state).unwrap_err(),
            SolverErr::DiffBoxesGoals
        );
    }

    #[test]
    fn one_box_reachability() {
        use crate::data::Dir::*;
        use std::collections::HashSet;

        let level = r"
##########
#######  #
####     #
####  ##.#
##### ## #
##   $   #
## ##@#  ######
## ## ####    #
#.$         * #
##########    #
###############";
        let level: Level = level.parse().unwrap();
        let map = level.goal_map();
        let center_box = level.state.boxes[0];
        let left_box = level.state.boxes[1];
        let right_box = level.state.boxes[2];

        fn hash_set(v: Vec<Dir>) -> HashSet<Dir> {
            v.into_iter().collect()
        }
        let dfs = Solver::<GoalMap>::dfs_one_box;

        // although the function should handle all player positions,
        // in practise the player will always be next to the box
        assert_eq!(
            hash_set(dfs(map, center_box, center_box + Up)),
            hash_set(vec![Down, Left])
        );
        assert_eq!(
            hash_set(dfs(map, center_box, center_box + Right)),
            hash_set(vec![Down, Left])
        );
        assert_eq!(
            hash_set(dfs(map, center_box, center_box + Down)),
            hash_set(vec![Up, Right])
        );
        assert_eq!(
            hash_set(dfs(map, center_box, center_box + Left)),
            hash_set(vec![Up, Right])
        );
        assert_eq!(
            hash_set(dfs(map, left_box, left_box + Up)),
            hash_set(vec![Left])
        );
        assert_eq!(
            hash_set(dfs(map, left_box, left_box + Right)),
            hash_set(vec![Left])
        );
        assert_eq!(
            hash_set(dfs(map, left_box, left_box + Left)),
            hash_set(vec![Right])
        );
        assert_eq!(
            hash_set(dfs(map, right_box, right_box + Up)),
            hash_set(vec![Up, Right, Down, Left])
        );
        assert_eq!(
            hash_set(dfs(map, right_box, right_box + Right)),
            hash_set(vec![Up, Right, Down, Left])
        );
        assert_eq!(
            hash_set(dfs(map, right_box, right_box + Down)),
            hash_set(vec![Up, Right, Down, Left])
        );
        assert_eq!(
            hash_set(dfs(map, right_box, right_box + Left)),
            hash_set(vec![Up, Right, Down, Left])
        );
    }

    #[test]
    #[ignore] // pretty slow even in release mode
    fn push_dists() {
        fn heuristic_push_manhattan_goals(sd: &StaticData<GoalMap>, state: &State) -> u16 {
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

        let level0 = r"
###############
#######  ######
####     ######
####  ## ######
##### ## ######
##       ######
## ##@#  ######
## ## ####    #
#             #
##########    #
###############";
        let level1 = r"
###########
#@       ##
######## ##
######   ##
#         #
#         #
## ########
#        ##
#        ##
##  # #####
###########";
        let level0: Level = level0.parse().unwrap();
        let level1: Level = level1.parse().unwrap();
        for level in &[level0, level1] {
            let push_dists = Solver::<GoalMap>::push_dists(level.goal_map());

            // put box on every position and try to get it to every position
            for box_pos in level.map.grid().positions() {
                println!("{:?}", box_pos);
                if level.map.grid()[box_pos] == MapCell::Wall {
                    continue;
                }

                for &dir in &DIRECTIONS {
                    let player_pos = box_pos + dir.inverse();
                    if level.map.grid()[player_pos] == MapCell::Wall {
                        continue;
                    }

                    let fake_state = State::new(player_pos, vec![box_pos]);

                    for goal_pos in level.map.grid().positions() {
                        if level.map.grid()[goal_pos] == MapCell::Wall {
                            continue;
                        }

                        let mut fake_map = level.goal_map().clone();
                        fake_map.grid[goal_pos] = MapCell::Goal;
                        fake_map.goals = vec![goal_pos];
                        let fake_solver = Solver::new_with_goals(&fake_map, &fake_state).unwrap();
                        let moves = fake_solver
                            .search(
                                Method::PushOptimal,
                                false,
                                Solver::<GoalMap>::expand_push,
                                heuristic_push_manhattan_goals,
                            ).moves;

                        let dist_result = push_dists[box_pos][dir as usize][goal_pos];
                        let dist_expected = moves.map(|m| m.push_cnt() as u16);

                        assert_eq!(dist_result, dist_expected);
                    }
                }
            }
        }
    }

    #[test]
    fn closest_distances_one_goal_1() {
        let level = r"
#######
###@###
###$###
#    .#
#######";
        let level: Level = level.parse().unwrap();

        let expected = r"
None None    None    None    None    None None 
None None    None    None    None    None None 
None None    None    None    None    None None 
None None Some(3) Some(2) Some(1) Some(0) None 
None None    None    None    None    None None 
".trim_left_matches('\n');

        let solver = Solver::new_with_goals(level.goal_map(), &level.state).unwrap();
        let result = format!("{:?}", solver.sd.closest_push_dists);
        assert_eq!(result, expected);
    }

    #[test]
    fn closest_distances_one_goal_2() {
        let level = r"
#######
#  @###
# #$###
#    .#
#######";
        let level: Level = level.parse().unwrap();

        let expected = r"
None None    None    None    None    None None 
None None    None    None    None    None None 
None None    None Some(3)    None    None None 
None None Some(3) Some(2) Some(1) Some(0) None 
None None    None    None    None    None None 
".trim_left_matches('\n');

        let solver = Solver::new_with_goals(level.goal_map(), &level.state).unwrap();
        let result = format!("{:?}", solver.sd.closest_push_dists);
        assert_eq!(result, expected);
    }

    #[test]
    fn closest_distances_many_goals() {
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

        let solver = Solver::new_with_goals(level.goal_map(), &level.state).unwrap();
        let result = format!("{:?}", solver.sd.closest_push_dists);
        assert_eq!(result, expected);
    }

    #[test]
    fn processing() {
        let level: &str = r"
*####*
#@$.*#
*####*#
".trim_left_matches('\n');

        //let solver = Solver::new(&level.parse().unwrap()).unwrap();
        let level: Level = level.parse().unwrap();
        let solver = Solver::new_with_goals(level.goal_map(), &level.state).unwrap();

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
        let level: Level = level.parse().unwrap();
        //let solver = Solver::new(&level).unwrap();
        let solver = Solver::new_with_goals(level.goal_map(), &level.state).unwrap();
        let states = Arena::new();
        let neighbor_states =
            Solver::<GoalMap>::expand_push(&solver.sd, &solver.initial_state, &states);
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
        let level: Level = level.parse().unwrap();
        //let solver = Solver::new(&level).unwrap();
        let solver = Solver::new_with_goals(level.goal_map(), &level.state).unwrap();
        let states = Arena::new();
        let neighbor_states =
            Solver::<GoalMap>::expand_move(&solver.sd, &solver.initial_state, &states);
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
        let level: Level = level.parse().unwrap();
        //let solver = Solver::new(&level).unwrap();
        let solver = Solver::new_with_goals(level.goal_map(), &level.state).unwrap();
        let states = Arena::new();
        let neighbor_states =
            Solver::<GoalMap>::expand_move(&solver.sd, &solver.initial_state, &states);
        assert_eq!(neighbor_states.len(), 4);
    }
}

// TODO when the interface is stable, remove all the commented out code in tests
