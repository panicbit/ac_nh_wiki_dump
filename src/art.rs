use std::collections::BTreeMap;
use select::{node::Node, predicate::*};
use serde::*;
use failure::{Fallible};
use itertools::Itertools;
use crate::common::*;
use crate::id;

#[derive(Debug, Serialize)]
pub struct Art {
    pub id: usize,
    #[serde(rename="name")]
    pub names: BTreeMap<String, String>,
    pub kind: &'static str,
    pub fake_exists: bool,
    #[serde(skip)]
    pub fake_image_url: Option<String>,
    pub fake_description: BTreeMap<String, String>,
    #[serde(skip)]
    pub image_url: Option<String>,
}

pub fn fetch_all() -> Fallible<Vec<Art>> {
    let page = download_page("https://animalcrossingwiki.de/acnh/reiner")?;

    let art_nodes = page.find(
        Class("level2")
        .descendant(Class("desktoponly"))
        .descendant(Name("table"))
    );

    let mut all_art = Vec::new();

    for table in art_nodes {
        let art = parse_table(&table)?;
        all_art.extend(art)
    }

    Ok(all_art)
}

fn parse_table(table: &Node) -> Fallible<Vec<Art>> {
    let rows = table.find(Name("tr"));

    let mut all_art = Vec::new();

    for row in rows {
        if row.find(Name("th")).count() > 0 {
            continue;
        }

        let cols = row.find(Name("td")).collect_vec();

        let fake_exists = cols.len() > 3;
        let name_index = if fake_exists { 2 } else { 1 };

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

        let fake_img = if !fake_exists { None } else {
            let img = cols.get(1)
                .and_then(|img| img.find(Name("img")).next())
                .and_then(|img| img.attr("src"))
                .map(tweak_image_url);
            let img = match img {
                Some(img) if img.contains("bildfehlt") => None,
                Some(img) if img.starts_with('/') => Some(format!("https://animalcrossingwiki.de{}", img)),
                Some(img) => Some(img),
                None => continue,
            };

            img
        };

        let mut fake_description = BTreeMap::new();

        if fake_exists {
            let description = cols.get(4)
                .map(|text| text.text().trim().to_string());
            
            if let Some(description) = description {
                fake_description.insert("deu".into(), description);
            }
        };

        let names = cols.get(name_index)
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

        let kind = if english_name.contains("statue") { "statue" }
            else if english_name.contains("painting") { "painting" }
            else { panic!("Unknown art kind: '{}'", english_name) };

        // Fix some missing names
        german_name = match &*english_name {
            // "black cosmos" => "Schwarzcosmea".into(),
            // "blue roses" => "Blaurose".into(),
            _ => german_name,
        };

        let id = id::art(&english_name);

        let names = btreemap!{
            "eng".into() => english_name,
            "deu".into() => german_name,
        };

        let art = Art {
            id,
            names,
            fake_exists,
            fake_description,
            kind,
            fake_image_url: fake_img,
            image_url: img,
        };

        all_art.push(art);
    }

    Ok(all_art)
}

impl HasFiles for Art {
    fn files(&self) -> Vec<File> {
        let mut files = vec![];

        if let Some(image_url) = &self.image_url {
            files.push(File {
                name: format!("art{}.png", self.id),
                url: image_url.clone(),
                transform: convert_image_to_png,
            })
        }

        if let Some(fake_image_url) = &self.fake_image_url {
            files.push(File {
                name: format!("art{}_fake.png", self.id),
                url: fake_image_url.clone(),
                transform: convert_image_to_png,
            })
        }

        files
    }
}
