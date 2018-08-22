use std::fmt::{self, Debug, Display, Formatter};

use crate::config::Format;
use crate::map::{GoalMap, Map};
use crate::moves::Moves;
use crate::state::State;

pub struct SolutionFormatter<'a> {
    map: &'a GoalMap,
    initial_state: &'a State,
    moves: &'a Moves,
    include_steps: bool,
    format: Format,
}

impl<'a> SolutionFormatter<'a> {
    crate fn new(
        map: &'a GoalMap,
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
        // TODO verify moves (somebody could pass moves from a different level)

        writeln!(
            f,
            "{}",
            self.map.format_with_state(self.format, &self.initial_state)
        )?;
        let mut last_state = self.initial_state.clone();
        for &mov in self.moves {
            let new_player_pos = last_state.player_pos + mov.dir;
            let new_boxes = last_state
                .boxes
                .iter()
                .cloned()
                .map(|b| if b == new_player_pos { b + mov.dir } else { b })
                .collect();
            let new_state = State::new(new_player_pos, new_boxes);
            if mov.is_push || self.include_steps {
                writeln!(f, "{}", self.map.format_with_state(self.format, &new_state));
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
