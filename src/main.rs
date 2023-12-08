mod game;
mod types;
mod api;
mod card_classification;

use std::collections::HashMap;
use card_classification::explore_cardlist;
use colored::Colorize;
use game::*;

use types::*;
use mcts::{MCTSManager, tree_policy::UCTPolicy, transposition_table::ApproxTable};
use crate::{game::{Player, Move}};


fn prompt_for_move(game:&Game) -> Move {
  let mut line = String::new();
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
      let entered_move = Move::new(card, coords, game.turn);
      let mut test_game = game.clone();
      match test_game.make_move(&entered_move) {
        Some(_moveresult) => entered_move,
        None => {
          println!("Invalid Move! Try again");
          prompt_for_move(game)
        }

      }
    },
    _ => prompt_for_move(game)
  }
}

fn prompt_for_your_color() -> Player {
  let mut line = String::new();
  println!("What color are you?");

  match std::io::stdin().read_line(&mut line) {
    Ok(_response_size) => {
      let inputs = line.trim();
      match inputs {
        line if line.to_lowercase() == "r" || line.to_lowercase() == "red" => Player::Red,
        line if line.to_lowercase() == "b" || line.to_lowercase() == "blue" => Player::Blue,
        _ => {
          println!("Input didn't match [{},{},{},{}], try again", "red", "r", "blue", "b");
          prompt_for_first_player()
        }
      }
    }
    _ => {
      println!("Encountered unknown error when parsing input");
      prompt_for_first_player()
    }
  }
}

fn prompt_for_first_player() -> Player {
  let mut line = String::new();
  println!("Who is the first player?");

  match std::io::stdin().read_line(&mut line) {
    Ok(_response_size) => {
      let inputs = line.trim();
      match inputs {
        line if line.to_lowercase() == "r" || line.to_lowercase() == "red" => Player::Red,
        line if line.to_lowercase() == "b" || line.to_lowercase() == "blue" => Player::Blue,
        _ => {
          println!("Input didn't match [{},{},{},{}], try again", "red", "r", "blue", "b");
          prompt_for_first_player()
        }
      }
    }
    _ => {
      println!("Encountered unknown error when parsing input");
      prompt_for_first_player()
    }
  }
}

fn prompt_for_card() -> GameCard {
  let mut line = String::new();
  println!("Enter the first opponent's card, \"top right bottom left\" ");

  match std::io::stdin().read_line(&mut line) {
    Ok(_response_size) => {
      let inputs = line.trim().split(" ").collect::<Vec<&str>>();
      let valid_inputs = inputs.iter().map(|x| x.parse::<usize>()).filter_map((|x| match x {Ok(x) if (1..=10).contains(&x) => Some(x), _ => None})).collect::<Vec<usize>>();
      if valid_inputs.len() == 4 {
        let top = valid_inputs[0];
        let right = valid_inputs[1];
        let bottom = valid_inputs[2];
        let left = valid_inputs[3];
        let card = GameCard{ card:Card{name:"Doesn't_Matter".to_string(), id:0, stars:3, stats: CardStats { top:top, right:right, bottom:bottom, left:left, tribe:None}}, player:Some(Player::Red)};
        card
      }
      else {
        println!("Encountered error when parsing numbers. Use format \"# # # #\" ");
        prompt_for_card()
      }
    }
    _ => {println!("Encountered unknown error when parsing input"); prompt_for_card()}
  }
}


