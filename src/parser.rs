use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use data::{Format, MapCell, Pos, State, MAX_SIZE};
use level::Level;
use map::GoalMap;
use vec2d::Vec2d;

#[derive(Debug, PartialEq)]
crate enum ParserErr {
    Pos(usize, usize),
    TooLarge,
    MultiplePlayers,
    MultipleRemovers,
    BoxOnRemover,
    NoPlayer,
    RemoverAndGoals,
}

impl Display for ParserErr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            ParserErr::Pos(r, c) => write!(f, "Invalid cell at pos: [{}, {}]", r, c),
            ParserErr::TooLarge => write!(f, "Map larger than 255 rows/columns"),
            ParserErr::MultiplePlayers => write!(f, "More than one player"),
            ParserErr::MultipleRemovers => write!(f, "Multiple removers - only one allowed"),
            ParserErr::BoxOnRemover => write!(f, "Box on remover"),
            ParserErr::NoPlayer => write!(f, "No player"),
            ParserErr::RemoverAndGoals => write!(f, "Both remover and goals"),
        }
    }
}

impl FromStr for Level {
    type Err = ParserErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse(s)
    }
}

crate fn parse(level: &str) -> Result<Level, ParserErr> {
    if level.trim_left().contains('<') {
        parse_format(level, Format::Custom)
    } else {
        parse_format(level, Format::Xsb)
    }
}

crate fn parse_format(level: &str, format: Format) -> Result<Level, ParserErr> {
    // trim so we can specify levels using raw strings more easily
    let level = level.trim_matches('\n').trim_right();

    let (grid, goals, remover, boxes, player_pos) = match format {
        Format::Custom => parse_custom(level)?,
        Format::Xsb => parse_xsb(level)?,
    };
    let player_pos = player_pos.ok_or(ParserErr::NoPlayer)?;
    let grid = Vec2d::new(&grid);

    if let Some(_remover) = remover {
        if goals.len() > 0 {
            Err(ParserErr::RemoverAndGoals)
        } else {
            unimplemented!()
            /*Ok(Level::new(
                RemoverMap::new(grid, remover),
                State::new(player_pos, boxes)))*/
        }
    } else {
        Ok(Level::new(
            GoalMap::new(grid, goals),
            State::new(player_pos, boxes),
        ))
    }
}

/// Parses my custom format
fn parse_custom(
    level: &str,
) -> Result<
    (
        Vec<Vec<MapCell>>,
        Vec<Pos>,
        Option<Pos>,
        Vec<Pos>,
        Option<Pos>,
    ),
    ParserErr,
> {
    let mut grid = Vec::new();
    let mut goals = Vec::new();
    let mut remover = None;
    let mut boxes = Vec::new();
    let mut player_pos = None;

    for (r, line) in level.lines().enumerate() {
        if r > MAX_SIZE {
            return Err(ParserErr::TooLarge);
        }
        grid.push(Vec::new());
        let mut chars = line.chars();
        while let (Some(c1), Some(c2)) = (chars.next(), chars.next()) {
            let c = grid[r].len();
            if c > MAX_SIZE {
                return Err(ParserErr::TooLarge);
            }
            let pos = Pos::new(r as u8, c as u8);

            let mut has_box = false;
            match c1 {
                '<' => {
                    if c2 != '>' {
                        return Err(ParserErr::Pos(r, c));
                    }
                    grid[r].push(MapCell::Wall);
                    continue; // skip parsing c2
                }
                ' ' => {}
                'B' => {
                    boxes.push(pos);
                    has_box = true;
                }
                'P' => {
                    if player_pos.is_some() {
                        return Err(ParserErr::MultiplePlayers);
                    }
                    player_pos = Some(pos);
                }
                _ => return Err(ParserErr::Pos(r, c)),
            }
            match c2 {
                ' ' => grid[r].push(MapCell::Empty),
                '_' => {
                    goals.push(pos);
                    grid[r].push(MapCell::Goal);
                }
                'R' => {
                    if remover.is_some() {
                        return Err(ParserErr::MultipleRemovers);
                    }
                    if has_box {
                        return Err(ParserErr::BoxOnRemover);
                    }
                    remover = Some(pos);
                    grid[r].push(MapCell::Remover);
                }
                _ => return Err(ParserErr::Pos(r, c)),
            }
        }
    }

    Ok((grid, goals, remover, boxes, player_pos))
}

