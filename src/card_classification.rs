
use itertools::Itertools;

use crate::{types::Card, api};

///These categories are meta-specific; 
/// - Three star cards can be "corner 8s"(cards with two adjacent sides having 8, the highest value possible on a 3*)
/// --> These are useful because when placed in a corner with the 8s out, they can only be captured by 4*, 5*, Plus, or Same
/// - Opposing 8s (three star cards with 2 8s on opposite sides) 
/// --> Useful in Plus/Same games, to capitalize on "corner 8s" in adjacent corners.
/// - Three high side cards (generally, some form of 6-8-7-1)
/// --> These cards are used for counterplay in matches with Plus/Same, to present fewer Same/Plus 8 surfaces for attack
/// --> And to capitalize on 1 or 2 point differences for Plus/Same

#[derive(PartialOrd, PartialEq, Eq, Ord, Debug)]
pub enum ThreeStarMetaClasses 
{ TRCorner
, BRCorner
, TLCorner
, BLCorner
, LREight
, TBEight
, TripleT
, TripleR
, TripleL
, TripleB
}

pub fn sum_value_scores(card:&Card) -> usize {
  card.stats.top + card.stats.right + card.stats.bottom + card.stats.left
}

pub fn square_value_scores(card:&Card) -> usize {
  card.stats.top.pow(2) + card.stats.right.pow(2) + card.stats.bottom.pow(2) + card.stats.left.pow(2)
}

#[allow(dead_code)]
pub fn double_eights(card:&Card) -> bool {
  vec![card.stats.top, card.stats.right, card.stats.bottom, card.stats.left]
    .into_iter()
    .filter(|x| x == &8)
    .collect::<Vec<usize>>()
    .len() >= 2
}

pub fn is_corner_eight(card:&Card) -> bool {
  match (card.stats.top, card.stats.right, card.stats.bottom, card.stats.left) {
    (8, 8, _, _) => true,
    (_, 8, 8, _) => true,
    (_, _, 8, 8) => true,
    (8, _, _, 8) => true,
    _ => false
  }
}

pub fn classify_three_star(card:&Card) -> Option<ThreeStarMetaClasses> {
  match card {
    card if card.stars != 3 => None,
    card =>
      match (card.stats.top, card.stats.right, card.stats.bottom, card.stats.left) 
      { (8, 8, _, _) => Some(ThreeStarMetaClasses::TRCorner)
      , (_, 8, 8, _) => Some(ThreeStarMetaClasses::BRCorner)
      , (_, _, 8, 8) => Some(ThreeStarMetaClasses::BLCorner)
      , (8, _, _, 8) => Some(ThreeStarMetaClasses::TLCorner)
      , (8, _, 8, _) => Some(ThreeStarMetaClasses::LREight)
      , (_, 8, _, 8) => Some(ThreeStarMetaClasses::TBEight)
      , (8, r, _, l) if r+l >= 6+7 => Some(ThreeStarMetaClasses::TripleT)
      , (t, 8, b, _) if t+b >= 6+7 => Some(ThreeStarMetaClasses::TripleR)
      , (_, r, 8, l) if r+l >= 6+7 => Some(ThreeStarMetaClasses::TripleB)
      , (t, _, b, 8) if t+b >= 6+7 => Some(ThreeStarMetaClasses::TripleL) 
      , _ => None
      }
  }

}

pub fn explore_cardlist() -> () {
  api::update_cardlist();
  let cardlist = api::read_cardlist();
  for card in &cardlist {
    println!("{:?}", card)
  }
  let filtered_cardlist: Vec<Card> = 
    cardlist.clone()
    .into_iter()
    .filter(|card| card.stars >= 3)
    .collect::<Vec<Card>>();
  for card in filtered_cardlist {
    println!("{:?}", card)
  }
  let three_star_cards = 
    cardlist.clone()
    .into_iter()
    .filter(|card| card.stars == 3)
    .collect::<Vec<Card>>();

  let four_star_cards = 
    cardlist.clone()
    .into_iter()
    .filter(|card| card.stars == 4)
    .collect::<Vec<Card>>();

  let five_star_cards = 
    cardlist.clone()
    .into_iter()
    .filter(|card| card.stars == 5)
    .collect::<Vec<Card>>();

  println!("Number of 3* cards: {}", three_star_cards.len());
  println!("Number of 4* cards: {}", four_star_cards.len());
  println!("Number of 5* cards: {}", five_star_cards.len());

  // let three_star_cards_ordered = 
  //   three_star_cards
  //   .clone()
  //   .into_iter()
  //   .map(|card| (sum_value_scores(&card), card))
  //   .sorted_by(|(c1s, _card1), (c2s, _card2)| Ord::cmp(c1s, c2s))
  //   .collect::<Vec<(usize, Card)>>();

  //   for card in &three_star_cards_ordered {
  //     println!("{:?}", card)
  //   }

  // let three_star_cards_ordered_sq = 
  //   three_star_cards
  //   .clone()
  //   .into_iter()
  //   .map(|card| (square_value_scores(&card), card))
  //   .sorted_by(|(c1s, _card1), (c2s, _card2)| Ord::cmp(c1s, c2s))
  //   .collect::<Vec<(usize, Card)>>();

  // for card in &three_star_cards_ordered_sq {
  //   println!("{:?}", card)
  // }

  // let corner_eights = 
  //   three_star_cards
  //   .clone()
  //   .into_iter()
  //   .filter(is_corner_eight)
  //   .collect::<Vec<Card>>();
  
  // for card in &corner_eights {
  //   println!("{:?}", card)
  // }

  let three_star_cards_by_group =
    three_star_cards
    .clone()
    .into_iter()
    .sorted_by(|c1, c2| Ord::cmp(&classify_three_star(c1), &classify_three_star(c2)))
    .group_by(|card| classify_three_star(card));

  for (key, cards) in &three_star_cards_by_group { 
    let cards = cards.into_iter().collect::<Vec<Card>>();
    println!("Number of {} Cards: {}", key.map(|class| format!("{:?}", class)).unwrap_or("Not meta".to_string()), cards.len());
    for card in cards {
      println!("{:?}", card)
    }
  }
}