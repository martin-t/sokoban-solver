crate mod a_star;
mod backtracking;
mod preprocessing;

#[cfg(feature = "graph")]
mod graph;

use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

use fnv::FnvHashMap; // using rustc-hash gives the same results, maybe bench again when able to solve levels with many boxes
use log::debug;
use typed_arena::Arena;

use crate::config::Method;
use crate::data::{MapCell, Pos, DIRECTIONS, MAX_BOXES};
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
            SolverErr::DiffBoxesGoals => write!(f, "Different number of reachable boxes and goals"),
        }
    }
}

impl Error for SolverErr {}

#[derive(Debug)]
pub struct SolverOk {
    pub moves: Option<Moves>,
    pub stats: Stats,
}

impl SolverOk {
    fn new(moves: Option<Moves>, stats: Stats) -> Self {
        Self { moves, stats }
    }
}

impl Solve for Level {
    fn solve(&self, method: Method, print_status: bool) -> Result<SolverOk, SolverErr> {
        debug!("Processing level...");

        // I am not quite sure how to merge these branches.
        // It should be possible with trait objects but they have additional restrictions
        // (https://doc.rust-lang.org/error-index.html#E0038) plus even then I might run
        // into [this](https://github.com/rust-lang/rust/issues/23856) bug.
        // It might be easier to keep the 2 branches.

        match self.map {
            MapType::Goals(ref goals_map) => {
                let solver = Solver::new_with_goals(goals_map, &self.state)?;

                debug!("Processed level");

                match method {
                    Method::MoveOptimal => Ok(solver.search(print_status, MoveLogic)),
                    Method::PushOptimal => Ok(solver.search(print_status, PushLogic)),
                }
            }
            MapType::Remover(ref remover_map) => {
                let solver = Solver::new_with_remover(remover_map, &self.state)?;

                debug!("Processed level");

                match method {
                    Method::MoveOptimal => Ok(solver.search(print_status, MoveLogic)),
                    Method::PushOptimal => Ok(solver.search(print_status, PushLogic)),
                }
            }
        }
    }
}

#[derive(Debug)]
struct Solver<M: Map> {
    // this should remain private given i might use unsafe to optimize things
    // and some of the values must be correct to avoid out of bounds array access
    sd: StaticData<M>,
}

#[derive(Debug)]
struct StaticData<M: Map> {
    map: M,
    initial_state: State,
    push_dists: Vec2d<[Vec2d<Option<u16>>; 4]>,
    closest_push_dists: Vec2d<Option<u16>>,
}

impl Solver<GoalMap> {
    fn new_with_goals(map: &GoalMap, state: &State) -> Result<Solver<GoalMap>, SolverErr> {
        // Guarantees we have here:
        // - the player exists and therefore map is at least 1x1.
        // - rows and cols is <= 255
        // Do some more low level checking so we can omit some checks later.

        let processed_grid = preprocessing::check_reachability(map, state)?;

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

        if reachable_boxes.len() != reachable_goals.len() {
            return Err(SolverErr::DiffBoxesGoals);
        }

        // only 255 boxes max because 255 (index of the 256th box) is used to represent empty in expand_{move,push}
        if reachable_boxes.len() > MAX_BOXES {
            return Err(SolverErr::TooMany);
        }

        let processed_map = GoalMap::new(processed_grid, reachable_goals);
        let clean_state = State::new(state.player_pos, reachable_boxes);
        let push_dists = preprocessing::push_dists(&processed_map);
        let closest_push_dists = preprocessing::closest_push_dists(&processed_map, &push_dists);
        Ok(Solver {
            sd: StaticData {
                map: processed_map,
                initial_state: clean_state,
                push_dists,
                closest_push_dists,
            },
        })
    }
}

impl Solver<RemoverMap> {
    fn new_with_remover(map: &RemoverMap, state: &State) -> Result<Solver<RemoverMap>, SolverErr> {
        // Guarantees we have here:
        // - the player exists and therefore map is at least 1x1.
        // - rows and cols is <= 255
        // Do some more low level checking so we can omit some checks later.

        let processed_grid = preprocessing::check_reachability(map, state)?;

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
        let push_dists = preprocessing::push_dists(&processed_map);
        let closest_push_dists = preprocessing::closest_push_dists(&processed_map, &push_dists);
        Ok(Solver {
            sd: StaticData {
                map: processed_map,
                initial_state: state.clone(),
                push_dists,
                closest_push_dists,
            },
        })
    }
}