/// Parses (a subset of) the format described [here](http://www.sokobano.de/wiki/index.php?title=Level_format)
fn parse_xsb(
    level: &str,
) -> Result<
    (
        Vec<Vec<MapCell>>,
        Vec<Pos>,
        Option<Pos>,
        Vec<Pos>,
        Option<Pos>,
    ),
    ParserErr,
> {
    let mut grid = Vec::new();
    let mut goals = Vec::new();
    let mut remover = None;
    let mut boxes = Vec::new();
    let mut player_pos = None;

    for (r, line) in level.lines().enumerate() {
        if r > MAX_SIZE {
            return Err(ParserErr::TooLarge);
        }
        let mut line_tiles = Vec::new();
        for (c, cur_char) in line.chars().enumerate() {
            if c > MAX_SIZE {
                return Err(ParserErr::TooLarge);
            }
            let pos = Pos::new(r as u8, c as u8);

            let tile = match cur_char {
                '#' => MapCell::Wall,
                'p' | '@' => {
                    if player_pos.is_some() {
                        return Err(ParserErr::MultiplePlayers);
                    }
                    player_pos = Some(pos);
                    MapCell::Empty
                }
                'P' | '+' => {
                    if player_pos.is_some() {
                        return Err(ParserErr::MultiplePlayers);
                    }
                    player_pos = Some(pos);
                    goals.push(pos);
                    MapCell::Goal
                }
                'b' | '$' => {
                    boxes.push(pos);
                    //Cell::Path(Content::Box, Tile::Empty)
                    MapCell::Empty
                }
                'B' | '*' => {
                    boxes.push(pos);
                    goals.push(pos);
                    MapCell::Goal
                }
                'r' => {
                    if remover.is_some() {
                        return Err(ParserErr::MultipleRemovers);
                    }
                    remover = Some(pos);
                    MapCell::Remover
                }
                'R' => {
                    // this is player on remover, box on remover makes no sense
                    if player_pos.is_some() {
                        return Err(ParserErr::MultiplePlayers);
                    }
                    player_pos = Some(pos);
                    if remover.is_some() {
                        return Err(ParserErr::MultipleRemovers);
                    }
                    remover = Some(pos);
                    MapCell::Remover
                }
                '.' => {
                    goals.push(pos);
                    MapCell::Goal
                }
                ' ' | '-' | '_' => MapCell::Empty,
                _ => return Err(ParserErr::Pos(r, c)),
            };
            line_tiles.push(tile);
        }
        grid.push(line_tiles)
    }

    Ok((grid, goals, remover, boxes, player_pos))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_fail_empty() {
        let level = "";
        assert_failure(level, ParserErr::NoPlayer);
    }

    #[test]
    fn custom_fail_no_player() {
        let level = r"
<><><>
<>  <>
<><><>
";
        assert_failure(level, ParserErr::NoPlayer);
    }

    #[test]
    fn custom_fail_remover_and_goals() {
        let level = r"
<><><><>
<>P  R<>
<> _  <>
<><><><>
";
        assert_failure(level, ParserErr::RemoverAndGoals);
    }

    #[test]
    fn custom_fail_box_on_remover() {
        let level = r"
<><><><>
<>P BR<>
<><><><>
";
        assert_failure(level, ParserErr::BoxOnRemover);
    }

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
    #[should_panic] // TODO remove when remover is implemented
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
    fn xsb_fail_pos() {
        let level = r"
#####
#@X.#
#####
";
        assert_failure(level, ParserErr::Pos(1, 2));
    }

    #[test]
    fn xsb_simplest() {
        let level = r"
#####
#@$.#
#####
";
        assert_success_xsb(level);
    }

    #[test]
    fn xsb_corner_boxes() {
        // TODO also test solution shows the corner boxes
        let level = r"
*###*
#@$.#
*###*
";
        assert_success_xsb(level);
    }

    #[test]
    fn xsb_original_1() {
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

    fn assert_failure(input_level: &str, expected_err: ParserErr) {
        // shared for XSB and custom because no need to print here
        assert_eq!(input_level.parse::<Level>().unwrap_err(), expected_err);
    }

    fn assert_success_custom(input_level: &str) {
        let level = parse_format(input_level, Format::Custom).unwrap();
        assert_eq!(
            level.custom().to_string(),
            input_level.trim_left_matches('\n')
        );
    }

    fn assert_success_xsb(input_level: &str) {
        let level = parse_format(input_level, Format::Xsb).unwrap();
        assert_eq!(level.to_string(), input_level.trim_left_matches('\n'));
    }
}
