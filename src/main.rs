use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::env;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::prelude::*;
use std::ops::Add;
use std::process;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Cell {
    Wall,
    Path(PathCell),
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PathCell {
    content: Content,
    goal: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Content {
    Empty,
    Player,
    Box,
}

#[derive(Debug, Clone)]
struct Map {
    map: Vec<Vec<Cell>>,
    goals: Vec<Pos>,
    dead_ends: Vec<Vec<bool>>,
}

impl Map {
    fn at(&self, pos: Pos) -> &Cell {
        &self.map[pos.r as usize][pos.c as usize]
    }

    fn at_mut(&mut self, pos: Pos) -> &mut Cell {
        &mut self.map[pos.r as usize][pos.c as usize]
    }

    fn with_state(self, state: &State) -> Map {
        self.with_boxes(state).with_player(state)
    }

    fn with_boxes(mut self, state: &State) -> Map {
        for pos in &state.boxes {
            if let Cell::Path(ref mut pc) = *self.at_mut(*pos) {
                pc.content = Content::Box;
            } else {
                unreachable!();
            }
        }
        self
    }

    fn with_player(mut self, state: &State) -> Map {
        if let Cell::Path(ref mut pc) = *self.at_mut(state.player_pos) {
            pc.content = Content::Player;
        } else {
            unreachable!();
        }
        self
    }

    fn to_string(&self) -> String {
        let mut res = String::new();
        for row in &self.map {
            for cell in row {
                match *cell {
                    Cell::Wall => res += "<>",
                    Cell::Path(ref path) => {
                        match path.content {
                            Content::Empty => res += " ",
                            Content::Box => res += "B",
                            Content::Player => res += "P",
                        }
                        match path.goal {
                            true => res += "_",
                            false => res += " ",
                        }
                    }
                }
            }
            res += "\n";
        }
        res
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct State {
    player_pos: Pos,
    boxes: Vec<Pos>,
}

#[derive(Debug)]
struct SearchState {
    state: State,
    prev: Option<State>,
    dist: i32,
    h: i32,
}

impl Ord for SearchState {
    fn cmp(&self, other: &Self) -> Ordering {
        // intentionally reversed for BinaryHeap
        //other.heuristic().cmp(&self.heuristic())
        (other.dist + other.h).cmp(&(self.dist + self.h))
    }
}

impl PartialOrd for SearchState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for SearchState {}

impl PartialEq for SearchState {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Pos {
    r: i32,
    c: i32,
}

impl Pos {
    fn dist(self, other: Pos) -> i32 {
        (self.r - other.r).abs() + (self.c - other.c).abs()
    }
}

#[derive(Debug, Clone, Copy)]
struct Dir {
    r: i32,
    c: i32,
}

impl Add<Dir> for Pos {
    type Output = Pos;

    fn add(self, dir: Dir) -> Pos {
        Pos { r: self.r + dir.r, c: self.c + dir.c }
    }
}

#[derive(Debug)]
enum ParseError {
    Pos(usize, usize),
    LineLength(usize),
    IncompleteBorder,
    MorePlayers,
    NoPlayer,
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
            ParseError::BoxesGoals => write!(f, "Diferent number of boxes and goals"),
        }
    }
}

const UP: Dir = Dir { r: -1, c: 0 };
const RIGHT: Dir = Dir { r: 0, c: 1 };
const DOWN: Dir = Dir { r: 1, c: 0 };
const LEFT: Dir = Dir { r: 0, c: -1 };
const DIRECTIONS: [Dir; 4] = [UP, RIGHT, DOWN, LEFT];

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: sokoban-solver <path_to_puzzle>");
        process::exit(1);
    }
    let path = &args[1];

    let puzzle = read_puzzle(path).unwrap_or_else(|err| {
        let current_dir = env::current_dir().unwrap();
        println!("Can't read file {} in {:?}: {}", path, current_dir, err);
        process::exit(1);
    });

    let (mut map, initial_state) = parse(&puzzle).unwrap_or_else(|err| {
        println!("Failed to parse: {}", err);
        process::exit(1);
    });
    /*println!("Empty map:\n{}", map.to_string());
    println!("Initial state:\n{}",
             map.clone().with_state(&initial_state).to_string());*/
    //println!("Expanding: {:?}", expand(&map, &initial_state));

    mark_dead_ends(&mut map);

    match search(&map, &initial_state, true) {
        Some(path) => {
            println!("Found solution:");
            for state in &path {
                println!("{}", map.clone().with_state(state).to_string());
            }
            println!("{} moves", &path.len() - 1);
        }
        None => println!("No solution"),
    }
}

fn expand(map: &Map, state: &State) -> Vec<State> {
    //expand_move(map, state)
    expand_push(map, state)
}

fn heuristic(map: &Map, state: &State) -> i32 {
    //heuristic_move(map, state)
    heuristic_push(map, state)
}

fn mark_dead_ends(map: &mut Map) {
    // init first since otherwise we would use this partially initialized in search()
    for r in 0..map.map.len() {
        map.dead_ends.push(Vec::new());
        for _ in &map.map[r] {
            map.dead_ends[r].push(false);
        }
    }

    for r in 0..map.map.len() {
        'cell: for c in 0..map.map[r].len() {
            let box_pos = Pos { r: r as i32, c: c as i32 };
            if let &Cell::Wall = map.at(box_pos) {
                //print!("w");
                continue;
            }

            for dir in DIRECTIONS.iter() {
                let player_pos = box_pos + *dir;
                if let &Cell::Wall = map.at(player_pos) { continue; }

                let fake_state = State {
                    player_pos: player_pos,
                    boxes: vec![box_pos],
                };
                if let Some(_) = search(map, &fake_state, false) {
                    //print!("cont");
                    continue 'cell; // need to find only one solution
                }
            }
            //print!("true");
            map.dead_ends[r][c] = true; // no solution from any direction
            //print!("|");
        }
        //println!();
    }
    //println!();

    for r in 0..map.map.len() {
        for c in 0..map.map[r].len() {
            if map.dead_ends[r][c] {
                print!("{}", 1);
            } else {
                print!("{}", 0);
            }
        }
        println!();
    }
}

