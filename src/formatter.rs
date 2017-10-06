use std::fmt;
use std::fmt::{Display, Formatter};

use data::{Map, MapCell, Pos, State};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Format {
    Custom,
    Xsb,
}

#[derive(Debug, PartialEq)]
pub enum ParseErr {
    Pos(usize, usize),

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

pub fn parse(level: &str, format: Format) -> Result<(Map, State), ParseErr> {
    let level = level.trim_matches('\n');


    let (mut map, goals, remover, boxes, player_pos) = match format {
        Format::Custom => parse_custom(level)?,
        Format::Xsb => parse_xsb(level)?,
    };

    if player_pos.is_none() {
        return Err(ParseErr::NoPlayer);
    }
    let player_pos = player_pos.unwrap();

    let mut to_visit = vec![(player_pos.r, player_pos.c)];
    let mut visited = Map::create_scratch_map(&map, false);

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

    // to avoid errors with some code that iterates through all non-walls
    for r in 0..map.len() {
        for c in 0..map[r].len() {
            if !visited[r][c] {
                map[r][c] = MapCell::Wall;
            }
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
}

/// Parses my custom format
fn parse_custom(level: &str)
                -> Result<
                    (Vec<Vec<MapCell>>, Vec<Pos>, Option<Pos>, Vec<Pos>, Option<Pos>),
                    ParseErr>
{
    let mut map = Vec::new();
    let mut goals = Vec::new();
    let mut remover = None;
    let mut boxes = Vec::new();
    let mut player_pos = None;

    for (r, line) in level.lines().enumerate() {
        map.push(Vec::new());
        let mut chars = line.chars();
        while let (Some(c1), Some(c2)) = (chars.next(), chars.next()) {
            let c = map[r].len();

            match c1 {
                '<' => {
                    if c2 != '>' { return Err(ParseErr::Pos(r, c)); }
                    map[r].push(MapCell::Wall);
                    continue; // skip parsing c2
                }
                ' ' => {}
                'B' => boxes.push(Pos::new(r, c)),
                'P' => {
                    if player_pos.is_some() { return Err(ParseErr::MultiplePlayers); }
                    player_pos = Some(Pos::new(r, c));
                }
                _ => return Err(ParseErr::Pos(r, c)),
            }
            match c2 {
                ' ' => map[r].push(MapCell::Empty),
                '_' => {
                    goals.push(Pos::new(r, c));
                    map[r].push(MapCell::Goal);
                }
                'R' => {
                    if remover.is_some() { return Err(ParseErr::MultipleRemovers); }
                    remover = Some(Pos::new(r, c));
                    map[r].push(MapCell::Remover);
                }
                _ => return Err(ParseErr::Pos(r, c)),
            }
        }
    }

    Ok((map, goals, remover, boxes, player_pos))
}

/// Parses (a subset of) the format described [here](http://www.sokobano.de/wiki/index.php?title=Level_format)
fn parse_xsb(level: &str)
             -> Result<
                 (Vec<Vec<MapCell>>, Vec<Pos>, Option<Pos>, Vec<Pos>, Option<Pos>),
                 ParseErr>
{
    let mut map = Vec::new();
    let mut goals = Vec::new();
    let mut remover = None;
    let mut boxes = Vec::new();
    let mut player_pos = None;

    for (r, line) in level.lines().enumerate() {
        let mut line_tiles = Vec::new();
        for (c, char) in line.chars().enumerate() {
            let tile = match char {
                '#' => {
                    MapCell::Wall
                }
                'p' | '@' => {
                    if player_pos.is_some() {
                        return Err(ParseErr::MultiplePlayers);
                    }
                    player_pos = Some(Pos::new(r, c));
                    MapCell::Empty
                }
                'P' | '+' => {
                    if player_pos.is_some() {
                        return Err(ParseErr::MultiplePlayers);
                    }
                    player_pos = Some(Pos::new(r, c));
                    goals.push(Pos::new(r, c));
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
                    MapCell::Goal
                }
                'r' => {
                    if remover.is_some() {
                        return Err(ParseErr::MultipleRemovers);
                    }
                    remover = Some(Pos::new(r, c));
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
                    MapCell::Remover
                }
                '.' => {
                    goals.push(Pos::new(r, c));
                    MapCell::Goal
                }
                ' ' | '-' | '_' => {
                    MapCell::Empty
                }
                _ => return Err(ParseErr::Pos(r, c))
            };
            line_tiles.push(tile);
        }
        map.push(line_tiles)
    }

    Ok((map, goals, remover, boxes, player_pos))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_goals() {
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
    fn custom_remover() {
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
    fn custom_player() {
        let level = r"
<><><>
<>P <>
<><><>
";
        assert_success_custom(level);
    }

    #[test]
    fn custom_f1() {
        let level = r"
<><><>
<>  <>
<><><>
";
        assert_failure_custom(level, ParseErr::NoPlayer);
    }

    #[test]
    fn custom_f2() {
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
        let (map, state) = parse(input_level, Format::Custom).unwrap();
        assert_eq!(map.empty_map_state().with_state(&state).to_string(), input_level.trim_left());
    }

    fn assert_failure_custom(input_level: &str, expected_err: ParseErr) {
        assert_eq!(parse(input_level, Format::Custom).unwrap_err(), expected_err);
    }

    fn assert_success_xsb(input_level: &str) {
        parse(input_level, Format::Xsb).unwrap(); // TODO write out, compare
    }

    fn assert_failure_xsb(input_level: &str, expected_err: ParseErr) {
        assert_eq!(parse(input_level, Format::Xsb).unwrap_err(), expected_err);
    }
}

