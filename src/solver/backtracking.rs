use std::borrow::Borrow;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{BuildHasher, Hash};

use crate::data::{MapCell, Pos};
use crate::map::GoalMap;
use crate::moves::{Move, Moves};
use crate::state::State;

// Terminology:
// move = changing player position by one cell
// push = a move that changes a box position
// step = a move that doesn't change a box position

crate fn reconstruct_moves<H: BuildHasher>(
    map: &GoalMap,
    prevs: &HashMap<&State, &State, H>,
    final_state: &State,
) -> Moves {
    let states = backtrack_prevs(prevs, final_state);

    // TODO this should work but test what happens when we give solver an already solved level

    let mut moves = Moves::default();
    let mut iter = states.iter();
    let mut cur_state = iter.next().unwrap();
    for next_state in iter {
        moves.extend(&moves_between_states(map, cur_state, next_state));
        cur_state = next_state;
    }

    moves
}

/// The difference between them must be any number of steps and zero or one push
fn moves_between_states(map: &GoalMap, old: &State, new: &State) -> Moves {
    let old_boxes: HashSet<_> = old.boxes.iter().collect();
    let new_boxes: HashSet<_> = new.boxes.iter().collect();

    let mut old_iter = old_boxes.difference(&new_boxes);
    let maybe_old_box_pos = old_iter.next();
    if maybe_old_box_pos.is_none() {
        // no box moved so there are only steps
        return player_steps(map, old, old.player_pos, new.player_pos);
    }
    let old_box_pos = **maybe_old_box_pos.unwrap();
    assert!(old_iter.next().is_none());

    let mut new_iter = new_boxes.difference(&old_boxes);
    let new_box_pos = **new_iter.next().unwrap();
    assert!(new_iter.next().is_none());

    let push_dir = old_box_pos.dir_to(new_box_pos);
    let player_pos_before_push = new.player_pos + push_dir.inverse();
    let mut moves = player_steps(map, old, old.player_pos, player_pos_before_push);
    moves.add(Move::new(push_dir, true));

    moves
}

fn player_steps(map: &GoalMap, state: &State, src_pos: Pos, dest_pos: Pos) -> Moves {
    if src_pos == dest_pos {
        // because it's not a proper BFS with an open set
        return Moves::default();
    }

    let mut box_grid = map.grid.scratchpad();
    for &b in &state.boxes {
        box_grid[b] = true;
    }

    let mut prevs = HashMap::new();
    prevs.insert(src_pos, src_pos);

    let mut to_visit = VecDeque::new();
    to_visit.push_back(src_pos);

    'bfs: loop {
        let player_pos = to_visit
            .pop_front()
            .expect("Couldn't find a path to dest_pos");

        for &new_player_pos in &player_pos.neighbors() {
            if map.grid[new_player_pos] == MapCell::Wall
                || box_grid[new_player_pos]
                || prevs.contains_key(&new_player_pos)
            {
                continue;
            }

            prevs.insert(new_player_pos, player_pos);
            if new_player_pos == dest_pos {
                break 'bfs;
            }
            to_visit.push_back(new_player_pos);
        }
    }

    let positions = backtrack_prevs(&prevs, dest_pos);

    let mut moves = Moves::default();
    let mut iter = positions.iter();
    let mut cur = iter.next().unwrap();
    for next in iter {
        moves.add(Move::new(cur.dir_to(*next), false));
        cur = next;
    }

    moves
}

fn backtrack_prevs<T: Clone + Eq + Hash + Borrow<T>, H: BuildHasher>(
    prevs: &HashMap<T, T, H>,
    final_state: T,
) -> Vec<T> {
    let mut states = Vec::new();
    let mut cur = &final_state;
    loop {
        states.push(cur.clone());
        let prev = &prevs[&cur];
        if prev == cur {
            states.reverse();
            return states;
        }
        cur = prev;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::Level;

    #[test]
    fn backtracking() {
        // Combining moves and pushes in one solution and also testing multiple moves with no push.
        // Currently can't happen but might later if I decide to optimize the moves-based solver.

        let level0 = r"
###########
#$      $.#
#        .#
##$*## #..#
#    #  ###
#    #    #
#  @ #    #
## #### ###
# $   #   #
#     #   #
#         #
###########
".trim_left_matches('\n');
        // 2 steps
        let level1 = r"
###########
#$      $.#
#        .#
##$*## #..#
#    #  ###
# @  #    #
#    #    #
## #### ###
# $   #   #
#     #   #
#         #
###########
".trim_left_matches('\n');
        // 2 steps + 1 push
        let level2 = r"
###########
#$      $.#
#        .#
##$*## #..#
#    #  ###
#    #    #
#    #    #
## #### ###
# @   #   #
# $   #   #
#         #
###########
".trim_left_matches('\n');
        // 1 step
        let level3 = r"
###########
#$      $.#
#        .#
##$*## #..#
#    #  ###
#    #    #
#    #    #
## #### ###
#@    #   #
# $   #   #
#         #
###########
".trim_left_matches('\n');
        // 19 steps + 1 push
        let level4 = r"
###########
#$      @*#
#        .#
##$*## #..#
#    #  ###
#    #    #
#    #    #
## #### ###
#     #   #
# $   #   #
#         #
###########
".trim_left_matches('\n');
        let level0: Level = level0.parse().unwrap();
        let level1: Level = level1.parse().unwrap();
        let level2: Level = level2.parse().unwrap();
        let level3: Level = level3.parse().unwrap();
        let level4: Level = level4.parse().unwrap();

        let mut prevs = HashMap::new();
        prevs.insert(&level0.state, &level0.state);
        prevs.insert(&level1.state, &level0.state);
        prevs.insert(&level2.state, &level1.state);
        prevs.insert(&level3.state, &level2.state);
        prevs.insert(&level4.state, &level3.state);

        let moves = reconstruct_moves(&level0.map, &prevs, &level4.state);
        assert_eq!(moves.to_string(), "ulddDlrrrrddrruuuuuuluuurR");
    }
}