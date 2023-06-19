use crate::url::{build_data_url, parse_wishes_urls};

extern crate reqwest;

use std::path::PathBuf;
use clap::{Args, ValueEnum};
use std::thread::sleep;
use std::time::Duration;
use json;
use glob::glob;
use colored::Colorize;
use spinoff::{Spinner, spinners, Color};

#[derive(Args)]
pub struct DataArgs {
    #[arg(short = 'p', long)]
    /// Path to the game installation
    pub game_path: PathBuf,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, ValueEnum)]
pub enum Game {
    /// Genshin Impact
    Genshin, /// 100: Beginner; 200: Permanent; 3xx: Event; 301: Character; 302: Weapon;

    /// Honkai: Star Rail
    HSR,     /// 1: Permanent; 2: Beginner; 1x: Event; 11: Character; 12: Weapon

    /// Unsupported
    Unsupported
}

impl DataArgs {
    pub fn execute(&self) -> anyhow::Result<()> {
        if !self.game_path.exists() {
            anyhow::bail!("{}", "Given game path doesn't exist".bold().red());
        }
        let _filter = self.game_path.to_str().unwrap().to_owned() + "/**/webCaches/Cache/Cache_Data/data_2";
        for data_path in glob(_filter.as_str()).expect("Failed to read glob pattern")
        {
            match data_path {
                Err(_) => {},
                Ok(path) => {
                    if path.exists() {
                        match parse_wishes_urls(path) {
                            Ok(urls) if urls.is_empty() => {
                                anyhow::bail!("{}", "No wishes URL found".red().bold());
                            }
                            Ok(urls) => {
                                let (acc, meta, url) = fetch_data_recursive( urls[0].clone());
                                println!(
                                    "---\n{}\n{}\n---\n{}:\n{}\n---\n{}:\n{}",
                                    "JSON".bold().green(), acc.pretty(2),
                                    "Metadata".bold().green(), meta.pretty(2),
                                    "Final Data Group URL".bold().green(), url
                                );
                            }
                            Err(err) => eprintln!("Failed to parse wishes URLs: {err}")
                        }

                        // One empty line to split series
                        println!();
                    }
                }
            }
        }

        Ok(())
    }
}

fn urlgen(url: String, gacha_type: String, szgate: usize, page: u8, end_id: String) -> String {
    let mut url_construct = url.clone();
    url_construct += format!("&size={szgate}&page={page}&end_id={end_id}&gacha_type={gacha_type}").as_str();
    return url_construct;
}

fn fetch_data_rec(
    acc: &mut json::JsonValue, meta: &mut json::JsonValue,
    url: String, gacha_type: String, page: u8, szgate: usize, end_id: String
) -> String {
    let req_url = urlgen(url.clone(), gacha_type.clone(), szgate, page, end_id);
    (*meta) = json::parse(reqwest::blocking::get(req_url.as_str()).unwrap().text().unwrap().as_str()
    ).unwrap()["data"].clone();
    let count = meta["list"].len();
    for i in 0..count {
        acc.push(meta["list"][i].clone()).expect("Bonk");
    }
    sleep(Duration::from_secs(2));
    if count != szgate {
        // Final Recursion.
        meta["uid"] = json::JsonValue::from(acc[acc.len() - 1]["uid"].as_str());
        meta["gacha_type"] = json::JsonValue::from(acc[acc.len() - 1]["gacha_type"].as_str());
        meta["total"] = json::JsonValue::from(acc.len());

        // Last call: clean these up
        meta.remove("page");
        meta.remove("size");
        meta.remove("list");
        return req_url.clone();
    }
    return fetch_data_rec(
        acc, meta, url, gacha_type,
        page + 1, szgate, String::from(acc[acc.len() - 1]["id"].as_str().unwrap())
    );
}

pub fn fetch_data_recursive(_url: String) -> (json::JsonValue, json::JsonValue, String) {
    let (url, gacha_type) = build_data_url(_url).unwrap();
    let mut acc: json::JsonValue = json::JsonValue::new_array();
    let mut meta: json::JsonValue = json::JsonValue::new_object();
    let spinner: Spinner = Spinner::new(spinners::Dots8bit, "Fetching...", Color::Yellow);

    let final_url = fetch_data_rec(
        &mut acc, &mut meta,
        url.clone(), gacha_type,
        1, 5, String::from("0")
    );
    spinner.success("Done");
    return (acc, meta, final_url);
}