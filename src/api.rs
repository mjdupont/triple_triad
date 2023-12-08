use std::fs::File;
use std::io::Write;
use serde::{Serialize, Deserialize};
use serde_json;

use crate::types::{Tribe, CardStats, Card};



const CARDLIST_API : &str = "https://triad.raelys.com/api/cards";
const CARDLIST_FILENAME : &str = "cardlist.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiStats
{ pub top: usize
, pub right: usize
, pub bottom: usize
, pub left: usize
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ApiStatsWrapper 
{ pub numeric: ApiStats
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ApiType
{ id: usize
, name: String
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ApiCard 
{ pub id: usize
, pub name: String
, pub stars: usize
, pub image: String
, pub image_red: String
, pub image_blue: String
, pub stats: ApiStatsWrapper
, pub r#type: ApiType
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ApiData {
  pub count: usize,
  pub results: Vec<ApiCard>
}

fn translate_type(api_type:ApiType) -> Option<Tribe> {
  match api_type.id {
    1 => Some(Tribe::Primal),
    2 => Some(Tribe::Scion),
    3 => Some(Tribe::Beastman),
    4 => Some(Tribe::Garlean),
    _ => None,
  }
}

fn translate_stats(api_stats:ApiStatsWrapper, api_type: ApiType) -> CardStats {
  let api_stats = api_stats.numeric;
  CardStats 
    { top: api_stats.top 
    , right: api_stats.right
    , bottom: api_stats.bottom
    , left: api_stats.left
    , tribe: translate_type(api_type)
    }
}

fn translate_card(api_card:ApiCard) -> Card {
  Card 
  {  id: api_card.id
  , stars: api_card.stars
  , name: api_card.name
  , stats: translate_stats(api_card.stats, api_card.r#type)
  }
}

pub fn update_cardlist() -> (){
  let resp = 
    reqwest::blocking::get(CARDLIST_API).expect(&format!("GET request to {} failed!", CARDLIST_API))
    .text().expect(&format!("Response from {} could not be parsed as text!", CARDLIST_API));
  let mut file = File::create(CARDLIST_FILENAME).expect("Failed to create cardlist file!");
  write!(file,"{}", resp).expect("Failed to write the cardlist!");
}

fn translate_json_data(api_data:ApiData) -> Vec<Card>{
  api_data.results
  .into_iter()
  .map(translate_card)
  .collect()
}


pub fn read_cardlist() -> Vec<Card> {
  let parsed_values : ApiData = 
    serde_json::from_str(&std::fs::read_to_string(CARDLIST_FILENAME)
                          .expect("Cardlist file must exist and be accessible!"))
    .expect("Serde must be able to parse the cardlist file as a JSON");
  translate_json_data(parsed_values)
}