fn build_my_hand(game:&mut Game, player:Player) -> &mut Game{
  let cards = HashMap::from(
    [ ("Hildi", GameCard{ card: Card { name: "Hildibrand & Nashu Mhakaracca".to_string(), id: 0, stars: 5, stats: CardStats {  top:1, right:8, bottom:10, left:8, tribe:None}}, player:None})
    , ("Roundrox", GameCard{ card: Card { name: "Roundrox".to_string(), id: 0, stars: 3, stats: CardStats { top: 2, right:2, bottom:8, left: 8, tribe:Some(Tribe::Beastman) }}, player:None})
    , ("Estinien", GameCard{ card: Card { name: "Estinien".to_string(), id: 0, stars: 3, stats: CardStats { top: 8, right:8, bottom:2, left: 3, tribe:None}}, player:None})
    , ("Alphinaud and Alisae", GameCard{ card: Card { name: "Alphinaud and Alisae".to_string(), id: 0, stars: 4, stats: CardStats { top: 9, right:3, bottom:3, left: 9, tribe:None}}, player:None})
    , ("Ysayle", GameCard{ card: Card { name: "Ysayle".to_string(), id: 0, stars: 3, stats: CardStats { top: 4, right:8, bottom:8, left: 1, tribe:None}}, player:None})
    , ("Therion", GameCard{ card: Card { name: "Thereon".to_string(), id: 0, stars: 5, stats: CardStats { top: 9, right:9, bottom:2, left: 9, tribe:None}}, player:None})
    ]
  );
  let game = game.add_card_to_hand(cards["Hildi"].clone(), player);
  let game = game.add_card_to_hand(cards["Roundrox"].clone(), player);
  let game = game.add_card_to_hand(cards["Estinien"].clone(), player);
  let game = game.add_card_to_hand(cards["Alphinaud and Alisae"].clone(), player);
  let game = game.add_card_to_hand(cards["Ysayle"].clone(), player);
  game
}


fn initialize_game() -> Game {
  let player_color = prompt_for_your_color();
  let first_player = prompt_for_first_player();
  let empty_hand : [Square; 5] = [Square(None), Square(None), Square(None), Square(None), Square(None)];
  let mut game = Game
    { turn: first_player
    , hands: HashMap::from(
      [ (Player::Red, Hand(empty_hand.clone())),
        (Player::Blue, Hand(empty_hand.clone()))
      ]
    )
    , board: Board([[Square(None), Square(None), Square(None)], [Square(None), Square(None), Square(None)], [Square(None), Square(None), Square(None)]])
    , first_player: first_player
    , my_color: player_color
    , rules: vec![Rule::Plus, Rule::AllOpen]
    };
  let mut game = build_my_hand(&mut game, player_color);
  for i in (0..=4) {
    let card = prompt_for_card();
    game = game.add_card_to_hand(card, match player_color {Player::Red => Player::Blue, Player::Blue => Player::Red});
  }
  game.clone()
}



fn play_game(game: &mut Game) {
  loop {

    let copied_game = game.clone();
    

    println!("{}", game);

    if game.turn == game.my_color {
      println!("Evaluating Moves...");
      let mut mcts = MCTSManager::new(copied_game, MyMCTS, MyEvaluator, UCTPolicy::new(0.5), ApproxTable::new(1024));
      mcts.playout_n_parallel(250_000, 4);
      mcts.tree().debug_moves();
    }
    let next_move = prompt_for_move(game);
    let result = game.make_move(&next_move);
    if result.is_none() {
      println!("Game Over!");
      break
    }
  }
}



fn main() -> (){

  // let game = game.add_card_to_hand(cards["Hildi"].clone(), Player::Red);
  // let game = game.add_card_to_hand(cards["Roundrox"].clone(), Player::Red);
  // let game = game.add_card_to_hand(cards["Estinien"].clone(), Player::Red);
  // let game = game.add_card_to_hand(cards["Alphinaud and Alisae"].clone(), Player::Red);
  // let game = game.add_card_to_hand(cards["Ysayle"].clone(), Player::Red);

  // let game = game.add_card_to_hand(cards["Hildi"].clone(), Player::Blue);
  // let game = game.add_card_to_hand(cards["Roundrox"].clone(), Player::Blue);
  // let game = game.add_card_to_hand(cards["Estinien"].clone(), Player::Blue);
  // let game = game.add_card_to_hand(cards["Alphinaud and Alisae"].clone(), Player::Blue);
  // let game = game.add_card_to_hand(cards["Ysayle"].clone(), Player::Blue);

  //let game = game.clone();
  
  // println!("{:?}", mcts.best_move().unwrap())

  explore_cardlist()

}

