use select::document::Document;
use failure::Fallible;
use std::path::Path;
use std::fs;
use ::reqwest::blocking as reqwest;
use regex::Regex;

pub fn parse_text(name: impl AsRef<str>) -> Option<String> {
    let name = name
        .as_ref()
        .trim()
        .to_string();

    Some(name)
}

pub fn parse_price(price: impl AsRef<str>) -> Option<i32> {
    price
        .as_ref()
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse::<i32>()
        .ok()
}

pub fn parse_time_slots(time: impl AsRef<str>) -> Option<Vec<[u8; 2]>> {
    let time = time
        .as_ref()
        .split('&')
        .flat_map(|span| {
            let span = span.trim();

            if span == "All day" {
                return Some([0, 24]);
            }

            let mut span = span.split(" - ");
            let start = span.next().unwrap().trim();
            let start = parse_am_pm_time(start)?;
            let end = span.next().unwrap().trim();
            let end = parse_am_pm_time(end)?;

            Some([start, end])
        })
        .collect::<Vec<[u8; 2]>>();
    
    Some(time)
}

fn parse_am_pm_time(time: &str) -> Option<u8> {
    Some(match time {
        "12 AM" => 0,
        "1 AM" => 1,
        "2 AM" => 2,
        "3 AM" => 3,
        "4 AM" => 4,
        "5 AM" => 5,
        "6 AM" => 6,
        "7 AM" => 7,
        "8 AM" => 8,
        "9 AM" => 9,
        "10 AM" => 10,
        "11 AM" => 11,
        "12 PM" => 12,
        "1 PM" => 12,
        "2 PM" => 13,
        "3 PM" => 14,
        "4 PM" => 16,
        "5 PM" => 17,
        "6 PM" => 18,
        "7 PM" => 19,
        "8 PM" => 20,
        "9 PM" => 21,
        "10 PM" => 22,
        "11 PM" => 23,
        _ => return None,
    })
}

pub fn download_page(url: &str) -> Fallible<Document> {
    let page = reqwest::get(url)
        .unwrap()
        .text()
        .unwrap();

    let page = Document::from(&*page);

    Ok(page)
}

type Url = String;

pub fn download_files(dir: impl AsRef<Path>, items: impl Iterator<Item = (Url, String)>) -> Fallible<()> {
    let dir = dir.as_ref();

    fs::create_dir_all(dir)?;

    for (url, file_name) in items {
        let path = dir.join(file_name);

        println!("Downloading '{}' to '{}'", url, path.display());

        let bytes = reqwest::get(&url)?.bytes()?;

        fs::write(path, bytes)?;
    }

    Ok(())
}

pub fn tweak_image_url(url: impl AsRef<str>) -> String {
    let url = url.as_ref();
    let re = Regex::new(r"/scale-to-width-down/\d+").unwrap();
    let url = re.replace_all(url, "");

    url.into_owned()
}
