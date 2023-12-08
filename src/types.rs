use std::{str::FromStr, fmt};

use colored::Colorize;


#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Row
{ Top = 0
, Middle = 1
, Bottom = 2
} impl Row {
  pub fn idx(self) -> usize { self as usize }
  pub fn from_idx(idx:usize) -> Option<Row> {
    match idx {
      0 => Some(Row::Top),
      1 => Some(Row::Middle),
      2 => Some(Row::Bottom),
      _ => None
    }
  }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Column
{ Left = 0
, Middle = 1
, Right = 2
} impl Column {
  pub fn idx(self) -> usize { self as usize }
  pub fn from_idx(idx:usize) -> Option<Column> {
    match idx {
      0 => Some(Column::Left),
      1 => Some(Column::Middle),
      2 => Some(Column::Right),
      _ => None
    }
  }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Coord
{ pub row: Row
, pub col: Column
} impl FromStr for Coord {
  type Err = std::num::ParseIntError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let coords: Vec<&str> = s.trim()
                              .split(',')
                              .collect();
    let row = Row::from_idx(coords[0].parse::<usize>().unwrap() - 1).unwrap();
    let col = Column::from_idx(coords[1].parse::<usize>().unwrap() - 1).unwrap();
    Ok(Coord {row, col})
  }
} impl Coord {
  pub fn from_numbers(row: usize, col: usize) -> Option<Self>{
    match (Row::from_idx(row), Column::from_idx(col)) {
      (Some(row), Some(col)) => Some(Coord {row,col}),
      _ => None
    }
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Tribe {
  Beastman,
  Scion,
  Garlean,
  Primal,
} impl fmt::Display for Tribe {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let out = match self {
      Tribe::Beastman => "*".green().to_string(),
      Tribe::Garlean => "*".blue().to_string(),
      Tribe::Primal => "*".red().to_string(),
      Tribe::Scion => "*".yellow().to_string()
    };
    write!(f, "{}", out)
  }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct CardStats 
{ pub top: usize
, pub right: usize
, pub bottom: usize
, pub left: usize
, pub tribe: Option<Tribe>
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Card 
{ pub id: usize
, pub name: String
, pub stars: usize
, pub stats: CardStats
} 