fn heuristic_push(map: &Map, state: &State) -> i32 {
    let mut goal_dist_sum = 0;
    for box_pos in &state.boxes {
        let mut min = i32::max_value();
        for goal in &map.goals {
            let dist = box_pos.dist(*goal);
            if dist < min {
                min = dist;
            }
        }
        goal_dist_sum += min;
    }
    goal_dist_sum
}

fn heuristic_move(map: &Map, state: &State) -> i32 {
    // less is better

    let mut closest_box = i32::max_value();
    for box_pos in &state.boxes {
        let dist = state.player_pos.dist(*box_pos);
        if dist < closest_box {
            closest_box = dist;
        }
    }

    let mut goal_dist_sum = 0;
    for box_pos in &state.boxes {
        let mut min = i32::max_value();
        for goal in &map.goals {
            let dist = box_pos.dist(*goal);
            if dist < min {
                min = dist;
            }
        }
        goal_dist_sum += min;
    }

    closest_box + goal_dist_sum
}

fn search(map: &Map, initial_state: &State, print_status: bool) -> Option<Vec<State>> {
    let mut expands = 0;
    let mut state_counts = Vec::new();
    state_counts.push(0);

    let mut to_visit = BinaryHeap::new();
    let mut closed = HashSet::new();
    let mut prev = HashMap::new();

    let h = heuristic(&map, &initial_state);
    let start = SearchState {
        state: initial_state.clone(),
        prev: None,
        dist: 0,
        h: h,
    };
    to_visit.push(start);
    while let Some(current) = to_visit.pop() {
        //println!("Trying (dist {}):\n{}", current.dist, map.clone().with_state(&current.state).to_string());

        if closed.contains(&current.state) { continue; }

        if current.dist > (state_counts.len() - 1) as i32 {
            state_counts.push(0);
            if print_status {
                println!("Depth: {}", current.dist);
                /*if current.dist == 50 {
                    for ss in &to_visit {
                        println!("{}", map.clone().with_state(&ss.state).to_string());
                    }
                }*/
            }
        }

        state_counts[current.dist as usize] += 1;

        // insert here and not as soon as we discover it
        // otherwise we overwrite the shortest path with longer ones
        if let Some(p) = current.prev {
            prev.insert(current.state.clone(), p.clone());
        }

        if solved(map, &current.state) {
            if print_status {
                println!("Expands: {}", expands);
                println!("Visited states in depth:");
                for i in 0..state_counts.len() {
                    println!("{}: {}", i, state_counts[i]);
                }
            }

            return Some(backtrack_path(&prev, &current.state))
        }

        expands += 1;
        for neighbor_state in expand(&map, &current.state) {
            // TODO this could probably be optimized a bit by allocating on the heap
            // and storing references only (to current state, neighbor state is always different)
            //prev.insert(neighbor_state.clone(), current.state.clone());

            // rust's binary heap doesn't support update_key() so we always insert and then ignore duplicates
            let h = heuristic(&map, &neighbor_state);
            let next = SearchState {
                state: neighbor_state,
                prev: Some(current.state.clone()),
                dist: current.dist + 1,
                h: h,
            };
            to_visit.push(next);
        }

        closed.insert(current.state);
    }

    if print_status {
        println!("Expands: {}", expands);
    }

    None
}

fn backtrack_path(prev: &HashMap<State, State>, final_state: &State) -> Vec<State> {
    let mut ret = Vec::new();
    let mut state = final_state;
    loop {
        ret.push(state.clone());
        if let Some(prev) = prev.get(state) {
            state = prev;
        } else {
            ret.reverse();
            return ret;
        }
    }
}

fn solved(map: &Map, state: &State) -> bool {
    // to detect dead ends, this has to test all boxes are on a goal, not that all goals have a box
    for pos in &state.boxes {
        if let &Cell::Path(PathCell { goal: true, .. }) = map.at(*pos) {} else { return false }
    }
    true
}

