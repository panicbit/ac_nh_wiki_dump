use serde_json as json;
use common::*;

mod bugs;
mod fossils;
mod fish;
mod common;

const IMAGE_DL_FOLDER: &str = "images";

fn main() {
    // ### Bugs ###
    let bugs = bugs::fetch_all().expect("bugs");
    let json_bugs = json::to_string_pretty(&bugs).unwrap();
    println!("{}", json_bugs);
    download_images(bugs, IMAGE_DL_FOLDER).unwrap();

    // ### Fish ###
    let fish = fish::fetch_all().expect("fish");
    let json_fish = json::to_string_pretty(&fish).unwrap();
    println!("{}", json_fish);
    download_images(fish, IMAGE_DL_FOLDER).unwrap();

    // ### Fossils ###
    let fossils = fossils::fetch_all().expect("fossils");
    let json_fossils = json::to_string_pretty(&fossils).unwrap();
    println!("{}", json_fossils);
    download_images(fossils, IMAGE_DL_FOLDER).unwrap();
}
