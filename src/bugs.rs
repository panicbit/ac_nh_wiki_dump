use std::collections::BTreeMap;
use reqwest::blocking::get;
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
}

pub fn fetch_all() -> Fallible<Vec<Bug>> {
    let page = download_page("https://animalcrossing.fandom.com/wiki/Bugs_(New_Horizons)")?;
    let bugs = parse_bugs(page)?;
    Ok(bugs)
}

fn parse_bugs(page: Document) -> Fallible<Vec<Bug>> {
    let north_table = page.find(Name("table"))
        .nth(2)
        .ok_or_else(|| format_err!("Could not find north table"))?;

    let south_table = page.find(Name("table"))
        .nth(4)
        .ok_or_else(|| format_err!("Could not find south table"))?;

    let north_rows = north_table.find(Name("tr")).skip(1);
    let south_rows = south_table.find(Name("tr")).skip(1);

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
        
        // let _img = col[1].next();

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
            names,
            price,
            location,
            north_months,
            south_months,
            time,
        };

        bugs.push(bug);
    }

    Ok(bugs)
}

fn download_page(url: &str) -> Fallible<Document> {
    let page = get(url)
        .unwrap()
        .text()
        .unwrap();

    let page = Document::from(&*page);

    Ok(page)
}
