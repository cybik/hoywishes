use std::fs;
use std::fs::{OpenOptions};
use std::io::Write;
use crate::url::{build_data_url, parse_wishes_urls};

use strum::Display;
extern crate reqwest;

use std::path::PathBuf;
use clap::{Args, ValueEnum};
use std::thread::sleep;
use std::time::Duration;
use json;
use glob::glob;
use colored::Colorize;
use directories::ProjectDirs;
use spinoff::{Spinner, spinners, Color, Streams};

// App Identifiers
const QAL: &str = "wishget";
const ORG: &str = "moe.launcher";
const WTT: &str = "A Gacha Wish Tracking Tool";

#[derive(Args)]
pub struct DataArgs {
    #[arg(short = 'p', long)]
    /// Path to the game installation
    pub game_path: PathBuf,

    #[arg(short = 'i', long, default_value_t = false)]
    /// Ignore existing cache when getting data
    pub ignore_cache: bool,

    #[arg(short = 's', long, default_value_t = false)]
    /// Do not write new cache to storage
    pub skip_write_cache: bool,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, ValueEnum, Display)]
pub enum Game {
    /// Genshin Impact

    #[strum(serialize = "yuanshen")]
    Genshin, /// 100: Beginner; 200: Permanent; 3xx: Event; 301: Character; 302: Weapon;

    /// Honkai: Star Rail
    #[strum(serialize = "hkrpg")]
    HSR,     /// 1: Permanent; 2: Beginner; 1x: Event; 11: Character; 12: Weapon

