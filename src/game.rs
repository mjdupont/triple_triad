#[path="game_move.rs"] mod game_move;
#[path="game_elements.rs"] mod game_elements;
use std::{hash::{Hash, Hasher}, collections::hash_map::DefaultHasher};
#[allow(unstable_name_collisions)]

use std::{str::FromStr, fmt, ops::RangeInclusive, collections::HashMap};
use colored::Colorize;
use itertools::*;

use mcts::{*, tree_policy::UCTPolicy, transposition_table::{ApproxTable, TranspositionHash}};



#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Row
{ Top = 0
, Middle = 1
, Bottom = 2
} impl Row {
  fn idx(self) -> usize { self as usize }
  fn from_idx(idx:usize) -> Option<Row> {
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
  fn idx(self) -> usize { self as usize }
  fn from_idx(idx:usize) -> Option<Column> {
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
    println!("{:?}", coords);
    let row = Row::from_idx(coords[0].parse::<usize>().unwrap() - 1).unwrap();
    let col = Column::from_idx(coords[1].parse::<usize>().unwrap() - 1).unwrap();
    Ok(Coord {row, col})
  }
} impl Coord {
  fn from_numbers(row: usize, col: usize) -> Option<Self>{
    match (Row::from_idx(row), Column::from_idx(col)) {
      (Some(row), Some(col)) => Some(Coord {row,col}),
      _ => None
    }
  }
}

pub fn valid_hand_idx(hand: &Hand, s:&str) -> Result<usize, String>{
  match s.parse::<usize>() {
    Err(_e) => Err("Failed to parse input as an int!".to_string()),
    Ok(idx) if !(0..5).contains(&idx) => Err("Invalid card index! Please enter a number between 0 and 4".to_string()),
    Ok(idx) if (0..5).contains(&idx) && hand.0[idx].0.is_none() => Err("You already played the card in that position! Select a different position".to_string()),
    Ok(idx) if (0..5).contains(&idx) && hand.0[idx].0.is_some() => Ok(idx),
    _ => Err("An undetermined parse error occurred. Please check your input and try again.".to_string())
  }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Move
{ card: Card
, coords: Coord
, player: Player
, is_combo: bool
} impl Move {
  pub fn new(card:Card, coords:Coord, player:Player) -> Move {
    Move { card:card, coords:coords, player:player, is_combo:false }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Player 
{ Red
, Blue
} 
impl fmt::Display for Player {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let out = match self {
      Player::Red => "R",
      Player::Blue => "B",
    };
    write!(f, "{}", out)
    
  }
}

fn format_number(number:usize) -> String {
  match number {
    1..=9 => number.to_string().bold().to_string(),
    10 => "A".bold().to_string(),
    _ => "#".bold().to_string()
  }
}


#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Card 
{ pub name: String
, pub top : usize
, pub right: usize
, pub bottom: usize
, pub left: usize
, pub tribe: Option<Tribe>
, pub player: Option<Player>
} 
impl fmt::Display for Card {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let tribe = self.tribe.map(|t| t.to_string()).unwrap_or(" ".to_string());
    
    let color_line = |x:&String| match self.player {
      None => x.white().to_string(),
      Some(Player::Red) => x.red().to_string(),
      Some(Player::Blue) => x.blue().to_string()
    };
    let card_top = format!("{}{}{}", "╔".to_string().yellow(), color_line(&"━━━━━".to_string()), "╗".to_string().yellow());
    let top     = format!("{}{}{}", color_line(&format!("┃  {} ",format_number(self.top))), tribe, color_line(&"┃".to_string()));
    let middle  = color_line(&format!("┃{} {} {}┃", format_number(self.left), " ", format_number(self.right)));
    let bottom  = color_line(&format!("┃{} {} {}┃", " ", format_number(self.bottom), " "));
    let card_bottom = format!("{}{}{}", "╚".to_string().yellow(), color_line(&"━━━━━".to_string()), "╝".to_string().yellow());
    let out = vec![card_top, top, middle, bottom, card_bottom];
    write!(f, "{}", out.join("\n"))
  }
} 
impl Card {
  pub fn flip(&self) -> Card {
    let mut out = self.clone();
    out.player = out.player.map(|x| match x { Player::Blue => Player::Red, Player::Red => Player::Blue});
    out
  }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Square(pub Option<Card>);
impl fmt::Display for Square {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let out = match self {
      Square(None) => String::from(vec!(
                                    "╔─────╗"
                                  , "┃     ┃"
                                  , "┃     ┃"
                                  , "┃     ┃"
                                  , "╚─────╝"
                                  ).join("\n")),
      Square(Some(card)) => format!("{}", card) 
    };
    write!(f, "{}", out)
  }
}
impl Square {
  pub fn new(card:Card) -> Self {
    Self(Some(card))
  }
  pub fn flip(self) -> Square {
    let Square(sq) = self;
    Square(sq.map(|card| card.clone().flip()))
  }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct Score(pub isize);
impl fmt::Display for Score {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let out = match self {
      Score(x) if (-4..=4).contains(x) => {
        let blue_score = "[]".to_string().repeat((5 + x).try_into().unwrap()).blue().to_string();
        let red_score = "[]".to_string().repeat((5 - x).try_into().unwrap()).red().to_string();
        format!("     {}║{}     ", blue_score, red_score)
      },
      _ => "   ##########ERROR##########   ".to_string()
    };
    write!(f, "{}", out)
  }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Hand(pub [Square; 5]);
impl fmt::Display for Hand {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let hand_top =    "┌─────────┬─────────┬─────────┐";
    let middle_sep =  "└────┬────┴────┬────┴────┬────┘";
    let hand_bottom = "     └─────────┴─────────┘     ";

    let fmt_top_row = || {
      let row = vec![&self.0[0], &self.0[1], &self.0[2]];
      let splitrow = 
        row
        .iter()
        .map(|x| 
          format!("{}", x)
          .split("\n")
          .map(|x| x.to_string())
          .collect())
        .collect::<Vec<Vec<String>>>();
  
      let n_lines = splitrow.first().unwrap().len();
      let mut row_iters : Vec<_> = splitrow.into_iter().map(Vec::into_iter).collect();
      let out_row = (0..n_lines)
        .map(|_| row_iters.iter_mut().map(|x| x.next().unwrap()).collect())
        .collect::<Vec<Vec<String>>>()
        .into_iter()
        .enumerate()
        .map(|(idx, lines)| {
          if idx != 0 {
            format!("│ {} │", lines.join(" │ ")).to_string()
          } else {
            let interspersed_seps = (1..lines.len()).map(|n| format!(" │{}", n).to_string()).collect::<Vec<String>>();
            let final_line = 
              interleave(lines, interspersed_seps.into_iter())
              .reduce(|string:String, next:String| format!("{}{}", string, next))
              .unwrap();
            format!("│{}{} │", 0, final_line).to_string()
          }})
        .collect::<Vec<String>>()
        .join("\n")
        ;
      out_row
    };

    let fmt_bottom_row = || {
      let row = vec![&self.0[3], &self.0[4]];
      let splitrow = 
        row
        .iter()
        .map(|x| 
          format!("{}", x)
          .split("\n")
          .map(|x| x.to_string())
          .collect())
        .collect::<Vec<Vec<String>>>();
  
      let n_lines = splitrow.first().unwrap().len();
      let mut row_iters : Vec<_> = splitrow.into_iter().map(Vec::into_iter).collect();
      let out_row = (0..n_lines)
        .map(|_| row_iters.iter_mut().map(|x| x.next().unwrap()).collect())
        .collect::<Vec<Vec<String>>>()
        .into_iter()
        
        .enumerate()
        .map(|(idx, lines)| {
          if idx != 0 {
            format!("     │ {} │     ", lines.join(" │ ")).to_string()
          } else {
            let interspersed_seps = vec![format!(" │{}", 4).to_string()];
            let final_line = 
              interleave(lines, interspersed_seps.into_iter())
              .reduce(|string:String, next:String| format!("{}{}", string, next))
              .unwrap();
            format!("     │{}{} │     ", 3, final_line).to_string()
          }})
        .collect::<Vec<String>>()
        .join("\n")
        ;
      out_row
    };
    
  let mut top_row: Vec<String> = vec![fmt_top_row()];
  let mut bottom_row = vec![fmt_bottom_row()];
  let rows = 
    &mut vec![hand_top.to_string()];
    rows.append(&mut top_row);
    rows.append(&mut vec![middle_sep.to_string()]);
    rows.append(&mut bottom_row);
    rows.append(&mut vec![hand_bottom.to_string()]);

    let out = rows.join("\n");
    write!(f, "{}", out)
  }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Board(pub [[Square;3];3]);
impl fmt::Display for Board {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let board_top =    "┌─────────┬─────────┬─────────┐";
    let row_sep =      "├─────────┼─────────┼─────────┤";
    let board_bottom = "└─────────┴─────────┴─────────┘";

    let Board(board) = &self;

    let fmt_row = |n:usize| {
      let row = vec![&board[n][0], &board[n][1], &board[n][2]];
      let splitrow = 
        row
        .iter()
        .map(|x| 
          format!("{}", x)
          .split("\n")
          .map(|x| x.to_string())
          .collect())
        .collect::<Vec<Vec<String>>>();
  
      let n_lines = splitrow.first().unwrap().len();
      let mut row_iters : Vec<_> = splitrow.into_iter().map(Vec::into_iter).collect();
      let out_row = (0..n_lines)
        .map(|_| row_iters.iter_mut().map(|x| x.next().unwrap()).collect())
        .collect::<Vec<Vec<String>>>()
        .iter()
        .map(|x| format!("│ {} │", x.join(" │ ")))
        .collect::<Vec<String>>()
        .join("\n")
        ;
      out_row
    };

    let mut rows = 
      (0..=2)
      .into_iter()
      .map(fmt_row)
      .intersperse(row_sep.to_string())
      .collect::<Vec<String>>();
   
    let full_board = &mut vec![board_top.to_string()];
    full_board.append(&mut rows);
    full_board.append(&mut vec![board_bottom.to_string()]);

    let out_rows = full_board.join("\n");

    write!(f, "{}", out_rows)
  }
}

pub enum GameResult
{ Draw
, Win(Player)
}

pub enum MoveResult
{ Finished(GameResult)
, Combo(Vec<Move>)
, NextMove
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Side 
{ Top
, Right
, Bottom
, Left
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Rule
{ AllOpen
//TODO , ThreeOpen
//TODO , Chaos
//TODO , Order
, Plus
//TODO, Same
//TODO, Reverse
//TODO, Ascension
//TODO, Descension
//TODO, FallenAce
}


#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Game
{ pub board: Board
, pub hands: HashMap<Player, Hand>
, pub turn: Player
, pub first_player : Player
, pub score_blue: Score
, pub rules: Vec<Rule>
} 
impl fmt::Display for Game {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut blue_hand_str = format!("{}",self.hands[&Player::Blue]).split("\n").map(|x| x.to_string()).collect::<Vec<String>>();
    let mut red_hand_str = format!("{}", self.hands[&Player::Red]).split("\n").map(|x| x.to_string()).collect::<Vec<String>>();
    let board_str = format!("{}", self.board.clone()).split("\n").map(|x| x.to_string()).collect::<Vec<String>>();
    let vertical_padding = board_str.len() - blue_hand_str.len();
    let upper_padding = vertical_padding / 2;
    let lower_padding = vertical_padding / 2 + vertical_padding % 2;
    let hand_padding = (0..31).map(|_| " ").collect::<String>();
    let mut blue_hand_for_print = (0..upper_padding).map(|_| format!("{}", hand_padding)).collect::<Vec<String>>();
    blue_hand_for_print.append(&mut blue_hand_str);
    blue_hand_for_print.append(&mut vec![hand_padding.clone(); lower_padding]);

    let mut red_hand_for_print = (0..upper_padding).map(|_| format!("{}", hand_padding)).collect::<Vec<String>>();
    red_hand_for_print.append(&mut red_hand_str);
    red_hand_for_print.append(&mut vec![hand_padding.clone(); lower_padding]);

    let out = 
      izip!(blue_hand_for_print, board_str, red_hand_for_print)
      .map(|(bhand, board, rhand)| format!("{}   {}   {}", bhand, board, rhand).to_string())
      .join("\n");

    write!(f, "\n{0}   {1}   {0}\n{2}\n", hand_padding, self.score_blue, out)
  }
}
impl Game {
  fn open_squares(&self) -> Vec<Coord> {
    let all_indices = (0..3).cartesian_product((0..3));
    all_indices
    .filter(|(row, col)| self.board.0[*row as usize][*col as usize].0.is_none())
    .map(|(row, col)| Coord{row:Row::from_idx(row).unwrap(), col:Column::from_idx(col).unwrap()}) 
    .collect::<Vec<Coord>>()
  }



  fn is_valid_move(&self, user_move:&Move) -> bool {
    let is_players_turn = user_move.player == self.turn;
    let is_available_card = self.hands[&self.turn].0.contains(&Square::new(user_move.card.clone()));
    let is_available_space = self.board.0[user_move.coords.row.idx()][user_move.coords.col.idx()].0.is_none(); 

    (is_players_turn && is_available_card && is_available_space) || user_move.is_combo
  }



  fn get_valid_moves(&self) -> Vec<Move> {
    let cards = self.hands[&self.turn].0.iter().filter_map(|x| x.0).collect::<Vec<Card>>();
    let spaces = self.open_squares();
    let cartesian_product = itertools::iproduct!(cards, spaces).map(|(card, coord)| Move::new(card, coord, self.turn)).collect::<Vec<Move>>();
    cartesian_product
  }



  fn add_card_to_board(&mut self, user_move:Move) -> () {
    self.board.0[user_move.coords.row.idx()][user_move.coords.col.idx()] = Square::new(user_move.card)
  }



  pub fn add_card_to_hand(&mut self, mut new_card:Card, player:Player) -> &mut Game {
    new_card.player = Some(player);
  

    let available_idx = 
    self.hands[&player].0
      .iter()
      .enumerate()
      .find_map(|(idx, ele)| if ele.0.is_none() {Some(idx)} else {None});
    
    match available_idx 
    {
      Option::Some(idx) => 
      self.hands.get_mut(&player).unwrap().0[idx] = Square::new(new_card),
      Option::None => ()
    }
    self
  }



  fn remove_card_from_hand(&mut self, user_move:&Move) {
    let card_matches = self.hands[&user_move.player].0.iter().map(|x| x == &Square::new(user_move.card.clone())).collect::<Vec<bool>>();
    let hand_card_idx = card_matches.iter().enumerate().filter_map(|(idx, b)| (*b).then(|| idx)).nth(0).unwrap();
    self.hands.get_mut(&user_move.player).unwrap().0[hand_card_idx] = Square(None);
  }



  fn play_move(&mut self, user_move:Move) -> () {
    println!("Playing {} card {} at {} {}!", user_move.card.player.unwrap(), user_move.card.name, user_move.coords.row.idx(), user_move.coords.col.idx());
    if !user_move.is_combo {
      self.remove_card_from_hand(&user_move);
    }
    self.add_card_to_board(user_move);
  }



  fn flip_turn(&mut self) -> () {
    match self.turn {
      Player::Red => self.turn = Player::Blue,
      Player::Blue => self.turn = Player::Red
    }
  }



  fn identify_captured_cards(comparisons:Vec<(Coord, Card, isize)>, capturing_player:&Player) -> Vec<(Card, Coord)> {
    comparisons
      .into_iter()
      .filter(|(_coords, card, diff)| (diff > &0) && !(card.player == Some(*capturing_player)))
      .map(|(coords, card, _diff)| (card.clone(), coords))
      .collect::<Vec<(Card, Coord)>>()
  }



  fn capture_cards(&mut self, comparisons:Vec<(Coord, Card, isize)>, capturing_player:&Player) -> () {
    let capturing_moves = Game::identify_captured_cards(comparisons, capturing_player);

    for (_card, coords) in capturing_moves {
      self.board.0[coords.row.idx()][coords.col.idx() as usize].0.as_mut().unwrap().player = Some(*capturing_player);
      match self.turn {
        Player::Red => self.score_blue.0 -= 1,
        Player::Blue => self.score_blue.0 += 1
      }
    }
  }



  fn calculate_card_diffs(user_move:&Move, comparisons:Vec<(Coord, Card, Side)>) -> Vec<(Coord, Card, isize)> {
    fn card_diff(board_card:&Card, new_card:&Card, side:&Side) -> isize{
      match side {
        Side::Top => new_card.bottom as isize - board_card.top as isize,
        Side::Right => new_card.left as isize - board_card.right as isize, 
        Side::Bottom => new_card.top as isize - board_card.bottom as isize, 
        Side::Left => new_card.right as isize - board_card.left as isize, 
      }
    }
    comparisons
    .into_iter()
    .map(|(coord, card, side)| {let diff = card_diff(&card, &user_move.card, &side); (coord, card, diff)})
    .collect::<Vec<(Coord, Card, isize)>>()
  }



  fn resolve_card_comparisons(&mut self, user_move:Move, comparisons:Vec<(Coord, Card, Side)>) -> Option<Vec<Move>>{
    
    fn card_sum(board_card:&Card, new_card:&Card, side:&Side) -> isize{
      match side {
        Side::Top => new_card.bottom as isize + board_card.top as isize,
        Side::Right => new_card.left as isize + board_card.right as isize, 
        Side::Bottom => new_card.top as isize + board_card.bottom as isize, 
        Side::Left => new_card.right as isize + board_card.left as isize, 
      }
    }
    
    println!("comparisons: {:?}", comparisons);
    match comparisons {
      comparisons if user_move.is_combo => {  
        let captured_cards = Game::identify_captured_cards(Game::calculate_card_diffs(&user_move, comparisons), &user_move.player);

        let player = user_move.player;

        // Play the selected move
        self.play_move(user_move);

        // Remove all cards captured by combo
        for (_, coords) in &captured_cards {
          self.board.0[coords.row.idx()][coords.col.idx()] = Square(None)
        }

        // Add captured cards to be evaluated and continue combo
        let combo_moves = 
          captured_cards
          .into_iter()
          .map(|(card, coords)| 
            { let mut new_card = card.clone(); 
              new_card.player = Some(player);
              Move {card: new_card, coords:coords, player:player, is_combo:true}}
            )
          .collect::<Vec<Move>>();
        Some(combo_moves)
        
      }
      //Don't check for Plus if in combo
      comparisons if self.rules.contains(&Rule::Plus) && !user_move.is_combo => {
        let (not_plus_comparisons, plus_comparisons) =
          comparisons
          .into_iter()
          // Group all comparisons by the magnitude of their sum; same sum = Plus
          .into_group_map_by(|(_, card, side)| card_sum(card, &user_move.card, side))
          .iter_mut()
          .fold( (Vec::new(), Vec::new()), 
                |(mut not_plus_comparisons, mut plus_comparisons), (_, comparisons)| {
                  // Categorize groups of comparisons; If a group is 2 or more elements, Plus has been achieved.
                  // If a singleton group, no neighbors had plus, and this neighbor should be resolved normally.
                  if comparisons.len() >= 2 { plus_comparisons.append(comparisons) }
                  else { not_plus_comparisons.append(comparisons) } 
                  (not_plus_comparisons, plus_comparisons)
                }
              );
        println!("non_plus_comparisons: {:?}, plus_comparisons: {:?}", not_plus_comparisons, plus_comparisons);

        // Resolve all non-plus moves as normal.
        self.capture_cards(Game::calculate_card_diffs(&user_move, not_plus_comparisons), &user_move.player);
        
        let moving_player = user_move.player.clone();

        // Play the current move
        self.play_move(user_move);

        // Check for, and handle, "Combo"
        match plus_comparisons {
          // Case where Plus was activated; proceed to combo
          continuations if continuations.len() > 0 => 
            { 
              // Remove all cards affected by Plus
              for (coords, _, _) in &continuations {
                self.board.0[coords.row.idx()][coords.col.idx()] = Square(None)
              }

              // Convert continuations to move
              let moves = 
                continuations
                .into_iter()
                .map(|(coords, mut card, _)| {
                  card.player = Some(moving_player);
                  Move{card:card, coords:coords, player:moving_player, is_combo:true}
                })
                .collect::<Vec<Move>>();

              // Return list of replay moves to play (all the cards in plus)
              Some(moves)
            }

          // No Plus was activated, and no combo needs to be evaluated.
          _ => None
        }
      }

      // No captures
      comparisons if comparisons.is_empty() => {
        self.play_move(user_move);
        None
      }

      // Captures
      comparisons => {
        
        self.capture_cards(Game::calculate_card_diffs(&user_move, comparisons), &user_move.player);
        self.play_move(user_move);
        None
      }
    }
  }



  fn compare_move_card_to_neighbors(&mut self, user_move:&Move) -> Vec<(Coord, Card, Side)> {

    let relative_neighbor_positions = vec![((1,0), Side::Top), ((0,1), Side::Left), ((-1,0), Side::Bottom), ((0,-1), Side::Right)];
    
    let valid_neighbors = 
      relative_neighbor_positions
      .into_iter()
      // Apply all relative neighbor adjustments to our intended move;
      // Coord constructor returns an Option if coords are valid; filter_map removes invalid coords.
      .filter_map(|((row_adj, col_adj), side)| 
        {
          let coords = Coord::from_numbers((user_move.coords.row.idx() as isize + row_adj) as usize, (user_move.coords.col.idx() as isize + col_adj) as usize);
          coords
          .map(|x| (x, side.clone()))
          // Filter out moves whose Squares have "None" for a card
          .and_then(|(coords, side)| self.board.0[coords.row.idx() as usize][coords.col.idx() as usize].0.clone().map(|board_card|  (coords, board_card, side))) 
        }
      )
      .collect::<Vec<(Coord, Card, Side)>>();

    valid_neighbors
  }



  pub fn make_move(&mut self, user_move:Move) -> Option<MoveResult>{
    if self.is_valid_move(&user_move) {
      let is_combo = user_move.is_combo;

      let card_comparisons = self.compare_move_card_to_neighbors(&user_move);
      let combos = self.resolve_card_comparisons(user_move, card_comparisons);

      match combos {
        Some(combos) =>
          for combo_move in combos {
            self.make_move(combo_move);
          }
        None => ()
      }

      if !is_combo { self.flip_turn()};

      // Return result
      if self.open_squares().len() > 0 { Some(MoveResult::NextMove) }
      else {
        match self.score_blue {
          score if score.0 == 0 => Some(MoveResult::Finished(GameResult::Draw)),
          score if score.0 > 0 => Some(MoveResult::Finished(GameResult::Win(Player::Blue))),
          score if score.0 < 0 => Some(MoveResult::Finished(GameResult::Win(Player::Red))),
          _ => Some(MoveResult::Finished(GameResult::Draw))
        }}
      
    }
    else {
      None
    }
  }



}
impl GameState for Game {
  type Move = Move;
  type Player = Player;
  type MoveList = Vec<Move>;
  fn current_player(&self) -> Self::Player {
    self.turn
  }
  fn available_moves(&self) -> Vec<Move> {
    self.get_valid_moves()
  }
  fn make_move(&mut self, mov:&Self::Move) {
    self.make_move(*mov);
  }

}

impl TranspositionHash for Game {
  fn hash(&self) -> u64 {
    let mut hasher = DefaultHasher::new();
    self.board.hash(&mut hasher);
    for hand in self.hands.values() {
      hand.hash(&mut hasher);
    }
    self.first_player.hash(&mut hasher);

    hasher.finish()
  }
}

struct MyEvaluator;
impl Evaluator<MyMCTS> for MyEvaluator {
  type StateEvaluation = f32;

  fn evaluate_new_state(&self, state: &Game, moves: &Vec<Move>, _: Option<SearchHandle<MyMCTS>>) -> (Vec<()>, f32) {
    (vec![(); moves.len()], state.score_blue.0 as f32)
  }

fn evaluate_existing_state(&self, state: &<MyMCTS as MCTS>::State, existing_evaln: &Self::StateEvaluation,
        handle: SearchHandle<MyMCTS>)
        -> Self::StateEvaluation {
        state.score_blue.0 as f32
    }

fn interpret_evaluation_for_player(&self,
        evaluation: &Self::StateEvaluation,
        player: &mcts::Player<MyMCTS>) -> i64 {
        *evaluation as i64
    }
}

#[derive(Default)]
struct MyMCTS;
impl MCTS for MyMCTS {
  type State = Game;
  type Eval = MyEvaluator;
  type NodeData = ();
  type ExtraThreadData = ();
  type TreePolicy = UCTPolicy;
  type TranspositionTable = ApproxTable<Self>;

  fn cycle_behaviour(&self) -> CycleBehaviour<Self> {
      CycleBehaviour::PanicWhenCycleDetected
  }
}