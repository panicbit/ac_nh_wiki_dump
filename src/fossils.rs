use std::collections::BTreeMap;
use select::{node::Node, predicate::*};
use serde::*;
use failure::{Fallible};
use itertools::Itertools;
use failure::format_err;
use crate::common::*;
use crate::id;

#[derive(Debug, Serialize)]
pub struct Fossil {
    pub id: usize,
    #[serde(rename="name")]
    pub names: BTreeMap<String, String>,
    pub price: i32,
    #[serde(skip)]
    pub image_url: Option<String>,
    #[serde(skip)]
    pub hi_res_image_url: Option<String>,
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

    let mut fossils = standalone_fossils.into_iter()
        .chain(multipart_fossils)
        .enumerate()
        .map(|(id, mut fossil)| {
            fossil.id = id;
            fossil
        })
        .collect_vec();

    let extra_info = fetch_extra_info()?;

    for fossil in &mut fossils {
        let name = fossil.names["eng"].to_lowercase();

        if let Some(extra_info) = extra_info.get(&name) {
            fossil.names.insert("deu".into(), extra_info.german_name.clone());
            fossil.hi_res_image_url = extra_info.hi_res_image_url.clone();
        }
    }

    Ok(fossils)
}

fn parse_table(table: Node, skip_rows: usize) -> Fallible<Vec<Fossil>> {
    let rows = table.find(Name("tr")).skip(skip_rows);

    let mut fossils = Vec::new();

    for row in rows {
        if row.find(Name("th")).count() > 0 {
            continue;
        }

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
            id: id::fossil(&names["eng"]),
            image_url,
            names,
            price,
            hi_res_image_url: None,
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
                transform: convert_image_to_png,
            }])
            .unwrap_or_default()
    }
}

#[derive(Debug)]
struct ExtraInfo {
    // english_name: String,
    german_name: String,
    hi_res_image_url: Option<String>,
}

fn fetch_extra_info() -> Fallible<BTreeMap<String, ExtraInfo>> {
    let page = download_page("https://animalcrossingwiki.de/acnh/katalog/fossilien")?;
    
    let table = page.find(Name("table"))
        .nth(1)
        .ok_or_else(|| format_err!("Could not find bug hi-res table"))?;

    let rows = table.find(Name("tr")).skip(1);

    let mut extra_infos = BTreeMap::new();

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
        let (german_name, english_name);

        match names {
            Some(names) if names.len() == 2 => {
                german_name = names[0].clone();
                english_name = names[1].clone();
            },
            _ => continue,
        }

        let key = english_name.to_lowercase();

        let key = match &*key {
            "archaeopterix" => "archaeopteryx".into(),
            "australopithecus" => "australopith".into(),
            "shark tooth" => "shark-tooth pattern".into(),
            "archelon torso" => "archelon tail".into(),
            "pachy skull" => "pachysaurus skull".into(),
            "pachy tail" => "pachysaurus tail".into(),
            "sabertooth torso" => "sabertooth tail".into(),
            "t. rex-torso" => "t. rex torso".into(),
            _ => key,
        };
        let extra_info = ExtraInfo {
            // english_name,
            german_name,
            hi_res_image_url: img,
        };
 
        extra_infos.insert(key, extra_info);
    }

    Ok(extra_infos)
}
