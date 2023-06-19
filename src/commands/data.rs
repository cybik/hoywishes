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
    Genshin,

    /// Honkai: Star Rail
    HSR,

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
                                let acc : json::JsonValue;
                                let meta : json::JsonValue;
                                (acc, meta) = fetch_data_recursive( build_data_url(urls[0].clone()).unwrap());
                                println!(
                                    "{}:\n{}\n{}:\n{}",
                                    "JSON".bold().green(), acc.pretty(2),
                                    "Metadata".bold().green(), meta.pretty(2)
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


fn urlgen(url: String, szgate: usize, page: u8, end_id: String) -> String {
    let mut url_construct = url.clone();
    url_construct += format!("&size={szgate}&page={page}&end_id={end_id}").as_str();
    return url_construct;
}

fn fetch_data_rec(
    acc: &mut json::JsonValue, meta: &mut json::JsonValue,
    url: String, page: u8, szgate: usize, end_id: String
) {
    (*meta) = json::parse(
        reqwest::blocking::get(urlgen(url.clone(), szgate, page, end_id).as_str())
            .unwrap().text().unwrap().as_str()
    ).unwrap()["data"].clone();
    let count = (*meta)["list"].len();
    for i in 0..count {
        acc.push((*meta)["list"][i].clone()).expect("Bonk");
    }
    sleep(Duration::from_secs(2));
    if count == szgate {
        fetch_data_rec(
            acc, meta, url, page + 1, szgate,
            String::from(acc[acc.len() - 1]["id"].as_str().unwrap())
        );
    } else {
        // Final Recursion.
        meta["uid"] = json::JsonValue::from(acc[acc.len() - 1]["uid"].as_str());
        meta["total"] = json::JsonValue::from(acc.len());

        // Last call: clean these up
        meta.remove("page");
        meta.remove("size");
        meta.remove("list");
    }
}

pub fn fetch_data_recursive(url: String)
    -> (json::JsonValue, json::JsonValue )
{
    let mut acc: json::JsonValue = json::JsonValue::new_array();
    let mut meta: json::JsonValue = json::JsonValue::new_object();
    let spinner: Spinner = Spinner::new(spinners::Dots8bit, "Fetching...", Color::Yellow);

    fetch_data_rec(
        &mut acc, &mut meta,
        url, 1, 5, String::from("0")
    );
    spinner.success("Done");
    return (acc, meta);
}