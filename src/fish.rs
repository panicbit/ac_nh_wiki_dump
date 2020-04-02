use std::collections::BTreeMap;
use select::document::Document;
use select::predicate::*;
use serde::*;
use failure::{Fallible, format_err};
use itertools::Itertools;
use crate::common::*;

#[derive(Debug, Serialize)]
pub struct Fish {
    id: usize,
    #[serde(rename="name")]
    names: BTreeMap<String, String>,
    price: i32,
    location: String,
    shadow: Shadow,
    time: Vec<[u8; 2]>,
    #[serde(rename="months_north")]
    north_months: Vec<bool>,
    #[serde(rename="months_south")]
    south_months: Vec<bool>,
    image_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Shadow {
    size: i8,
    is_narrow: bool,
    has_fin: bool,
}

pub fn fetch_all() -> Fallible<Vec<Fish>> {
    let page = download_page("https://animalcrossing.fandom.com/wiki/Fish_(New_Horizons)")?;
    let fishs = parse_fish(page)?;
    Ok(fishs)
}

fn parse_fish(page: Document) -> Fallible<Vec<Fish>> {
    let north_table = page.find(Name("table"))
        .nth(2)
        .ok_or_else(|| format_err!("Could not find north table"))?;

    let south_table = page.find(Name("table"))
        .nth(4)
        .ok_or_else(|| format_err!("Could not find south table"))?;

    let north_rows = north_table.find(Name("tr")).skip(1);
    let south_rows = south_table.find(Name("tr")).skip(1);

    let mut fishs = Vec::new();

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
        
        let shadow = north_cols.get(4)
            .map(|shadow| parse_shadow(shadow.text()))
            .unwrap_or(Shadow {
                size: -1,
                is_narrow: false,
                has_fin: false,
            });

        let time = north_cols.get(5)
            .and_then(|time| parse_time_slots(time.text()))
            .unwrap_or_else(Vec::new);

        let mut north_months = north_cols.get(6..)
            .unwrap_or(&[])
            .iter()
            .take(12)
            .map(|n| n.text())
            .map(|checked| checked.trim() == "✓")
            .collect::<Vec<bool>>();
        north_months.resize(12, false);

        let mut south_months = south_cols.get(6..)
            .unwrap_or(&[])
            .iter()
            .take(12)
            .map(|n| n.text())
            .map(|checked| checked.trim() == "✓")
            .collect::<Vec<bool>>();
        south_months.resize(12, false);

        let fish = Fish {
            id,
            image_url,
            names,
            price,
            location,
            shadow,
            north_months,
            south_months,
            time,
        };

        fishs.push(fish);
    }

    Ok(fishs)
}

fn parse_shadow(shadow: impl AsRef<str>) -> Shadow {
    let shadow = shadow.as_ref().to_lowercase();
    let is_narrow = shadow.contains("narrow");
    let has_fin = shadow.contains("fin");
    let size = shadow
        .chars()
        .filter(char::is_ascii_digit)
        .collect::<String>()
        .parse::<i8>()
        .unwrap_or(-1);

    Shadow { size, is_narrow, has_fin }
}

impl HasFiles for Fish {
    fn files(&self) -> Vec<File> {
        self
            .image_url
            .as_ref()
            .map(|image_url| vec![File {
                name: format!("f{}.png", self.id),
                url: image_url.clone(),
            }])
            .unwrap_or_default()
    }
}
