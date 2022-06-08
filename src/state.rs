use crate::data::Pos;

// TODO private to keep sorted?
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub(crate) struct State {
    pub(crate) player_pos: Pos,
    pub(crate) boxes: Vec<Pos>,
}

impl State {
    pub(crate) fn new(player_pos: Pos, mut boxes: Vec<Pos>) -> State {
        // TODO use binary search when inserting instead (a different data structure might be even better)
        boxes.sort(); // sort to detect equal states when we reorder boxes
        State { player_pos, boxes }
    }
}
