mod game;
mod game_move;
mod game_elements;
use std::collections::HashMap;

use colored::Colorize;

use game::*;
use crate::game::{Player, Move};

// use mcts::*;
// use mcts::tree_policy::*;
// use mcts::transposition_table::*;


fn play_game(game: &mut Game) {
  let mut line = String::new();
  loop {
    println!("{}", game);
    let prompt_text = "Please enter your next move:";
    let prompt = match game.turn {Player::Blue => prompt_text.blue(), Player::Red => prompt_text.red()};
    println!("{}", prompt);

    let response = match std::io::stdin().read_line(&mut line) {
      Ok(_response_size) => {
        let inputs = line.trim().split(" ").collect::<Vec<&str>>();
        let hand = &game.hands[&game.turn];
        match (valid_hand_idx(&hand, inputs[0]), inputs[1].parse::<Coord>()) {
          (Ok(card_idx), Ok(coords)) => Ok((card_idx, coords)),
          (Ok(_), Err(_)) => Err("Encountered parse int error when reading coordinates".to_string()),
          (Err(e), Ok(_)) => Err(e),
          (Err(e1), Err(e2)) => Err(format!("{}\n{}", e1, e2)),
        }
      }
      _ => Err("Encountered unknown error when parsing input".to_string())
    };

    match response {
      Ok((card_idx, coords)) => {
        let card = game.hands[&game.turn].0[card_idx].0.clone().unwrap();
        match game.make_move(Move::new(card, coords, game.turn)) {
          Some(moveresult) => (),
          None => println!("Invalid Move! Try again")
        }
      }
      _ => ()
    }

    line = "".to_string();
  }
}

fn main() {
  let cards = HashMap::from(
    [ ("Hildi", Card { name: "Hildibrand & Nashu Mhakaracca".to_string(), top:1, right:8, bottom:10, left:8, tribe:None, player:None})
    , ("Roundrox", Card { name: "Roundrox".to_string(), top: 2, right:2, bottom:8, left: 8, tribe:Some(Tribe::Beastman), player:None})
    , ("Estinien", Card { name: "Estinien".to_string(), top: 8, right:8, bottom:2, left: 3, tribe:None, player:None})
    , ("Alphinaud and Alisae", Card { name: "Alphinaud and Alisae".to_string(), top: 9, right:3, bottom:3, left: 9, tribe:None, player:None})
    , ("Ysayle", Card { name: "Ysayle".to_string(), top: 4, right:8, bottom:8, left: 1, tribe:None, player:None})
    , ("Therion", Card { name: "Thereon".to_string(), top: 9, right:9, bottom:2, left: 9, tribe:None, player:None})
    ]
  );

  let empty_hand : [Square; 5] = [Square(None), Square(None), Square(None), Square(None), Square(None)];
  let mut board = Game
    { turn: Player::Blue
    , hands: HashMap::from(
      [ (Player::Red, Hand(empty_hand.clone())),
        (Player::Blue, Hand(empty_hand.clone()))
      ]
    )
    , board: Board([[Square(None), Square(None), Square(None)], [Square(None), Square(None), Square(None)], [Square(None), Square(None), Square(None)]])
    , first_player: Player::Blue
    , score_blue: Score(0)
    , rules: vec![Rule::Plus, Rule::AllOpen]
    };

  let board = board.add_card_to_hand(cards["Hildi"].clone(), Player::Red);
  let board = board.add_card_to_hand(cards["Roundrox"].clone(), Player::Red);
  let board = board.add_card_to_hand(cards["Estinien"].clone(), Player::Red);
  let board = board.add_card_to_hand(cards["Alphinaud and Alisae"].clone(), Player::Red);
  let board = board.add_card_to_hand(cards["Ysayle"].clone(), Player::Red);

  let board = board.add_card_to_hand(cards["Hildi"].clone(), Player::Blue);
  let board = board.add_card_to_hand(cards["Roundrox"].clone(), Player::Blue);
  let board = board.add_card_to_hand(cards["Estinien"].clone(), Player::Blue);
  let board = board.add_card_to_hand(cards["Alphinaud and Alisae"].clone(), Player::Blue);
  let board = board.add_card_to_hand(cards["Ysayle"].clone(), Player::Blue);
  
  play_game(board)

}

