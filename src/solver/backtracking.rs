use std::borrow::Borrow;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{BuildHasher, Hash};

use crate::data::{MapCell, Pos};
use crate::map::Map;
use crate::moves::{Move, Moves};
use crate::state::State;

// Terminology:
// move = changing player position by one cell
// push = a move that changes a box position
// step = a move that doesn't change a box position

crate fn backtrack_prevs<T: Clone + Eq + Hash + Borrow<T>, H: BuildHasher>(
    prevs: &HashMap<T, T, H>,
    final_state: T,
) -> Vec<T> {
    let mut states = Vec::new();
    let mut cur = &final_state;
    loop {
        states.push(cur.clone());
        let prev = &prevs[cur];
        if prev == cur {
            states.reverse();
            return states;
        }
        cur = prev;
    }
}

// dynamic dispatch has no perf impact here
crate fn reconstruct_moves(
    map: &dyn Map,
    real_initial_player_pos: Pos,
    states: &[&State],
) -> Moves {
    let mut moves = Moves::default();
    let mut iter = states.iter();
    let mut cur_state = iter.next().expect("There must be at least one state");

    // the states we're getting here might have been normalized (depending on solving method)
    // so we need to track the actual player positions (determined by how boxes are pushed)
    let mut real_player_pos = real_initial_player_pos;

    for next_state in iter {
        let (new_moves, new_player_pos) =
            moves_between_states(map, real_player_pos, cur_state, next_state);
        moves.extend(&new_moves);
        real_player_pos = new_player_pos;
        cur_state = next_state;
    }

    moves
}

/// The difference between them must be any number of steps and one push
fn moves_between_states(
    map: &dyn Map,
    old_player_pos: Pos,
    old: &State,
    new: &State,
) -> (Moves, Pos) {
    let old_boxes: HashSet<_> = old.boxes.iter().collect();
    let new_boxes: HashSet<_> = new.boxes.iter().collect();

    let mut old_iter = old_boxes.difference(&new_boxes);
    let mut new_iter = new_boxes.difference(&old_boxes);

    let old_box_pos = **old_iter
        .next()
        .expect("There must be exactly one push between states");
    assert!(
        old_iter.next().is_none(),
        "Only one box can change its position at a time"
    );

    let new_box_pos = match new_iter.next() {
        None => map
            .remover()
            .expect("A box disappeared so there must be a remover"),
        Some(&&pos) => pos,
    };
    assert!(
        new_iter.next().is_none(),
        "Only one box can change its position at a time"
    );

    let push_dir = old_box_pos.dir_to(new_box_pos);
    let player_pos_before_push = old_box_pos + push_dir.inverse();
    let mut moves = player_steps(map, old, old_player_pos, player_pos_before_push);
    moves.add(Move::new(push_dir, true));

    (moves, old_box_pos)
}

fn player_steps(map: &dyn Map, state: &State, src_pos: Pos, dest_pos: Pos) -> Moves {
    if src_pos == dest_pos {
        // because it's not a proper BFS with an open set
        return Moves::default();
    }

    let mut box_grid = map.grid().scratchpad();
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
            if map.grid()[new_player_pos] == MapCell::Wall
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
    let mut cur = iter.next().expect("There must be at least one position");
    for next in iter {
        moves.add(Move::new(cur.dir_to(*next), false));
        cur = next;
    }

    moves
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Format;
    use crate::level::Level;

    #[test]
    fn backtracking() {
        // this mixes normalized positions with actual ones which can't normally happen
        // but it shouldn't affect anything

        let level_initial = r"
###########
# $     $.#
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
"
        .trim_start_matches('\n');

        let level_state1 = r"
###########
# $     $.#
#        .#
##$*## #..#
#@   #  ###
#    #    #
#    #    #
## #### ###
# $   #   #
#     #   #
#         #
###########
"
        .trim_start_matches('\n');

        let level_state2 = r"
###########
#@$     $.#
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
"
        .trim_start_matches('\n');

        let level_state3 = r"
###########
#$@     $.#
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
"
        .trim_start_matches('\n');

        let level_state4 = r"
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
"
        .trim_start_matches('\n');

        let expected_pushes = r"
###########
# $     $.#
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

###########
# $     $.#
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

###########
#$@     $.#
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

"
        .trim_start_matches('\n');

        let level_initial: Level = level_initial.parse().unwrap();
        let level_state1: Level = level_state1.parse().unwrap();
        let level_state2: Level = level_state2.parse().unwrap();
        let level_state3: Level = level_state3.parse().unwrap();
        let level_state4: Level = level_state4.parse().unwrap();

        let mut prevs = HashMap::new();
        prevs.insert(&level_state1.state, &level_state1.state);
        prevs.insert(&level_state2.state, &level_state1.state);
        prevs.insert(&level_state3.state, &level_state2.state);
        prevs.insert(&level_state4.state, &level_state3.state);

        let states = backtrack_prevs(&prevs, &level_state4.state);
        let moves = reconstruct_moves(&level_state1.map, level_initial.state.player_pos, &states);
        assert_eq!(moves.to_string(), "ddDrrrddrruuuuuuluuulllLrrrrrR");

        let solution_pushes = level_initial
            .format_solution(Format::Xsb, &moves, false)
            .to_string();
        assert_eq!(solution_pushes, expected_pushes);
    }
}
