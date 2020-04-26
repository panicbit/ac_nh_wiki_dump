use serde::Deserialize;
use failure::Fallible;
use std::fs;

pub fn get_villager(name: &str) -> Fallible<Villager> {
    let mut name = name
        .trim()
        .to_lowercase()
        .replace(' ', "-")
        .replace('.', "")
        .replace('\'', "")
        .replace('é', "e");
    
    match &*name {
        "sally" => name = "sally2".into(),
        "hazel" => name = "hazel2".into(),
        "carmen" => name = "carmen2".into(),
        _ => {},
    }

    let path = format!("villagerdb/data/villagers/{}.json", name);
    let data = fs::read(path)?;
    let villager = serde_json::from_slice::<Villager>(&data)?;
    Ok(villager)
}

#[derive(Deserialize)]
pub struct Villager {
    pub name: String,
    pub species: String,
    pub games: GamesVillager,
}

#[derive(Deserialize)]
pub struct GamesVillager {
    pub nh: NHVillager,
}

#[derive(Deserialize)]
pub struct NHVillager {
    pub personality: String,
    pub phrase: String,
    pub song: Option<String>,
}
