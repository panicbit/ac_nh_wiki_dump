use serde_json as json;
use common::*;

mod bugs;
mod fossils;
mod fish;
mod common;

fn main() {
    // ### Bugs ###
    let bugs = bugs::fetch_all().expect("bugs");
    let json_bugs = json::to_string_pretty(&bugs).unwrap();
    println!("{}", json_bugs);

    // ### Fish ###
    let fish = fish::fetch_all().expect("fish");
    let json_fish = json::to_string_pretty(&fish).unwrap();
    println!("{}", json_fish);

    // ### Fossils ###
    let fossils = fossils::fetch_all().expect("fossils");
    let json_fossils = json::to_string_pretty(&fossils).unwrap();
    println!("{}", json_fossils);

    // let to_download = fossils
    //     .iter()
    //     .flat_map(|fossil|
    //         fossil.image_url.clone().map(|image_url| {
    //             let file_name = format!("fo{}.png", fossil.id);
    //             (image_url, file_name)
    //         })
    //     );
    
    // download_files("dl/fossils", to_download).unwrap();
}