use std::fmt::{self, Debug, Display, Formatter};

use crate::config::Format;
use crate::data::MapCell;
use crate::map::Map;
use crate::moves::Moves;
use crate::state::State;

pub struct SolutionFormatter<'a> {
    map: &'a dyn Map,
    initial_state: &'a State,
    moves: &'a Moves,
    include_steps: bool,
    format: Format,
}

impl<'a> SolutionFormatter<'a> {
    crate fn new(
        map: &'a dyn Map,
        initial_state: &'a State,
        moves: &'a Moves,
        include_steps: bool,
        format: Format,
    ) -> Self {
        Self {
            map,
            initial_state,
            moves,
            include_steps,
            format,
        }
    }
}

impl Display for SolutionFormatter<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}",
            self.map.format_with_state(self.format, &self.initial_state)
        )?;
        let mut last_state = self.initial_state.clone();
        for &mov in self.moves {
            // instead of verifying moves, they could have a reference to the map
            // to prevent the user from passing moves from a different level but this is a nice sanity check

            let new_player_pos = last_state.player_pos + mov.dir;
            assert_ne!(
                self.map.grid()[new_player_pos],
                MapCell::Wall,
                "new_player_pos: {:?}",
                new_player_pos
            );

            let mut new_boxes = last_state.boxes.clone();
            if mov.is_push {
                let new_box_pos = new_player_pos + mov.dir;
                assert_ne!(self.map.grid()[new_box_pos], MapCell::Wall);
                assert!(!new_boxes.as_slice().contains(&new_box_pos));
                let box_index = new_boxes
                    .iter()
                    .position(|&b| b == new_player_pos)
                    .expect("Move is a push but there is no box");
                new_boxes[box_index] = new_box_pos;
                if let Some(rem_pos) = self.map.remover() {
                    if new_box_pos == rem_pos {
                        new_boxes.remove(box_index);
                    }
                }
            } else {
                assert!(!new_boxes.as_slice().contains(&new_player_pos));
            }

            let new_state = State::new(new_player_pos, new_boxes);

            if mov.is_push || self.include_steps {
                writeln!(f, "{}", self.map.format_with_state(self.format, &new_state))?;
            }

            last_state = new_state;
        }
        Ok(())
    }
}

impl Debug for SolutionFormatter<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
