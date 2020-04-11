
use std::collections::BTreeMap;
use select::{node::Node, predicate::*};
use serde::*;
use failure::{Fallible};
use itertools::Itertools;
use crate::common::*;
use crate::id;

#[derive(Debug, Serialize)]
pub struct Flower {
    pub id: usize,
    #[serde(rename="name")]
    pub names: BTreeMap<String, String>,
    #[serde(skip)]
    pub image_url: Option<String>,
    #[serde(skip)]
    pub hi_res_image_url: Option<String>,
    pub sources: Vec<Source>,
}

pub fn fetch_all() -> Fallible<Vec<Flower>> {
    let page = download_page("https://animalcrossingwiki.de/acnh/blumen")?;

    let flower_nodes = page.find(
        Name("h2")
        .or(Class("level2").descendant(Name("table")))
    );

    let mut all_flowers = Vec::new();

    for (category, table) in flower_nodes.tuples() {
        let category = category.text();

        // println!("Category: {}", category);

        let mut flowers = parse_table(&table)?;

        enrich_flowers_with_sources(&mut flowers, &table);

        all_flowers.extend(flowers)
    }

    Ok(all_flowers)
}

fn parse_table(table: &Node) -> Fallible<Vec<Flower>> {
    let rows = table.find(Name("tr"));

    let mut flowers = Vec::new();

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
            "black cosmos" => "Schwarzcosmea".into(),
            "blue roses" => "Blaurose".into(),
            _ => german_name,
        };

        let id = id::flower(&english_name);

        let names = btreemap!{
            "eng".into() => english_name,
            "deu".into() => german_name,
        };

        let flower = Flower {
            id,
            names,
            image_url: img,
            sources: vec![],
            hi_res_image_url: None,
        };

        flowers.push(flower);
    }

    Ok(flowers)
}

fn enrich_flowers_with_sources(flowers: &mut [Flower], table: &Node) {    
    let rows = table.find(Name("tr"));

    for row in rows {
        if row.find(Name("th")).count() > 0 {
            continue;
        }

        let cols = row.find(Name("td")).collect_vec();

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

        let id = id::flower(&english_name);

        let sources = parse_source_rules(&cols[2], flowers);
        let flower = flowers.iter_mut().find(|flower| flower.id == id).unwrap();
        flower.sources = sources;
    }
}

fn parse_source_rules(node: &Node, all_flowers: &[Flower]) -> Vec<Source> {
    let text_sources = node
        .children()
        .flat_map(|node| {
            if node.name() == Some("img") {
                if node.attr("src").unwrap().contains("/stern.png") {
                    return Some("★".to_owned());
                }

                return Some(
                    node
                    .attr("title")
                    .unwrap()
                    .trim()
                    .to_owned())
                ;
            }

            match node.text().trim() {
                "" => return None,
                "×" => return Some("x".into()),
                _ => {},
            };

            Some(
                node
                .text()
                .trim()
                .to_owned()
            )
        })
        .join(" ");

    if text_sources.is_empty() {
        return vec![];
    }

    let sources = text_sources
        .split(" oder ")
        .map(|mut source| {
            let mut requires_gold_watering_can = false;
            let can = " + Goldgießkanne";
            if source.ends_with(can) {
                requires_gold_watering_can = true;

                source = &source[..source.len()-can.len()];
            }

            let mut requires_cultivated_flowers = false;
            let flowers = source
                .split(" x ")
                .map(|mut flower| {
                    let star = " ★";
                    if flower.ends_with(star) {
                        requires_cultivated_flowers = true;
                        flower = &flower[..flower.len()-star.len()];
                    }

                    flower
                })
                .collect_vec();
            assert_eq!(flowers.len(), 2);
            let flowers = [
                all_flowers.iter().find(|flower| flower.names["deu"] == flowers[0]).unwrap().id,
                all_flowers.iter().find(|flower| flower.names["deu"] == flowers[1]).unwrap().id
            ];

            Source {
                flowers,
                requires_gold_watering_can,
                requires_cultivated_flowers,
            }
        })
        .collect_vec();
    
    sources
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Source {
    flowers: [usize; 2],
    requires_gold_watering_can: bool,
    requires_cultivated_flowers: bool,
}

impl HasFiles for Flower {
    fn files(&self) -> Vec<File> {
        let mut files = vec![];

        if let Some(image_url) = &self.image_url {
            files.push(File {
                name: format!("fl{}.png", self.id),
                url: image_url.clone(),
                transform: convert_image_to_png,
            })
        }

        if let Some(hi_res_image_url) = &self.hi_res_image_url {
            files.push(File {
                name: format!("fl{}_hi.png", self.id),
                url: hi_res_image_url.clone(),
                transform: convert_image_to_png,
            })
        }

        files
    }
}
