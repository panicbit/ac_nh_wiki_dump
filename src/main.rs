
use serde_json as json;

mod bugs;
mod common;

fn main() {
    let bugs = bugs::fetch_all().expect("bugs");
    let json_bugs = json::to_string_pretty(&bugs).unwrap();

    println!("{}", json_bugs);
}
