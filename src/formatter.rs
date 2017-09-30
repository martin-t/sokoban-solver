use std::fmt;
use std::fmt::{Display, Formatter};

use data::*;

#[derive(Debug, PartialEq)]
pub enum ParseErr {
    Pos(usize, usize),

    // TODO remove
    LineLength(usize),

    MultiplePlayers,
    MultipleRemovers,
    NoPlayer,

    IncompleteBorder,
    UnreachableBoxes,
    UnreachableGoals,
    UnreachableRemover,

    RemoverAndGoals,
    BoxesGoals,
}

impl Display for ParseErr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            ParseErr::Pos(r, c) => write!(f, "Invalid cell at pos: [{}, {}]", r, c),
            ParseErr::LineLength(l) => write!(f, "Wrong line length on line {}", l),
            ParseErr::MultiplePlayers => write!(f, "Too many players"),
            ParseErr::MultipleRemovers => write!(f, "Multiple removers - only one allowed"),
            ParseErr::NoPlayer => write!(f, "No player"),
            ParseErr::IncompleteBorder => write!(f, "Player can exit the level because of missing border"),
            ParseErr::UnreachableBoxes => write!(f, "Boxes that are not on goal but can't be reached"),
            ParseErr::UnreachableGoals => write!(f, "Goals that don't have a box but can't be reached"),
            ParseErr::UnreachableRemover => write!(f, "Remover is not reachable"),
            ParseErr::RemoverAndGoals => write!(f, "Both remover and goals"),
            ParseErr::BoxesGoals => write!(f, "Different number of boxes and goals"),
        }
    }
}