trait SolverTrait {
    type M: Map + Clone;

    fn sd(&self) -> &StaticData<Self::M>;

    fn push_box(sd: &StaticData<Self::M>, state: &State, box_index: u8, push_dest: Pos)
        -> Vec<Pos>;

    fn search<GL: GameLogic<Self::M>>(&self, print_status: bool, _: GL) -> SolverOk
    where
        Solver<<Self as SolverTrait>::M>: SolverTrait,
    {
        debug!("Search called");

        let mut stats = Stats::new();

        // boxes that can't reach any goals
        // normally such states would not be generated at all but the first one is not generated so needs to be checked
        for &box_pos in &self.sd().initial_state.boxes {
            if self.sd().closest_push_dists[box_pos].is_none() {
                return SolverOk::new(None, stats);
            }
        }

        // already solved
        if self
            .sd()
            .initial_state
            .boxes
            .iter()
            .all(|&box_pos| self.sd().map.grid()[box_pos] == MapCell::Goal)
        {
            return SolverOk::new(Some(Moves::default()), stats);
        }

        // this might be more trouble than it's worth, we avoid expanding a whole *one* extra state
        // but it'll look cleaner if i ever get around to printing graphs of the state space
        let norm_initial_state = GL::preprocess_state(&self.sd().map, &self.sd().initial_state);

        let states = Arena::new();

        let mut to_visit = BinaryHeap::new();
        let mut prevs = FnvHashMap::default();

        let start = SearchNode::new(
            &norm_initial_state,
            None,
            0,
            GL::heuristic(self.sd(), &norm_initial_state),
        );
        stats.add_created(&start);
        to_visit.push(Reverse(start));

        //let mut counter = 0;
        while let Some(Reverse(cur_node)) = to_visit.pop() {
            /*counter += 1;
            if counter % 100_000 == 0 {
                use crate::map::Map;
                println!("prevs: {}, to_visit: {}", prevs.len(), to_visit.len());
                println!("{}", self.sd().map.xsb_with_state(&cur_node.state));
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
                #[cfg(feature = "graph")]
                graph::draw_states(&self.sd().map, &prevs);

                let moves = backtracking::reconstruct_moves(
                    &self.sd().map,
                    self.sd().initial_state.player_pos,
                    &prevs,
                    &cur_node.state,
                );
                return SolverOk::new(Some(moves), stats);
            }

            for (neighbor_state, cost, h) in GL::expand(self.sd(), &cur_node.state, &states) {
                // Insert everything and ignore duplicates when popping. This wastes memory
                // but when I filter them out here using a HashMap, push-optimal/boxxle2/4 becomes 8x slower
                // and generates much more states (although push-optimal/original/1 becomes about 2x faster).
                // I might have done something wrong, might wanna try again when i have better debugging tools
                // to look at the generated states.

                // Also might wanna try https://crates.io/crates/priority-queue for changing priorities
                // instead of adding duplicates.

                let next_node = SearchNode::new(
                    neighbor_state,
                    Some(&cur_node.state),
                    cur_node.dist + cost,
                    h,
                );
                stats.add_created(&next_node);
                to_visit.push(Reverse(next_node));
            }
        }

        SolverOk::new(None, stats)
    }
}

impl SolverTrait for Solver<GoalMap> {
    type M = GoalMap;

    fn sd(&self) -> &StaticData<Self::M> {
        &self.sd
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

trait GameLogic<M: Map + Clone>
where
    Solver<M>: SolverTrait,
{
    fn preprocess_state(_map: &M, state: &State) -> State {
        state.clone()
    }
    fn expand<'a>(
        sd: &StaticData<M>,
        state: &State,
        arena: &'a Arena<State>,
    ) -> Vec<(&'a State, u16, u16)>;
    fn heuristic(sd: &StaticData<M>, state: &State) -> u16;
}

struct MoveLogic;

impl<M: Map + Clone> GameLogic<M> for MoveLogic
where
    Solver<M>: SolverTrait<M = M>,
{
    fn expand<'a>(
        sd: &StaticData<M>,
        cur_state: &State,
        arena: &'a Arena<State>,
    ) -> Vec<(&'a State, u16, u16)> {
        let mut new_states = Vec::new();

        let mut box_grid = sd.map.grid().scratchpad_with_default(255u8);
        for (i, b) in cur_state.boxes.iter().enumerate() {
            box_grid[*b] = i as u8;
        }

        // find each box and each direction from which it can be pushed
        let mut reachable = sd.map.grid().scratchpad();
        reachable[cur_state.player_pos] = true;

        // Vec is noticeably faster than VecDeque on some levels
        let mut to_visit = VecDeque::new();
        to_visit.push_back((cur_state.player_pos, 0));

        while let Some((player_pos, steps)) = to_visit.pop_front() {
            for &dir in &DIRECTIONS {
                let new_player_pos = player_pos + dir;
                let box_index = box_grid[new_player_pos];
                if box_index < 255 {
                    // new_pos has a box
                    let push_dest = new_player_pos + dir;
                    if box_grid[push_dest] == 255 && sd.closest_push_dists[push_dest].is_some() {
                        // new state to explore
                        let new_boxes = Solver::<M>::push_box(sd, cur_state, box_index, push_dest);
                        let new_state = arena.alloc(State::new(new_player_pos, new_boxes));
                        let h = PushLogic::heuristic(sd, new_state);
                        // cost is number of steps plus the push
                        new_states.push((&*new_state, steps + 1, h));
                    }
                } else if sd.map.grid()[new_player_pos] != MapCell::Wall
                    && !reachable[new_player_pos]
                {
                    // new_pos is empty and not yet visited
                    reachable[new_player_pos] = true;
                    to_visit.push_back((new_player_pos, steps + 1));
                }
            }
        }

        new_states
    }

    fn heuristic(sd: &StaticData<M>, state: &State) -> u16 {
        if state.boxes.is_empty() {
            // TODO not necessary for goal maps
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
        closest_box - 1 + PushLogic::heuristic(sd, state)
    }
}

struct PushLogic;

impl<M: Map + Clone> GameLogic<M> for PushLogic
where
    Solver<M>: SolverTrait<M = M>,
{
    fn preprocess_state(map: &M, state: &State) -> State {
        State::new(
            normalized_pos(map, state.player_pos, &state.boxes),
            state.boxes.clone(),
        )
    }
    fn expand<'a>(
        sd: &StaticData<M>,
        cur_state: &State,
        arena: &'a Arena<State>,
    ) -> Vec<(&'a State, u16, u16)> {
        let mut new_states = Vec::new();

        let mut box_grid = sd.map.grid().scratchpad_with_default(255u8);
        for (i, b) in cur_state.boxes.iter().enumerate() {
            box_grid[*b] = i as u8;
        }

        // find each box and each direction from which it can be pushed
        let mut reachable = sd.map.grid().scratchpad();
        reachable[cur_state.player_pos] = true;

        // Vec is noticeably faster than VecDeque on some levels
        let mut to_visit = Vec::new();
        to_visit.push(cur_state.player_pos);

        while let Some(player_pos) = to_visit.pop() {
            for &dir in &DIRECTIONS {
                let new_player_pos = player_pos + dir;
                let box_index = box_grid[new_player_pos];
                if box_index < 255 {
                    // new_pos has a box
                    let push_dest = new_player_pos + dir;
                    if box_grid[push_dest] == 255 && sd.closest_push_dists[push_dest].is_some() {
                        // new state to explore
                        let new_boxes = Solver::<M>::push_box(sd, cur_state, box_index, push_dest);
                        let norm_player_pos = normalized_pos(&sd.map, new_player_pos, &new_boxes);
                        let new_state = arena.alloc(State::new(norm_player_pos, new_boxes));
                        let h = Self::heuristic(sd, &new_state);
                        new_states.push((&*new_state, 1, h));
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

    fn heuristic(sd: &StaticData<M>, state: &State) -> u16 {
        // thanks to precomputed distances, this is the same for goals and remover
        let mut goal_dist_sum = 0;

        for &box_pos in &state.boxes {
            goal_dist_sum += sd.closest_push_dists[box_pos].expect("Box on unreachable cell");
        }

        goal_dist_sum
    }
}

fn normalized_pos<M: Map>(map: &M, player_pos: Pos, boxes: &[Pos]) -> Pos {
    // note that pushing a box can reveal or hide new areas on both goal and remover maps
    // (and reusing is not worth it according to Brian Damgaard)
    // http://www.sokobano.de/wiki/index.php?title=Sokoban_solver_%22scribbles%22_by_Brian_Damgaard_about_the_YASS_solver#Re-using_the_calculated_player.27s_reachable_squares

    let mut top_left = player_pos;

    // this could be reused from the expand fn, just modified, then restored
    let mut box_grid = map.grid().scratchpad();
    for &b in boxes {
        box_grid[b] = true;
    }

    let mut to_visit = Vec::new();
    to_visit.push(player_pos);

    let mut visited = map.grid().scratchpad();
    visited[player_pos] = true;

    while let Some(cur_pos) = to_visit.pop() {
        for &new_pos in &cur_pos.neighbors() {
            if visited[new_pos] {
                continue;
            }
            visited[new_pos] = true;

            if map.grid()[new_pos] == MapCell::Wall || box_grid[new_pos] {
                continue;
            }

            to_visit.push(new_pos);
            if new_pos < top_left {
                top_left = new_pos;
            }
        }
    }

    top_left
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pos_normalization() {
        let levels = [
            r"
#####
#   #
# @ #
#   #
#####
",
            r"
#####
##  #
# @ #
#   #
#####
",
            r"
#####
#$  #
# @ #
#   #
#####
",
            r"
#####
#*  #
# @ #
#   #
#####
",
            r"
#####
#.  #
# @ #
#   #
#####
",
            r"
#############
##      #####
## #### #####
##$####$#####
##$         #
#   ###$### #
#$##### ### #
##     @$## #
## ######## #
## ######## #
##          #
#############
",
        ];
        let normalized_positions = [
            Pos::new(1, 1),
            Pos::new(1, 2),
            Pos::new(1, 2),
            Pos::new(1, 2),
            Pos::new(1, 1),
            Pos::new(4, 3),
        ];

        assert_eq!(levels.len(), normalized_positions.len());
        for (level, &expected_np) in levels.iter().zip(normalized_positions.iter()) {
            let level: Level = level.parse().unwrap();
            let np = normalized_pos(&level.map, level.state.player_pos, &level.state.boxes);
            assert_eq!(np, expected_np, "Level:\n{}", level.xsb());
        }
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
            let level: Level = level.parse().unwrap();
            assert_eq!(
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

        let err = Solver::new_with_goals(level.goal_map(), &level.state).unwrap_err();
        assert_eq!(err, SolverErr::TooMany);
        assert_eq!(err.to_string(), "More than 255 reachable boxes or goals");
    }

    #[test]
    fn diff_boxes_or_goals() {
        let level = r"
####
#@.*#
####
";
        let level: Level = level.parse().unwrap();
        assert_eq!(
            Solver::new_with_goals(level.goal_map(), &level.state).unwrap_err(),
            SolverErr::DiffBoxesGoals
        );
    }

    #[test]
    fn processing() {
        let level: &str = r"
*####*
#@$.*#
*####*#
".trim_left_matches('\n');

        let level: Level = level.parse().unwrap();
        let solver = Solver::new_with_goals(level.goal_map(), &level.state).unwrap();

        let processed_empty_level: &str = r"
#######
#  ..##
#######
".trim_left_matches('\n');
        assert_eq!(solver.sd.map.to_string(), processed_empty_level);

        assert_eq!(solver.sd.initial_state.player_pos, Pos { r: 1, c: 1 });
        assert_eq!(
            solver.sd.initial_state.boxes,
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
        let solver = Solver::new_with_goals(level.goal_map(), &level.state).unwrap();
        let states = Arena::new();
        let neighbor_states = PushLogic::expand(&solver.sd, &solver.sd.initial_state, &states);
        assert_eq!(neighbor_states.len(), 2);
    }

    #[test]
    fn expand_move1() {
        let level = r"
 ####
# $ .#
# @$*#
#.$  #
# .  #
 ####
";
        let level: Level = level.parse().unwrap();
        let solver = Solver::new_with_goals(level.goal_map(), &level.state).unwrap();
        let states = Arena::new();
        let neighbor_states = MoveLogic::expand(&solver.sd, &solver.sd.initial_state, &states);
        assert_eq!(neighbor_states.len(), 7);
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
        let solver = Solver::new_with_goals(level.goal_map(), &level.state).unwrap();
        let states = Arena::new();
        let neighbor_states = MoveLogic::expand(&solver.sd, &solver.sd.initial_state, &states);
        assert_eq!(neighbor_states.len(), 4);
    }
}