    /// Unsupported
    Unsupported,
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
                                let (acc, meta, url) = fetch_data(
                                    urls[0].clone(), self.ignore_cache, self.skip_write_cache
                                );
                                eprintln!(
                                    "---\n{}\n",
                                    "JSON".bold().green()
                                );
                                println!("{}", acc.pretty(2));
                                eprintln!(
                                    "---\n{}:\n{}\n---\n{}:\n{}",
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
    acc: &mut json::JsonValue, meta: &mut json::JsonValue, cache: json::JsonValue,
    url: String, gacha_type: &String, page: u8, szgate: usize, end_id: String, ignore_cache: bool
) -> String {
    return fetch_data_rec_priv(
        acc, meta, cache,
        url, gacha_type, page, szgate, end_id, false, ignore_cache
    );
}

fn fetch_data_rec_once(
    acc: &mut json::JsonValue, meta: &mut json::JsonValue,
    url: String, gacha_type: &String
) -> String {
    let _ = fetch_data_rec_priv(
        acc, meta, json::JsonValue::Null,
        url, gacha_type, 1, 1, String::from("0"),
        true, true
    );
    acc.clear(); // Reset.
    return meta["uid"].clone().to_string();
}

fn fetch_data_rec_priv(
    acc: &mut json::JsonValue, meta: &mut json::JsonValue, cache: json::JsonValue,
    url: String, gacha_type: &String, page: u8, szgate: usize, end_id: String,
    run_only_once: bool, ignore_cache: bool
) -> String {
    let mut cache_hit = false; // will sort out.
    let req_url = urlgen(url.clone(), gacha_type.clone(), szgate, page, end_id);
    (*meta) = json::parse(
        reqwest::blocking::get(req_url.as_str()).unwrap().text().unwrap().as_str()
    ).unwrap()["data"].clone();
    for elem in meta["list"].members() {
        // if elem["id"] in all the ids of the loaded cache
        // cache_hit = true;
        if cache != json::JsonValue::Null {
            if elem["id"] == cache[0]["id"] {
                // The data from the server is ALWAYS from the most recent to the oldest.
                //  The cache follows the same order. So if the cache's first element is
                //  the same as a given element being processed, we can assume the rest
                //  of the data would be the same. Thus, from that point, load from cache
                //  directly, skip validation, and exit early.
                for elem_cache in cache.members() {
                    acc.push(elem_cache.clone()).expect("Could not copy from cache");
                }
                cache_hit = true;
                break;
            }
        }
        acc.push(elem.clone()).expect("Bonk");
    }
    if (meta["list"].len() != szgate) || run_only_once || cache_hit {
        // Final Recursion.
        meta["uid"] = json::JsonValue::from(acc.members().last().unwrap()["uid"].as_str());
        meta["gacha_type"] = json::JsonValue::from(acc.members().last().unwrap()["gacha_type"].as_str());
        meta["total"] = json::JsonValue::from(acc.len());

        // Cleanup.
        meta.remove("page");
        meta.remove("size");
        meta.remove("list");
        return req_url.clone();
    }
    if !run_only_once { sleep(Duration::from_secs(2)); }
    return fetch_data_rec_priv(
        acc, meta, cache, url, gacha_type,
        page + 1, szgate, String::from(acc[acc.len() - 1]["id"].as_str().unwrap()),
        run_only_once, ignore_cache
    );
}

fn generate_path(proj_dirs: ProjectDirs, game: Game, uid: &String, gacha_type: &String) -> PathBuf {
    return proj_dirs.data_local_dir()
                    .join(game.to_string())
                    .join(uid)
                    .join(gacha_type.to_owned()+".cache");
}

fn write_to_local_cache(game: Game, uid: &String, gacha_type: &String, acc: &json::JsonValue) {
    if let Some(proj_dirs) = directories::ProjectDirs::from(
        QAL, ORG,  WTT
    ) {
        let path = generate_path(proj_dirs, game, uid, gacha_type);
        if !path.exists() {
            if !path.parent().unwrap().exists() {
                fs::create_dir_all(path.parent().unwrap()).expect("Could not create cache dir.");
            }
        }
        write!(
            OpenOptions::new().truncate(true).write(true).create(true).open(path).unwrap(),
            "{}", acc.pretty(2)
        ).expect("boom");
    }
}

fn load_local_cache_if_exists(game: Game, uid: &String, gacha_type: &String, ignore_cache: bool) -> json::JsonValue {
    if !ignore_cache {
        if let Some(proj_dirs) = directories::ProjectDirs::from(
            QAL, ORG,  WTT
        ) {
            let path = generate_path(proj_dirs, game, uid, gacha_type);
            if path.exists() {
                return json::parse(fs::read_to_string(path).unwrap().as_str()).unwrap();
            }
        }
    }
    return json::JsonValue::Null;
}

pub fn fetch_data(_url: String, ignore_cache: bool, skip_write_cache: bool)
    -> (json::JsonValue, json::JsonValue, String)
{
    let (game, url, gacha_type) = build_data_url(_url).unwrap();
    let mut acc: json::JsonValue = json::JsonValue::new_array();
    let mut meta: json::JsonValue = json::JsonValue::new_object();
    let mut spinner : Spinner;

    // Sequence
    // 0. Which fucking game.
    // 1. Run it once, get the UID.
    spinner = Spinner::new_with_stream(
        spinners::Dots, "Fetching Once...", Color::Yellow, Streams::Stderr
    );
    let _uid = fetch_data_rec_once(
        &mut acc, &mut meta, url.clone(), &gacha_type
    );
    spinner.success("Metadata processed.");

    // 2. Load the local cache, if it exists
    let acc_local = load_local_cache_if_exists(game, &_uid, &gacha_type, ignore_cache);

    // 3. Run it.
    spinner = Spinner::new_with_stream(
        spinners::Dots8bit, "Fetching Data...", Color::Yellow, Streams::Stderr
    );
    let final_url = fetch_data_rec(
        &mut acc, &mut meta, acc_local,
        url.clone(), &gacha_type,
        1, 5, String::from("0"),
        ignore_cache // TODO: implement ignore_cache argument to ignore cache.
                               //        Cache corruption and user option can happen.
    );
    spinner.success("Done!");

    // 4. Write to storage.
    if !skip_write_cache {
        write_to_local_cache(game, &_uid, &gacha_type, &acc);
    }

    return (acc, meta, final_url);
}