/// Parses my custom format
pub fn parse_custom(level: &str) -> Result<(MapState, State), ParseErr> {
    let level = level.trim(); // no support for weird shapes

    let mut map = Vec::new();
    let mut player_pos = None;
    let mut boxes = Vec::new();
    let mut goals = Vec::new();
    let mut remover = None;
    for (r, line) in level.lines().enumerate() {
        map.push(Vec::new());
        let mut chars = line.chars();
        while let (Some(c1), Some(c2)) = (chars.next(), chars.next()) {
            let c = map[r].len();
            match parse_cell(c1, c2) {
                Ok(Cell::Path(content, goal)) => {
                    match content {
                        Content::Player => {
                            if player_pos.is_some() {
                                return Err(ParseErr::MultiplePlayers);
                            }
                            player_pos = Some(Pos { r: r as i32, c: c as i32 });
                        }
                        Content::Box => boxes.push(Pos { r: r as i32, c: c as i32 }),
                        _ => {}
                    }
                    if let Tile::Goal = goal {
                        goals.push(Pos { r: r as i32, c: c as i32 });
                    } else if let Tile::Remover = goal {
                        if remover.is_some() { return Err(ParseErr::MultipleRemovers); }
                        remover = Some(Pos { r: r as i32, c: c as i32 });
                    }
                    map[r].push(Cell::Path(Content::Empty, goal));
                }
                Ok(cell) => map[r].push(cell),
                Err(_) => return Err(ParseErr::Pos(r, c)),
            }
        }
    }

    if player_pos.is_none() {
        return Err(ParseErr::NoPlayer);
    }

    if map.is_empty() || map[0].is_empty() {
        return Err(ParseErr::IncompleteBorder);
    }

    for i in 1..map.len() {
        if map[i].len() != map[0].len() {
            return Err(ParseErr::LineLength(i));
        }
    }

    let rows = map.len();
    let columns = map[0].len();
    for c in 0..columns {
        if map[0][c] != Cell::Wall {
            return Err(ParseErr::IncompleteBorder);
        }
        if map[rows - 1][c] != Cell::Wall {
            return Err(ParseErr::IncompleteBorder);
        }
    }
    for r in 1..rows - 1 {
        if map[r][0] != Cell::Wall {
            return Err(ParseErr::IncompleteBorder);
        }
        if map[r][columns - 1] != Cell::Wall {
            return Err(ParseErr::IncompleteBorder);
        }
    }

    if remover.is_some() {
        if goals.len() > 0 {
            Err(ParseErr::RemoverAndGoals)
        } else {
            // TODO with remover
            Ok((MapState { map: map, goals: goals, dead_ends: Vec::new() },
                State {
                    player_pos: player_pos.unwrap(),
                    boxes: boxes,
                }))
        }
    } else {
        if goals.len() != boxes.len() {
            Err(ParseErr::BoxesGoals)
        } else {
            // TODO with goals
            Ok((MapState { map: map, goals: goals, dead_ends: Vec::new() },
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
            Ok(Cell::Path(Content::Empty, parse_cell_goal(c2)?))
        }
        'B' => {
            Ok(Cell::Path(Content::Box, parse_cell_goal(c2)?))
        }
        'P' => {
            Ok(Cell::Path(Content::Player, parse_cell_goal(c2)?))
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

/// Parses (a subset of) the format described [here](http://www.sokobano.de/wiki/index.php?title=Level_format)
pub fn parse_xsb(level: &str) -> Result<(Map, State), ParseErr> {
    let level = level.trim_matches('\n');

    let mut map = Vec::new();
    let mut goals = Vec::new();
    let mut remover = None;
    let mut boxes = Vec::new();
    let mut player_pos = None;

    for (r, line) in level.lines().enumerate() {
        let mut line_tiles = Vec::new();
        for (c, char) in line.chars().enumerate() {
            let tile = match char {
                // need to create MapState because of reachability checks
                '#' => {
                    //Cell::Wall
                    MapCell::Wall
                }
                'p' | '@' => {
                    if player_pos.is_some() {
                        return Err(ParseErr::MultiplePlayers);
                    }
                    player_pos = Some(Pos::new(r, c));
                    //Cell::Path(Content::Player, Tile::Empty)
                    MapCell::Empty
                }
                'P' | '+' => {
                    if player_pos.is_some() {
                        return Err(ParseErr::MultiplePlayers);
                    }
                    player_pos = Some(Pos::new(r, c));
                    goals.push(Pos::new(r, c));
                    //Cell::Path(Content::Player, Tile::Goal)
                    MapCell::Goal
                }
                'b' | '$' => {
                    boxes.push(Pos::new(r, c));
                    //Cell::Path(Content::Box, Tile::Empty)
                    MapCell::Empty
                }
                'B' | '*' => {
                    boxes.push(Pos::new(r, c));
                    goals.push(Pos::new(r, c));
                    //Cell::Path(Content::Box, Tile::Goal)
                    MapCell::Goal
                }
                'r' => {
                    if remover.is_some() {
                        return Err(ParseErr::MultipleRemovers);
                    }
                    remover = Some(Pos::new(r, c));
                    //Cell::Path(Content::Empty, Tile::Remover)
                    MapCell::Remover
                }
                'R' => {
                    // this is player on remover, box on remover makes no sense
                    if player_pos.is_some() {
                        return Err(ParseErr::MultiplePlayers);
                    }
                    player_pos = Some(Pos::new(r, c));
                    if remover.is_some() {
                        return Err(ParseErr::MultipleRemovers);
                    }
                    remover = Some(Pos::new(r, c));
                    //Cell::Path(Content::Player, Tile::Remover)
                    MapCell::Remover
                }
                '.' => {
                    goals.push(Pos::new(r, c));
                    //Cell::Path(Content::Empty, Tile::Goal)
                    MapCell::Goal
                }
                ' ' | '-' | '_' => {
                    //Cell::Path(Content::Empty, Tile::Empty)
                    MapCell::Empty
                }
                _ => return Err(ParseErr::Pos(r, c))
            };
            line_tiles.push(tile);
        }
        map.push(line_tiles)
    }

    if player_pos.is_none() {
        return Err(ParseErr::NoPlayer);
    }
    let player_pos = player_pos.unwrap();

    let mut to_visit = vec![(player_pos.r, player_pos.c)];
    let mut visited = Vec::new();
    for row in map.iter() {
        visited.push(vec![false; row.len()]);
    }

    while !to_visit.is_empty() {
        let (r, c) = to_visit.pop().unwrap();
        visited[r as usize][c as usize] = true;

        let neighbors = [(r + 1, c), (r - 1, c), (r, c + 1), (r, c - 1)];
        for &(nr, nc) in neighbors.iter() {
            // this is the only place we need to check bounds
            // everything after that will be surrounded by walls
            // TODO make sure we're not wasting time bounds checking anywhere else
            if nr < 0 || nc < 0 || nr as usize >= map.len() || nc as usize >= map[nr as usize].len() {
                // we got out of bounds without hitting a wall
                return Err(ParseErr::IncompleteBorder);
            }
            if !visited[nr as usize][nc as usize] && map[nr as usize][nc as usize] != MapCell::Wall {
                //if !visited[nr as usize][nc as usize] && map[nr as usize][nc as usize] != Cell::Wall {
                to_visit.push((nr, nc));
            }
        }
    }

    if let Some(pos) = remover {
        if !visited[pos.r as usize][pos.c as usize] {
            return Err(ParseErr::UnreachableRemover);
        }
    }
    let mut reachable_goals = Vec::new();
    let mut reachable_boxes = Vec::new();
    for &pos in boxes.iter() {
        let (r, c) = (pos.r as usize, pos.c as usize);
        if visited[r][c] {
            reachable_boxes.push(pos);
        } else if !goals.contains(&pos) {
            return Err(ParseErr::UnreachableBoxes);
        }
    }
    for &pos in goals.iter() {
        let (r, c) = (pos.r as usize, pos.c as usize);
        if visited[r][c] {
            reachable_goals.push(pos);
        } else if !boxes.contains(&pos) {
            return Err(ParseErr::UnreachableGoals);
        }
    }

    if remover.is_some() {
        if goals.len() > 0 {
            return Err(ParseErr::RemoverAndGoals);
        }
    } else {
        if reachable_boxes.len() != reachable_goals.len() {
            return Err(ParseErr::BoxesGoals);
        }
    }

    Ok((Map::new(map, reachable_goals),
        State::new(player_pos, reachable_boxes)))
    /*Ok((MapState::new(map, goals), // TODO clear contents
        State::new(player_pos, boxes)))*/
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_goals() {
        let level = r"
<><><><><>
<> _B_<><>
<>B B <><>
<>  P_<><>
<><><><><>
";
        assert_success_custom(level);
    }

    #[test]
    fn parsing_remover() {
        let level = r"
<><><><><>
<>  B <><>
<>B   <><>
<>  P  R<>
<><><><><>
";
        assert_success_custom(level);
    }

    #[test]
    fn only_player() {
        let level = r"
<><><>
<>P <>
<><><>
";
        assert_success_custom(level);
    }

    #[test]
    fn nothing() {
        let level = r"
<><><>
<>  <>
<><><>
";
        assert_failure_custom(level, ParseErr::NoPlayer);
    }

    #[test]
    fn remover_and_goal() {
        let level = r"
<><><><>
<>P  R<>
<> _  <>
<><><><>
";
        assert_failure_custom(level, ParseErr::RemoverAndGoals);
    }

    #[test]
    fn xsb1() {
        let level = r"
#####
#@$.#
#####
";
        assert_success_xsb(level);
    }

    #[test]
    fn xsb2() {
        let level = r"
*###*
#@$.#
*###*
";
        assert_success_xsb(level);
    }

    #[test]
    fn xsb3() {
        let level = r"
    #####
    #   #
    #$  #
  ###  $##
  #  $ $ #
### # ## #   ######
#   # ## #####  ..#
# $  $          ..#
##### ### #@##  ..#
    #     #########
    #######
";
        assert_success_xsb(level);
    }

    #[test]
    fn xsb_f1() {
        let level = r"
########
#@$.#$.#
########
";
        assert_failure_xsb(level, ParseErr::UnreachableBoxes);
    }

    fn assert_success_custom(input_level: &str) {
        let (map, state) = parse_custom(input_level).unwrap();
        assert_eq!(map.with_state(&state).to_string(), input_level.trim_left());
    }

    fn assert_failure_custom(input_level: &str, expected_err: ParseErr) {
        assert_eq!(parse_custom(input_level).unwrap_err(), expected_err);
    }

    fn assert_success_xsb(input_level: &str) {
        parse_xsb(input_level).unwrap(); // TODO write out, compare
    }

    fn assert_failure_xsb(input_level: &str, expected_err: ParseErr) {
        assert_eq!(parse_xsb(input_level).unwrap_err(), expected_err);
    }
}

