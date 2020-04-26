use select::document::Document;
use failure::{Fallible, bail};
use std::path::Path;
use std::fs;
use ::reqwest::blocking as reqwest;
use regex::Regex;
use image::{ImageFormat, imageops::FilterType};
use threadpool::ThreadPool;

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

pub fn download_images<T: HasFiles>(items: impl IntoIterator<Item = T>, dir: impl AsRef<Path>) -> Fallible<()> {
    let tasks = ThreadPool::new(4);
    let dir = dir.as_ref();

    fs::create_dir_all(dir)?;

    for item in items {
        for file in item.files() {
            if tasks.panic_count() > 0 {
                bail!("Failed to download some files");
            }

            let dir = dir.to_owned();

            tasks.execute(move || {
                let path = dir.join(file.name);

                println!("Downloading '{}' to '{}'", file.url, path.display());

                let bytes = reqwest::get(&file.url).unwrap().bytes().unwrap().to_vec();
                let bytes = (file.transform)(bytes);

                fs::write(path, bytes).unwrap();
            });
        }
    }

    tasks.join();

    if tasks.panic_count() > 0 {
        bail!("Failed to download some files");
    }

    Ok(())
}

pub fn tweak_image_url(url: impl AsRef<str>) -> String {
    let url = url.as_ref();
    let re = Regex::new(r"(/scale-to-width-down/\d+)|(w=\d+)").unwrap();
    let url = re.replace_all(url, "");

    url.into_owned()
}

pub trait HasFiles {
    fn files(&self) -> Vec<File>;
}

pub struct File {
    pub name: String,
    pub url: String,
    pub transform: fn(Vec<u8>) -> Vec<u8>,
}

impl<T: HasFiles> HasFiles for &'_ T {
    fn files(&self) -> Vec<File> {
        (*self).files()
    }
}

pub fn convert_image_to_png(source: Vec<u8>) -> Vec<u8> {
    let source = image::load_from_memory(&source).unwrap()
        .resize(512, 512, FilterType::Lanczos3);

    let mut target = Vec::new();
    source.write_to(&mut target, ImageFormat::Png).unwrap();

    target
}
