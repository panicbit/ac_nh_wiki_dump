use std::collections::BTreeMap;
use select::document::Document;
use select::predicate::*;
use serde::*;
use failure::{Fallible, format_err};
use itertools::Itertools;
use crate::common::*;

#[derive(Debug, Serialize)]
pub struct Bug {
    id: usize,
    #[serde(rename="name")]
    names: BTreeMap<String, String>,
    price: i32,
    location: String,
    time: Vec<[u8; 2]>,
    #[serde(rename="months_north")]
    north_months: Vec<bool>,
    #[serde(rename="months_south")]
    south_months: Vec<bool>,
    image_url: Option<String>,
    hi_res_image_url: Option<String>,
}

pub fn fetch_all() -> Fallible<Vec<Bug>> {
    let page = download_page("https://animalcrossing.fandom.com/wiki/Bugs_(New_Horizons)")?;
    let mut bugs = parse_bugs(page)?;
    let extra_info = fetch_extra_info()?;

    for bug in &mut bugs {
        let name = bug.names["eng"].to_lowercase();
        
        if let Some(extra_info) = extra_info.get(&name) {
            bug.names.insert("deu".into(), extra_info.german_name.clone());
            bug.hi_res_image_url = extra_info.hi_res_image_url.clone();
        }
    }

    Ok(bugs)
}

fn parse_bugs(page: Document) -> Fallible<Vec<Bug>> {

    let north_table = page.find(Name("table"))
        .nth(2)
        .ok_or_else(|| format_err!("Could not find north table"))?;

    let south_table = page.find(Name("table"))
        .nth(4)
        .ok_or_else(|| format_err!("Could not find south table"))?;

    let north_rows = north_table.find(Name("tr")).skip(3);
    let south_rows = south_table.find(Name("tr")).skip(3);

    let mut bugs = Vec::new();

    for (id, (north_row, south_row)) in north_rows.zip(south_rows).enumerate() {
        let north_cols = north_row.find(Name("td")).collect_vec();
        let south_cols = south_row.find(Name("td")).collect_vec();

        let mut names = BTreeMap::new();
        let english_name = north_cols.get(0)
            .and_then(|name| parse_text(name.text()))
            .unwrap_or_else(|| "???".into())
            .to_owned();

        names.insert("eng".into(), english_name);
        names.insert("deu".into(), "TBD".into());

        let image_url = north_cols.get(1)
            .and_then(|img| img
                .find(Name("img"))
                .next()
                .and_then(|img| img.attr("data-src"))
                .map(tweak_image_url)
                .map(<_>::into)
            );

        let price = north_cols.get(2)
            .and_then(|price| parse_price(price.text()))
            .unwrap_or(-1);
        
        let location = north_cols.get(3)
            .and_then(|location| parse_text(location.text()))
            .unwrap_or_else(|| "???".into());
        
        let time = north_cols.get(4)
            .and_then(|time| parse_time_slots(time.text()))
            .unwrap_or_else(Vec::new);

        let mut north_months = north_cols.get(5..)
            .unwrap_or(&[])
            .iter()
            .take(12)
            .map(|n| n.text())
            .map(|checked| checked.trim() == "✓")
            .collect::<Vec<bool>>();
        north_months.resize(12, false);

        let mut south_months = south_cols.get(5..)
            .unwrap_or(&[])
            .iter()
            .take(12)
            .map(|n| n.text())
            .map(|checked| checked.trim() == "✓")
            .collect::<Vec<bool>>();
        south_months.resize(12, false);

        let bug = Bug {
            id,
            image_url,
            names,
            price,
            location,
            north_months,
            south_months,
            time,
            hi_res_image_url: None,
        };

        bugs.push(bug);
    }

    Ok(bugs)
}

impl HasFiles for Bug {
    fn files(&self) -> Vec<File> {
        let mut files = vec![];

        if let Some(image_url) = &self.image_url {
            files.push(File {
                name: format!("i{}.png", self.id),
                url: image_url.clone(),
            })
        }

        if let Some(hi_res_image_url) = &self.hi_res_image_url {
            files.push(File {
                name: format!("i{}_hi.png", self.id),
                url: hi_res_image_url.clone(),
            })
        }

        files
    }
}

struct ExtraInfo {
    // english_name: String,
    german_name: String,
    hi_res_image_url: Option<String>,
}

fn fetch_extra_info() -> Fallible<BTreeMap<String, ExtraInfo>> {
    let page = download_page("https://animalcrossingwiki.de/acnh/insekten")?;
    
    let table = page.find(Name("table"))
        .nth(1)
        .ok_or_else(|| format_err!("Could not find bug hi-res table"))?;

    let rows = table.find(Name("tr")).skip(1);

    let mut extra_infos = BTreeMap::new();

    for row in rows {
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
        let extra_info = ExtraInfo {
            // english_name,
            german_name,
            hi_res_image_url: img,
        };
 
        extra_infos.insert(key, extra_info);
    }

    Ok(extra_infos)
}
