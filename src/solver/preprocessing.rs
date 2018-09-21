use std::collections::VecDeque;

use crate::{
    data::{Dir, MapCell, Pos, DIRECTIONS},
    map::Map,
    solver::SolverErr,
    state::State,
    vec2d::Vec2d,
};

crate fn check_reachability<M: Map>(map: &M, state: &State) -> Result<Vec2d<MapCell>, SolverErr> {
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

#[inline(never)] // this is called only once and this way it's easier to see in callgrind
crate fn push_dists<M: Map>(map: &M) -> Vec2d<[Vec2d<Option<u16>>; 4]> {
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

    let mut push_dirs =
        map.grid()
            .scratchpad_with_default([Vec::new(), Vec::new(), Vec::new(), Vec::new()]);

    for box_pos in map.grid().positions() {
        if map.grid()[box_pos] == MapCell::Wall {
            continue;
        }

        for &player_to_box in &DIRECTIONS {
            // Technically, this could be optimized further because if the box is reachable from multiple dirs,
            // all of them have the same push dirs. `one_box_push_dirs` would have to be modified to return
            // reachable dists, not push dists.

            let player_pos = box_pos - player_to_box;
            if map.grid()[player_pos] == MapCell::Wall {
                continue;
            }

            push_dirs[box_pos][player_to_box as usize] =
                one_box_push_dirs(map, box_pos, player_pos);
        }
    }

    // this wastes some memory given
    // a) for one cell many directions likely have the same distances
    // b) there's a lot of cells and directions that have all dest cells None
    // 16x16 map: 2^4 * 2^4 (src size) * 4 (dirs) * 2^4 * 2^4 (dest size) * 4 B (size of Option<u16>)
    //            = 2^20 B = 1 MiB
    // 32x32 map: 2^24 B = 16 MiB
    // 64x64 map: 2^28 B = 256 MiB
    // 128x128 map: 2^32 B = 4 GiB
    // 256x256 map: 2^36 B = 64 GiB
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
            let player_start_pos = box_start_pos - initial_dir;
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

                let old_dist = &mut push_dists[box_start_pos][initial_dir as usize][cur_box_pos];
                if old_dist.is_none() {
                    // given this is BFS, the old value, if there is any, is always better
                    *old_dist = Some(cur_dist);
                }

                //for push_dir in Self::one_box_push_dirs(map, cur_box_pos, cur_player_pos) {
                for &push_dir in &push_dirs[cur_box_pos][player_to_box as usize] {
                    visited[cur_box_pos][player_to_box as usize] = true;
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
crate fn one_box_push_dirs<M: Map>(map: &M, box_pos: Pos, player_start_pos: Pos) -> Vec<Dir> {
    let mut ret = Vec::new();

    let mut touched = map.grid().scratchpad();
    touched[player_start_pos] = true;

    // BFS turns out to be faster than DFS here on the levels i benched
    let mut to_visit = VecDeque::new();
    to_visit.push_back(player_start_pos);

    while let Some(cur_pos) = to_visit.pop_front() {
        for &dir in &DIRECTIONS {
            let next_pos = cur_pos + dir;
            if next_pos == box_pos {
                // can't step on this pos (so `else if` is not taken) but can we actually push?
                if map.grid()[next_pos + dir] != MapCell::Wall {
                    // don't set touched here
                    // box pos can be touched multiple times - that's the whole point
                    ret.push(dir);
                    if ret.len() == 4 {
                        // there's only one box so 4 dirs is the max
                        return ret;
                    }
                }
            } else if map.grid()[next_pos] != MapCell::Wall && !touched[next_pos] {
                touched[next_pos] = true;
                to_visit.push_back(next_pos);
            }
        }
    }

    ret
}

crate fn closest_push_dists<M: Map>(
    map: &M,
    push_dists: &Vec2d<[Vec2d<Option<u16>>; 4]>,
) -> Vec2d<Option<u16>> {
    let mut closest_push_dists = map.grid().scratchpad();

    for src_pos in closest_push_dists.positions() {
        let mut best = None;
        for dests in &push_dists[src_pos] {
            for dest_pos in dests.positions() {
                if map.grid()[dest_pos] != MapCell::Goal && map.grid()[dest_pos] != MapCell::Remover
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::level::Level;
    use crate::map::GoalMap;
    use crate::solver::{Solver, SolverTrait, StaticData};

    #[test]
    fn one_box_reachability() {
        use crate::data::Dir::{self, *};
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
        let search_fn = one_box_push_dirs;

        // although the function should handle all player positions,
        // in practise the player will always be next to the box
        assert_eq!(
            hash_set(search_fn(map, center_box, center_box + Up)),
            hash_set(vec![Down, Left])
        );
        assert_eq!(
            hash_set(search_fn(map, center_box, center_box + Right)),
            hash_set(vec![Down, Left])
        );
        assert_eq!(
            hash_set(search_fn(map, center_box, center_box + Down)),
            hash_set(vec![Up, Right])
        );
        assert_eq!(
            hash_set(search_fn(map, center_box, center_box + Left)),
            hash_set(vec![Up, Right])
        );
        assert_eq!(
            hash_set(search_fn(map, left_box, left_box + Up)),
            hash_set(vec![Left])
        );
        assert_eq!(
            hash_set(search_fn(map, left_box, left_box + Right)),
            hash_set(vec![Left])
        );
        assert_eq!(
            hash_set(search_fn(map, left_box, left_box + Left)),
            hash_set(vec![Right])
        );
        assert_eq!(
            hash_set(search_fn(map, right_box, right_box + Up)),
            hash_set(vec![Up, Right, Down, Left])
        );
        assert_eq!(
            hash_set(search_fn(map, right_box, right_box + Right)),
            hash_set(vec![Up, Right, Down, Left])
        );
        assert_eq!(
            hash_set(search_fn(map, right_box, right_box + Down)),
            hash_set(vec![Up, Right, Down, Left])
        );
        assert_eq!(
            hash_set(search_fn(map, right_box, right_box + Left)),
            hash_set(vec![Up, Right, Down, Left])
        );
    }

    #[test]
    #[ignore] // pretty slow even in release mode
    fn push_distances() {
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
            let push_dists = push_dists(level.goal_map());

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
}
