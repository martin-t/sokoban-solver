use crate::data::Pos;

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
crate struct State {
    crate player_pos: Pos,
    crate boxes: Vec<Pos>,
}

impl State {
    crate fn new(player_pos: Pos, mut boxes: Vec<Pos>) -> State {
        // TODO use binary search when inserting instead
        boxes.sort(); // sort to detect equal states when we reorder boxes
        State { player_pos, boxes }
    }
}
