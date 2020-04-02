use std::collections::BTreeMap;
use select::{node::Node, predicate::*};
use serde::*;
use failure::{Fallible};
use itertools::Itertools;
use crate::common::*;

#[derive(Debug, Serialize)]
pub struct Fossil {
    pub id: usize,
    #[serde(rename="name")]
    pub names: BTreeMap<String, String>,
    pub image_url: Option<String>,
    pub price: i32,
}

pub fn fetch_all() -> Fallible<Vec<Fossil>> {
    let page = download_page("https://animalcrossing.fandom.com/wiki/Fossils_(New_Horizons)")?;

    let standalone_fossils = {
        let table = page.find(Name("table"))
            .nth(3)
            .unwrap();
        parse_table(table, 3)?
    };

    let multipart_fossils = {
        let table = page.find(Name("table"))
            .nth(5)
            .unwrap();
        parse_table(table, 4)?
    };

    let fossils = standalone_fossils.into_iter()
        .chain(multipart_fossils)
        .enumerate()
        .map(|(id, mut fossil)| {
            fossil.id = id;
            fossil
        })
        .collect_vec();

    Ok(fossils)
}

fn parse_table(table: Node, skip_rows: usize) -> Fallible<Vec<Fossil>> {
    let rows = table.find(Name("tr")).skip(skip_rows);

    let mut fossils = Vec::new();

    for (id, row) in rows.enumerate() {
        let cols = row.find(Name("td")).collect_vec();

        let mut names = BTreeMap::new();

        let english_name = cols.get(0)
            .and_then(|name| parse_text(name.text()))
            .unwrap_or_else(|| "???".into())
            .to_owned();

        names.insert("eng".into(), english_name);
        names.insert("deu".into(), "TBD".into());

        let image_url = cols.get(1)
            .and_then(|img| img
                .find(Name("img"))
                .next()
                .and_then(|img|
                    img.attr("data-src")
                    .or_else(|| img.attr("src"))
                )
                .map(tweak_image_url)
            );

        let price = cols.get(2)
            .and_then(|price| parse_price(price.text()))
            .unwrap_or(-1);

        let fossil = Fossil {
            id,
            image_url,
            names,
            price,
        };

        fossils.push(fossil);
    }

    Ok(fossils)
}

impl HasFiles for Fossil {
    fn files(&self) -> Vec<File> {
        self
            .image_url
            .as_ref()
            .map(|image_url| vec![File {
                name: format!("fo{}.png", self.id),
                url: image_url.clone(),
            }])
            .unwrap_or_default()
    }
}