use std::f32::INFINITY;
use std::cmp::{Ord, Ordering};
use std::{fmt, fs};
use std::str::FromStr;
use std::num::ParseIntError;
use std::ops::Index;
use std::collections::{BinaryHeap, HashSet, HashMap};
use std::iter::FromIterator;

use args::ArgsError;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Coord {
    x: usize,
    y: usize
}

impl Coord {
    pub fn is_inside(&self, level: &Level) -> bool {
        self.x < level.width && self.y < level.height
    }
}

impl From<(usize, usize)> for Coord {
    fn from(pair: (usize, usize)) -> Coord {
        Coord {x: pair.0, y: pair.1}
    }
}

pub enum CoordError {
    TooFew,
    TooMany,
    ParseIntError
}

impl From<ParseIntError> for CoordError {
    fn from(_: ParseIntError) -> Self { CoordError::ParseIntError }
}

impl FromStr for Coord {
    type Err = CoordError;
    
    fn from_str(s: &str) -> Result<Coord, Self::Err> {
        let chunks = s.split(":").collect::<Vec<&str>>();
        match chunks.len() {
            len if len <= 1 =>
                Err(CoordError::TooFew),
            len if len >= 3 =>
                Err(CoordError::TooMany),
            _ =>
                Ok(Coord {x: chunks[0].parse()?,
                          y: chunks[1].parse()?})
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Land {
    Pass,
    Block
}

impl Land {
    pub fn marker(&self) -> char {
        match self {
            Land::Block => '#',
            Land::Pass => '.'
        }
    }
}

pub struct Level {
    grid: Vec<Vec<Land>>,
    height: usize,
    width: usize
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\n")?;
        for row in &self.grid {
            for l in row {
                write!(f, "{}", l.marker())?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}


// only use this on checked coords which are inside
impl<'a> Index<&'a Coord> for Level {
    type Output = Land;

    fn index(&self, coord: &Coord) -> &Self::Output {
        &self.grid[coord.y][coord.x]
    }
}


impl Level {

    pub fn from_file(filename: &str) -> Result<Self, ArgsError> {
        let level_txt = fs::read_to_string(filename)?;
        let lines = level_txt.lines().map(String::from).collect::<Vec<_>>();
        let height = lines.len();
            
        if height == 0 {
            return Err(ArgsError::InvalidLevel)
        }
        
        let width = lines[0].len();
        let mut rows = Vec::with_capacity(lines.len());
        
        for line in lines {
            if line.len() != width {
                return Err(ArgsError::InvalidLevel)
            }
            let mut row = Vec::with_capacity(width);
            for c in line.chars() {
                match c {
                    '.' => row.push(Land::Pass),
                    '#' => row.push(Land::Block),
                    _ => return Err(ArgsError::InvalidLevel)
                }
            }
            rows.push(row);
        }
        Ok(Level {grid: rows, height: height, width: width})
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn max_x(&self) -> usize {
        self.width - 1
    }

    pub fn max_y(&self) -> usize {
        self.height - 1
    }

    fn neighbours(&self, pos: &Coord) -> Vec<Coord> {
        let min_x = if pos.x == 0 { 0 } else { pos.x - 1 };
        let min_y = if pos.y == 0 { 0 } else { pos.y - 1 };
        let max_x = if pos.x == self.max_x() { pos.x } else { pos.x + 1 };
        let max_y = if pos.y == self.max_y() { pos.y } else { pos.y + 1 };
        let mut coords = Vec::with_capacity(8);
        for x in min_x ..= max_x {
            for y in min_y ..= max_y {
                let coord = Coord {x, y};
                if coord != *pos && self[&coord] == Land::Pass {
                    coords.push(coord);
                }
            }
        }
        coords
    }
    
}

fn distance(from: &Coord, to: &Coord) -> f32 {
    let xs = (from.x as f32 - to.x as f32).powf(2.0);
    let ys = (from.y as f32 - to.y as f32).powf(2.0);
    (xs + ys).sqrt()
}


pub struct Path<'a> {
    level: &'a Level,
    coords: Vec<(usize, usize)>,
    distance: f32
}

impl<'a> fmt::Display for Path<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut groups = HashMap::<usize, Vec<usize>>::new();
        for (x, y) in self.coords.iter() {
            groups.entry(*y).and_modify(|xs| xs.push(*x)).or_insert_with(|| vec![*x]);
        }
        write!(f, "Path of {:?} coords travels distance of {} units.\n",
               self.coords.len(), self.distance)?;
        for (y, row) in self.level.grid.iter().enumerate() {
            let empty = &vec![];
            let xs = HashSet::<&usize>::from_iter(groups.get(&y).unwrap_or(empty));
            for (x, l) in row.iter().enumerate() {
                write!(f, "{}", if xs.contains(&x) { 'o' } else { l.marker() })?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}


#[derive(Debug, PartialEq)]
struct Candidate {
    cost: f32,     // heuristics cost (distance fn)
    coord: Coord
}

// since BinaryHeap is max-heap, we need results reversed (Less -> Greater and vice versa)
impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.cost.is_finite() && other.cost.is_finite() {
            if self.cost < other.cost { return Some(Ordering::Greater) } //Some(Ordering::Less) }
            if self.cost > other.cost { return Some(Ordering::Less) } //Some(Ordering::Greater) }
            Some(Ordering::Equal)
        } else {
            None
        }
    }
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap() // f32 come from distance fn which doesn't do division
    }
}

impl Eq for Candidate {}


const DIAGONAL_COST: f32 = 1.414;
const REGULAR_COST: f32 = 1.0;    


pub fn find(level: &Level, start: Coord, end: Coord) -> Option<Path> {
    if level[&start] == Land::Block || level[&end] == Land::Block {
        return None
    }

    let init_distance = distance(&start, &end);
    let init_candidate = Candidate {cost: init_distance, coord: start};
    let mut candidates = BinaryHeap::<Candidate>::from_iter(vec![init_candidate]);
    let mut origin = HashMap::<Coord, Coord>::new();
    let mut seen = HashSet::<Coord>::new();
    let mut open = HashSet::<Coord>::from_iter(vec![start]);
    let mut prefix_cost = HashMap::<Coord, f32>::from_iter(vec![(start, 0.0)]);
    let mut whole_cost = HashMap::<Coord, f32>::from_iter(vec![(start, init_distance)]); 

    loop {
        match candidates.pop() {
            None =>
                return None,
            Some(Candidate {coord: ref current, cost: distance}) if *current == end => {
                let mut coords = vec![(current.x, current.y)];
                let mut cursor = current;
                while let Some(source_coord) = origin.get(cursor) {
                    coords.push((source_coord.x, source_coord.y));
                    cursor = source_coord;
                }
                return Some(Path {level: level, coords: coords, distance: distance})
            }
            Some(Candidate {coord: current, ..}) => {

                open.remove(&current);
                seen.insert(current);

                let neighbours = level.neighbours(&current)
                    .into_iter()
                    .filter(|c| !seen.contains(c))
                    .collect::<Vec<_>>();
                
                let current_prefix_cost = prefix_cost.get(&current).unwrap().clone();
                
                for neighbor in neighbours {
                    let is_diagonal = neighbor.x != current.x && neighbor.y != current.y;
                    let transition_cost = if is_diagonal { DIAGONAL_COST } else { REGULAR_COST };
                    let neighbor_prefix_cost = current_prefix_cost + transition_cost;
                    let neighbor_postfix_cost = distance(&neighbor, &end);
                    let neighbor_whole_cost = neighbor_prefix_cost + neighbor_postfix_cost;
                    
                    if !open.contains(&neighbor) {
                        candidates.push(Candidate {cost: neighbor_whole_cost, coord: neighbor});
                        open.insert(neighbor);
                    } else {
                        let prev_npc = prefix_cost.get(&neighbor).unwrap_or(&INFINITY).clone();
                        if !(neighbor_prefix_cost < prev_npc) {
                            continue
                        }
                    }
                    origin.insert(neighbor, current);
                    prefix_cost.insert(neighbor, neighbor_prefix_cost);
                    whole_cost.insert(neighbor, neighbor_whole_cost);
                }
            }
        }
    }
}