fn expand_push(map: &Map, state: &State) -> Vec<State> {
    let mut new_states = Vec::new();

    let map_state = map.clone().with_boxes(&state);

    let mut reachable = Vec::new();
    for r in 0..map.map.len() {
        reachable.push(Vec::new());
        for _ in 0..map.map[r].len() {
            reachable[r].push(false)
        }
    }

    mark_reachable(&map_state, &mut reachable, state.player_pos, state, &mut new_states);

    new_states
}

fn mark_reachable(map_state: &Map, reachable: &mut Vec<Vec<bool>>,
                  pos: Pos, state: &State, new_states: &mut Vec<State>) {
    let r = pos.r as usize;
    let c = pos.c as usize;
    reachable[r][c] = true;
    for dir in DIRECTIONS.iter() {
        let new_pos = pos + *dir;
        if let Cell::Path(PathCell { content: Content::Empty, .. }) = *map_state.at(new_pos) {
            if !reachable[new_pos.r as usize][new_pos.c as usize] {
                mark_reachable(map_state, reachable, new_pos, state, new_states);
            }
        } else if let Cell::Path(PathCell { content: Content::Box, .. }) = *map_state.at(new_pos) {
            let behind_box = new_pos + *dir;
            if let Cell::Path(PathCell { content: Content::Empty, .. }) = *map_state.at(behind_box) {
                if !map_state.dead_ends[behind_box.r as usize][behind_box.c as usize] {
                    let mut new_boxes = state.boxes.clone();
                    for box_pos in &mut new_boxes {
                        if *box_pos == new_pos {
                            *box_pos = behind_box;
                        }
                    }
                    let new_state = State {
                        player_pos: new_pos,
                        boxes: new_boxes,
                    };
                    new_states.push(new_state);
                }
            }
        }
    }
}

fn expand_move(map: &Map, state: &State) -> Vec<State> {
    let mut new_states = Vec::new();

    let map_state = map.clone().with_boxes(&state);
    for dir in DIRECTIONS.iter() {
        let new_pos = state.player_pos + *dir;
        if let Cell::Path(PathCell { content: Content::Empty, .. }) = *map_state.at(new_pos) {
            let new_state = State {
                player_pos: new_pos,
                boxes: state.boxes.clone(),
            };
            new_states.push(new_state);
        } else if let Cell::Path(PathCell { content: Content::Box, .. }) = *map_state.at(new_pos) {
            let behind_box = new_pos + *dir;
            if let Cell::Path(PathCell { content: Content::Empty, .. }) = *map_state.at(behind_box) {
                if !map.dead_ends[behind_box.r as usize][behind_box.c as usize] {
                    // goal will never be a dead end - no need to check
                    let mut new_boxes = state.boxes.clone();
                    for box_pos in &mut new_boxes {
                        if *box_pos == new_pos {
                            *box_pos = behind_box;
                        }
                    }
                    let new_state = State {
                        player_pos: new_pos,
                        boxes: new_boxes,
                    };
                    new_states.push(new_state);
                }
            }
        }
    }

    new_states
}

fn read_puzzle(path: &str) -> Result<String, Box<Error>> {
    let mut file = File::open(path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn parse(puzzle: &str) -> Result<(Map, State), ParseError> {
    let mut map = Vec::new();
    let mut player_pos = None;
    let mut boxes = Vec::new();
    let mut goals = Vec::new();
    for (r, line) in puzzle.lines().enumerate() {
        map.push(Vec::new());
        let mut chars = line.chars();
        while let (Some(c1), Some(c2)) = (chars.next(), chars.next()) {
            let c = map[r].len();
            match parse_cell(c1, c2) {
                Ok(Cell::Path(PathCell { content, goal })) => {
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
                    if goal {
                        goals.push(Pos { r: r as i32, c: c as i32 });
                    }
                    map[r].push(Cell::Path(PathCell {
                        content: Content::Empty,
                        goal: goal,
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

    if boxes.len() != goals.len() {
        return Err(ParseError::BoxesGoals);
    }

    Ok((Map { map: map, goals: goals, dead_ends: Vec::new() },
        State {
            player_pos: player_pos.unwrap(),
            boxes: boxes,
        }))
}

fn parse_cell(c1: char, c2: char) -> Result<Cell, ()> {
    match c1 {
        '<' => if c2 == '>' { Ok(Cell::Wall) } else { Err(()) },
        ' ' => {
            Ok(Cell::Path(PathCell {
                content: Content::Empty,
                goal: parse_cell_goal(c2)?,
            }))
        }
        'B' => {
            Ok(Cell::Path(PathCell {
                content: Content::Box,
                goal: parse_cell_goal(c2)?,
            }))
        }
        'P' => {
            Ok(Cell::Path(PathCell {
                content: Content::Player,
                goal: parse_cell_goal(c2)?,
            }))
        }
        _ => Err(()),
    }
}

fn parse_cell_goal(c: char) -> Result<bool, ()> {
    match c {
        '_' => Ok(true),
        ' ' => Ok(false),
        _ => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing() {
        let puzzle = "\
<><><><><>
<> _B_<><>
<>  B <><>
<>  P_<><>
<><><><><>
";
        let (map, state) = parse(puzzle).unwrap();
        assert!(map.with_state(&state).to_string() == puzzle);
    }
}
