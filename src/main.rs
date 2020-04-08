use serde_json as json;
use common::*;
use std::fs;

mod bugs;
mod fossils;
mod fish;
mod common;
// mod indices;

const IMAGE_DL_FOLDER: &str = "images";

fn main() {
    fs::create_dir_all("data").unwrap();

    // ### Bugs ###
    let bugs = bugs::fetch_all().expect("bugs");
    let json_bugs = json::to_string_pretty(&bugs).unwrap();
    fs::write("data/bugs.json", json_bugs).unwrap();

    // ### Fish ###
    let fish = fish::fetch_all().expect("fish");
    let json_fish = json::to_string_pretty(&fish).unwrap();
    fs::write("data/fish.json", json_fish).unwrap();

    // ### Fossils ###
    let fossils = fossils::fetch_all().expect("fossils");
    let json_fossils = json::to_string_pretty(&fossils).unwrap();
    fs::write("data/fossils.json", json_fossils).unwrap();

    download_images(bugs, IMAGE_DL_FOLDER).unwrap();
    download_images(fish, IMAGE_DL_FOLDER).unwrap();
    download_images(fossils, IMAGE_DL_FOLDER).unwrap();
}
