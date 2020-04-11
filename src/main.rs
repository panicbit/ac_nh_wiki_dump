use serde_json as json;
use common::*;
use std::fs;

mod bugs;
mod fossils;
mod fish;
mod common;
mod id;

const IMAGE_DL_FOLDER: &str = "images";
const DATA_FOLDER: &str = "data";

fn main() {
    fs::create_dir_all(DATA_FOLDER).unwrap();

    // ### Bugs ###
    let bugs = bugs::fetch_all().expect("bugs");
    let json_bugs = json::to_string_pretty(&bugs).unwrap();
    fs::write("data/insects.json", json_bugs).unwrap();

    // ### Fish ###
    let fishes = fish::fetch_all().expect("fish");
    let json_fishes = json::to_string_pretty(&fishes).unwrap();
    fs::write("data/fish.json", json_fishes).unwrap();

    // ### Fossils ###
    let fossils = fossils::fetch_all().expect("fossils");
    let json_fossils = json::to_string_pretty(&fossils).unwrap();
    fs::write("data/fossils.json", json_fossils).unwrap();

    // let old_fish_json = include_bytes!("../../../Downloads/ac_res/res/raw/fish.json");
    // let mut old_fishes: Vec<json::Value> = json::from_slice(old_fish_json).unwrap();

    // for old_fish in &mut old_fishes {
    //     let old_fish = old_fish.as_object_mut().unwrap();
    //     let id = old_fish["id"].as_i64().unwrap();
    //     let fish = fishes.iter().find(|fish| fish.id == id as usize).unwrap();
    //     let shadow = serde_json::to_value(&fish.shadow).unwrap();
    //     old_fish.insert("shadow".into(), shadow);
    // }

    // let old_fishes = json::to_string_pretty(&old_fishes).unwrap();
    // fs::write("/tmp/fish.json", old_fishes).unwrap();

    // download_images(bugs, IMAGE_DL_FOLDER).unwrap();
    // download_images(fish, IMAGE_DL_FOLDER).unwrap();
    // download_images(fossils, IMAGE_DL_FOLDER).unwrap();
}
