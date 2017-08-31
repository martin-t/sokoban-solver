use std::fmt;
use std::fmt::{Display, Formatter};

use data::*;

#[derive(Debug)]
pub enum ParseError {
    Pos(usize, usize),
    LineLength(usize),
    IncompleteBorder,
    MorePlayers,
    NoPlayer,
    MultipleRemovers,
    RemoverGoals,
    BoxesGoals,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            ParseError::Pos(r, c) => write!(f, "Invalid cell at pos: [{}, {}]", r, c),
            ParseError::LineLength(l) => write!(f, "Wrong line length on line {}", l),
            ParseError::IncompleteBorder => write!(f, "Not surrounded by wall"),
            ParseError::MorePlayers => write!(f, "Too many players"),
            ParseError::NoPlayer => write!(f, "No player"),
            ParseError::MultipleRemovers => write!(f, "Multiple removers - only one allowed"),
            ParseError::RemoverGoals => write!(f, "Both remover and goals"),
            ParseError::BoxesGoals => write!(f, "Different number of boxes and goals"),
        }
    }
}

pub fn parse(puzzle: &str) -> Result<(Map, State), ParseError> {
    let mut map = Vec::new();
    let mut player_pos = None;
    let mut boxes = Vec::new();
    let mut goals = Vec::new();
    let mut remover = None;
    for (r, line) in puzzle.lines().enumerate() {
        map.push(Vec::new());
        let mut chars = line.chars();
        while let (Some(c1), Some(c2)) = (chars.next(), chars.next()) {
            let c = map[r].len();
            match parse_cell(c1, c2) {
                Ok(Cell::Path(PathCell { content, tile: goal })) => {
                    match content {
                        Content::Player => {
                            if player_pos.is_some() {
                                return Err(ParseError::MorePlayers);
                            }
                            player_pos = Some(Pos { r: r as i32, c: c as i32 });
                        }
                        Content::Box => boxes.push(Pos { r: r as i32, c: c as i32 }),
                        _ => {}
                    }
                    if let Tile::Goal = goal {
                        goals.push(Pos { r: r as i32, c: c as i32 });
                    } else if let Tile::Remover = goal {
                        if remover.is_some() { return Err(ParseError::MultipleRemovers); }
                        remover = Some(Pos { r: r as i32, c: c as i32 });
                    }
                    map[r].push(Cell::Path(PathCell {
                        content: Content::Empty,
                        tile: goal,
                    }));
                }
                Ok(cell) => map[r].push(cell),
                Err(_) => return Err(ParseError::Pos(r, c)),
            }
        }
    }

    if player_pos.is_none() {
        return Err(ParseError::NoPlayer);
    }

    if map.is_empty() || map[0].is_empty() {
        return Err(ParseError::IncompleteBorder);
    }

    for i in 1..map.len() {
        if map[i].len() != map[0].len() {
            return Err(ParseError::LineLength(i));
        }
    }

    let rows = map.len();
    let columns = map[0].len();
    for c in 0..columns {
        if map[0][c] != Cell::Wall {
            return Err(ParseError::IncompleteBorder);
        }
        if map[rows - 1][c] != Cell::Wall {
            return Err(ParseError::IncompleteBorder);
        }
    }
    for r in 1..rows - 1 {
        if map[r][0] != Cell::Wall {
            return Err(ParseError::IncompleteBorder);
        }
        if map[r][columns - 1] != Cell::Wall {
            return Err(ParseError::IncompleteBorder);
        }
    }

    if remover.is_some() {
        if goals.len() > 0 {
            Err(ParseError::RemoverGoals)
        } else {
            // TODO with remover
            Ok((Map { map: map, goals: goals, dead_ends: Vec::new() },
                State {
                    player_pos: player_pos.unwrap(),
                    boxes: boxes,
                }))
        }
    } else {
        if goals.len() != boxes.len() {
            Err(ParseError::BoxesGoals)
        } else {
            // TODO with goals
            Ok((Map { map: map, goals: goals, dead_ends: Vec::new() },
                State {
                    player_pos: player_pos.unwrap(),
                    boxes: boxes,
                }))
        }
    }
}

fn parse_cell(c1: char, c2: char) -> Result<Cell, ()> {
    match c1 {
        '<' => if c2 == '>' { Ok(Cell::Wall) } else { Err(()) },
        ' ' => {
            Ok(Cell::Path(PathCell {
                content: Content::Empty,
                tile: parse_cell_goal(c2)?,
            }))
        }
        'B' => {
            Ok(Cell::Path(PathCell {
                content: Content::Box,
                tile: parse_cell_goal(c2)?,
            }))
        }
        'P' => {
            Ok(Cell::Path(PathCell {
                content: Content::Player,
                tile: parse_cell_goal(c2)?,
            }))
        }
        _ => Err(()),
    }
}

fn parse_cell_goal(c: char) -> Result<Tile, ()> {
    match c {
        ' ' => Ok(Tile::Empty),
        '_' => Ok(Tile::Goal),
        'R' => Ok(Tile::Remover),
        _ => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing1() {
        let puzzle = "\
<><><><><>
<> _B_<><>
<>B B <><>
<>  P_<><>
<><><><><>
";
        let (map, state) = parse(puzzle).unwrap();
        assert!(map.with_state(&state).to_string() == puzzle);
    }

    #[test]
    fn parsing2() {
        let puzzle = "\
<><><><><>
<>  B <><>
<>B   <><>
<>  P  R<>
<><><><><>
";
        let (map, state) = parse(puzzle).unwrap();
        assert!(map.with_state(&state).to_string() == puzzle);
    }
}
