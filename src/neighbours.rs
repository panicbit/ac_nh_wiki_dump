
use std::collections::BTreeMap;
use select::{node::Node, predicate::*};
use serde::*;
use failure::{Fallible};
use itertools::Itertools;
use crate::common::*;
use crate::id;

#[derive(Debug, Serialize)]
pub struct Neighbour {
    pub id: usize,
    #[serde(rename="name")]
    pub names: BTreeMap<String, String>,
    #[serde(skip)]
    pub image_url: Option<String>,
    pub is_new: bool,
    pub kind: String,
}

pub fn fetch_all() -> Fallible<Vec<Neighbour>> {
    let page = download_page("https://animalcrossingwiki.de/acnh/nachbarn")?;

    let neighbour_nodes = page.find(
        Name("h2")
        .or(Name("table").and(Class("inline")))
    );

    let mut all_neighbours = Vec::new();

    for (kind, table) in neighbour_nodes.tuples() {
        let kind = kind.text();
        let kind = kind.trim();

        let neighbours = parse_table(&table, kind)?;

        all_neighbours.extend(neighbours)
    }

    Ok(all_neighbours)
}

fn parse_table(table: &Node, kind: &str) -> Fallible<Vec<Neighbour>> {
    let rows = table.find(Name("tr"));

    let mut neighbours = Vec::new();

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
        let (mut german_name, english_name);

        match names {
            Some(names) if names.len() == 2 => {
                german_name = names[0].clone();
                english_name = names[1].clone();
            },
            _ => continue,
        }

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

        let id = id::neighbour(&english_name);

        let names = btreemap!{
            "eng".into() => english_name,
            "deu".into() => german_name,
        };

        let neighbour = Neighbour {
            id,
            names,
            image_url: img,
            is_new,
            kind: kind.to_owned(),
        };

        neighbours.push(neighbour);
    }

    Ok(neighbours)
}

impl HasFiles for Neighbour {
    fn files(&self) -> Vec<File> {
        let mut files = vec![];

        if let Some(image_url) = &self.image_url {
            files.push(File {
                name: format!("nb{}.png", self.id),
                url: image_url.clone(),
                transform: convert_image_to_png,
            })
        }

        files.push(File {
            name: format!("nb{}_hi.png", self.id),
            url: format!("https://villagerdb.com/images/villagers/full/{}.png", self.names["eng"].to_lowercase()),
            transform: convert_image_to_png,
        });

        files
    }
}
