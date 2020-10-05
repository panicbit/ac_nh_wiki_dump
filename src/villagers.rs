use std::collections::BTreeMap;
use select::{node::Node, predicate::*};
use serde::*;
use failure::{Fallible};
use itertools::Itertools;
use crate::common::*;
use crate::id;
use rayon::prelude::*;
use crate::villagerdb;

#[derive(Debug, Serialize)]
pub struct Villager {
    pub id: usize,
    #[serde(rename="name")]
    pub names: BTreeMap<String, String>,
    #[serde(skip)]
    pub image_url: Option<String>,
    pub is_new: bool,
    pub species: String,
    pub gender: Gender,
    pub birthday: Option<[u8; 2]>,
    #[serde(rename="phrase")]
    pub phrases: BTreeMap<String, String>,
    // #[serde(rename="photo_phrase")]
    // pub photo_phrases: BTreeMap<String, String>,
    #[serde(rename="personalities")]
    pub personalities: BTreeMap<String, String>,
}

pub fn fetch_all() -> Fallible<Vec<Villager>> {
    let page = download_page("https://animalcrossingwiki.de/acnh/nachbarn")?;

    let villager_nodes = page.find(
        Name("h2")
        .or(Name("table").and(Class("inline")))
    );

    let mut all_villagers = Vec::new();

    for (kind, table) in villager_nodes.tuples() {
        let kind = kind.text();
        let kind = kind.trim();

        let villagers = parse_table(&table, kind)?;

        all_villagers.extend(villagers)
    }

    for villager in &mut all_villagers {
        let name = &villager.names["eng"];
        println!("Querying villagedb about villager '{}'", name);
        let db_villager = villagerdb::get_villager(name)?;

        assert_eq!(villager.names["eng"], db_villager.name);

        villager.species = db_villager.species;
        villager.phrases.insert("eng".into(), db_villager.games.nh.phrase);
        villager.personalities.insert("eng".into(), db_villager.games.nh.personality);
    }

    all_villagers.par_iter_mut()
        .try_for_each(enrich_with_extra_info)?;

    Ok(all_villagers)
}

fn parse_table(table: &Node, kind: &str) -> Fallible<Vec<Villager>> {
    let rows = table.find(Name("tr"));

    let mut villagers = Vec::new();

    for row in rows {
        if row.find(Name("th")).count() > 0 {
            continue;
        }

        let cols = row.find(Name("td")).collect_vec();

        let img = cols.get(0)
            .and_then(|img| img.find(Name("img")).next())
            .and_then(|img| img.attr("src"))
            .map(tweak_image_url);
        let img = match img {
            Some(img) if img.contains("bildfehlt") => None,
            Some(img) if img.starts_with('/') => Some(format!("https://animalcrossingwiki.de{}", img)),
            Some(img) => Some(img),
            None => continue,
        };

        let names = cols.get(1)
            .map(|name| name
                .text()
                .split('\n')
                .map(|name| name.trim().to_owned())
                .collect_vec()
            );
        let (mut german_name, mut english_name);

        match names {
            Some(names) if names.len() == 2 => {
                german_name = names[0].clone();
                english_name = names[1].clone();
            },
            _ => continue,
        }

        // Fix some english names
        english_name = match &*english_name {
            "Marrcel" => "Marcel".into(),
            "Sidney" => "Sydney".into(),
            "Gretel" => "Greta".into(),
            "Candy" => "Candi".into(),
            "Stiches" => "Stitches".into(),
            _ => english_name,
        };

        // Fix some missing names
        german_name = match &*english_name {
            // "black cosmos" => "Schwarzcosmea".into(),
            // "blue roses" => "Blaurose".into(),
            _ => german_name,
        };

        let is_new = cols
            .get(3)
            .map(|is_new| 
                !is_new
                .text()
                .trim()
                .is_empty()
            )
            .unwrap_or(false);

        let id = id::villager(&english_name);

        let names = btreemap!{
            "eng".into() => english_name,
            "deu".into() => german_name,
        };

        let villager = Villager {
            id,
            names,
            image_url: img,
            is_new,
            species: kind.to_owned(),
            gender: Gender::Unknown,
            birthday: None,
            phrases: BTreeMap::new(),
            // photo_phrases: BTreeMap::new(),
            personalities: BTreeMap::new(),
        };

        villagers.push(villager);
    }

    Ok(villagers)
}

fn enrich_with_extra_info(villager: &mut Villager) -> Fallible<()> {
    let lowercase_name = villager.names["deu"].to_lowercase();
    let url = format!("https://animalcrossingwiki.de/nachbarn/{}", lowercase_name);
    println!("Fetching '{}'", url);
    let page = download_page(&url)?;

    // main data
    {
        let table = page.find(
            Class("wrap_nachbarntabelle")
                .descendant(Name("table"))
            ).next().unwrap();
        let rows = table.find(Name("tr")).skip(2);

        for row in rows {
            let cols = row.find(Name("th").or(Name("td"))).collect_vec();
            let field = cols[0].text().trim().to_lowercase();
            let value = cols[1].text().trim().to_owned();

            match &*field {
                "geschlecht" => match &*value.to_lowercase() {
                    "weiblich" => villager.gender = Gender::Female,
                    "männlich" => villager.gender = Gender::Male,
                    gender => panic!("Unknown gender '{}'", gender),
                },
                "tierart" => {},
                | "persönlichkeit"
                | "persönlichkeit." => {
                    villager.personalities.insert("deu".into(), value.trim().into());
                },
                "geburtstag" => {
                    let parts = value.trim().split('.').map(str::trim).collect_vec();
                    let day = parts[0].parse::<u8>().unwrap();
                    let month = match parts[1] {
                        "Januar" => 1,
                        "Februar" => 2,
                        "März" => 3,
                        "April" => 4,
                        "Mai" => 5,
                        "Juni" => 6,
                        "Juli" => 7,
                        "August" => 8,
                        "September" => 9,
                        "Oktober" => 10,
                        "November" => 11,
                        "Dezember" => 12,
                        month => panic!("unknown month '{}'", month),
                    };
                    villager.birthday = Some([day, month]);
                },
                "floskel" => {
                    let phrase = value
                        .trim()
                        .replace('„', "")
                        .replace('“', "")
                        .replace('"', "");
                    villager.phrases.insert("deu".into(), phrase);
                },
                "fotospruch" => {
                    // villager.photo_phrases.insert("deu".into(), value.trim().into());
                },
                "auftreten" => {},
                field => panic!("Unknown field '{}', value: '{}'", field, value),
            }

            // println!(">>> {:#?}", cols[0]);
        }
    } 

    // println!("{:#?}", main_table);

    Ok(())
}

#[derive(Debug, Serialize)]
#[serde(rename_all="lowercase")]
pub enum Gender {
    Unknown,
    Male,
    Female,
}

impl HasFiles for Villager {
    fn files(&self) -> Vec<File> {
        let mut files = vec![];
        let name = villagerdb::get_villager_db_name(&self.names["eng"]);

        // if let Some(image_url) = &self.image_url {
        //     files.push(File {
        //         name: format!("wiki/{}.png", name),
        //         url: image_url.clone(),
        //         transform: convert_image_to_png,
        //     })
        // }

        files.push(File {
            name: format!("villagerdb/{}.png", name),
            url: format!("https://villagerdb.com/images/villagers/full/{}.png", name),
            transform: convert_image_to_png,
        });

        files
    }
}
