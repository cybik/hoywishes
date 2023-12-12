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
use glob::{glob, Paths, PatternError};
use colored::Colorize;
use directories::ProjectDirs;
use spinoff::{Spinner, spinners, Color, Streams};
use crate::commands::consts;

use std::io::{BufWriter, Read};
use log;
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, ConfigBuilder, SimpleLogger, TerminalMode, TermLogger, WriteLogger};
use log_buffer;

// App Identifiers
const QAL: &str = "wishget";
const ORG: &str = "moe.launcher";
const WTT: &str = "A Gacha Wish Tracking Tool";

#[derive(Args)]
pub struct DataArgs {
    #[arg(short = 'g', long)]
    /// Path to the game installation
    pub game_path: PathBuf,

    #[arg(short = 'i', long, default_value_t = false)]
    /// Ignore existing cache when getting data
    pub ignore_cache: bool,

    #[arg(short = 's', long, default_value_t = false)]
    /// Do not write new or updated cache to storage
    pub skip_write_cache: bool,

    #[arg(short = 'a', long, default_value_t = false)]
    /// Process all known banner types
    pub process_all_banners: bool,

    #[arg(short = 'k', long, default_value_t = String::from(""), required_unless_present("game_path"))]
    /// Base URL provided user, if known
    pub known_url: String,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, ValueEnum, Display)]
pub enum Game {
    /// Genshin Impact

    #[strum(serialize = "yuanshen")]
    Genshin, /// 100: Beginner; 200: Permanent; 3xx: Event; 301: Character; 302: Weapon;

    /// Honkai: Star Rail
    #[strum(serialize = "hkrpg")]
    HSR,     /// 1:  Permanent; 2: Beginner;    1x: Event;  11: Character;  12: Weapon

    /// Unsupported
    Unsupported,
}

fn get_glob(args: &DataArgs) -> Option<Result<Paths, PatternError>> {
    match args.game_path.to_str() {
        Some(path) => {
            Some(glob(format!("{path}/{}", consts::cache_dir()).as_str()))
        }
        None => None
    }
}

impl DataArgs {
    fn init_logger(&self) {
        let _ = CombinedLogger::init(
            vec![
                TermLogger::new(
                    LevelFilter::Info,
                    ConfigBuilder::new()
                        .set_time_level(LevelFilter::Off)
                        .set_max_level(LevelFilter::Off)
                        .build(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto
                )
            ]
        );
    }
    pub fn execute(&self) -> anyhow::Result<()> {
        self.init_logger();
        if !self.known_url.is_empty() {
            process_url_func(self.known_url.clone(), self);
        } else {
            if !self.game_path.exists() {
                anyhow::bail!("{}", "Given game path doesn't exist".bold().red());
            }
            match get_glob(self) {
                Some(data_paths) => {
                    for data_path in data_paths.expect("Failed to use glob") {
                        match data_path {
                            Err(_) => {},
                            Ok(path) => {
                                if path.exists() {
                                    match parse_wishes_urls(path) {
                                        Ok(urls) if urls.0.is_empty() => {
                                            anyhow::bail!("{}", "No wishes URL found".red().bold());
                                        }
                                        Ok(urls) => {
                                            process_url_func(urls.0[0].to_string(), self);
                                        }
                                        Err(err) => eprintln!("Failed to parse wishes URLs: {err}")
                                    }
                                    // One empty line to split series
                                    eprintln!();
                                }
                            }
                        }
                    }
                },
                None => {}
            }
        }
        Ok(())
    }
}

// Functional Code.
fn process_url_func(url: String, args: &DataArgs) {
    let urlses = fetch_data(
        url.clone(),
        args.ignore_cache, args.skip_write_cache, args.process_all_banners
    );
    eprintln!("---\n{}", "Final Data Group URL(s)".bold().green());
    urlses.clone().into_iter().for_each(|single_url| {
        eprint!("{}", (if urlses.len() > 1 {"- "} else { "" }));
        println!("{single_url}");
    });
}

fn get_list_of_gacha_types(game: Game, first_type: &String, process_all: bool) -> Vec<String> {
    match process_all {
        true => match game {
            Game::Genshin => {
                [String::from("100"), String::from("200"), String::from("301"), String::from("302")].to_vec()
            }
            Game::HSR => {
                [String::from("1"), String::from("2"), String::from("11"), String::from("12")].to_vec()
            }
            Game::Unsupported => panic!("Why even")
        },
        false => [format!("{first_type}")].to_vec()
    }
}

fn fetch_data_rec(
    acc: &mut json::JsonValue, meta: &mut json::JsonValue, cache: json::JsonValue,
    url: String, gacha_type: &String, page: u8, szgate: usize, end_id: String, ignore_cache: bool
) -> String {
    fetch_data_rec_priv(
        acc, meta, cache,
        url, gacha_type, page, szgate, end_id, false, ignore_cache
    )
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
    meta["uid"].clone().to_string()
}

/// BEGZONE: https://users.rust-lang.org/t/how-to-remove-last-character-from-str/68607/2
trait RemoveLast {
    fn remove_last(&self) -> &Self;
}

impl RemoveLast for str {
    fn remove_last(&self) -> &Self {
        self.strip_suffix(|_: char| true).unwrap_or(self)
    }
}
/// ENDZONE:  https://users.rust-lang.org/t/how-to-remove-last-character-from-str/68607/2

fn urlgen(url: String, gacha_type: String, szgate: usize, page: u8, end_id: String) -> String {
    let url_construct : String = match url.ends_with("#/") {
        true => url.as_str().remove_last().remove_last().to_string(),
        false => url.clone()
    };
    format!("{url_construct}&size={szgate}&page={page}&end_id={end_id}&gacha_type={gacha_type}")
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
                cache.members()
                    .into_iter()
                    .for_each(|member| {
                        acc.push(member.clone()).expect("Could not copy from cache");
                    });
                cache_hit = true;
                break;
            }
        }
        acc.push(elem.clone()).expect("Bonk");
    }
    if (meta["list"].len() != szgate) || run_only_once || cache_hit {
        // Final Recursion.
        meta["uid"] = json::JsonValue::from(
            if acc.members().len() == 0 {"-1"}
            else { acc.members().last().unwrap()["uid"].as_str().unwrap() }
        );
        meta["gacha_type"] = json::JsonValue::from(format!("{gacha_type}"));
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
    proj_dirs.data_local_dir()
        .join(game.to_string())
        .join(uid)
        .join(gacha_type.to_owned()+".cache")
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
            if !uid.contains("-1") {
                let path = generate_path(proj_dirs, game, uid, gacha_type);
                if path.exists() {
                    return json::parse(fs::read_to_string(path).unwrap().as_str()).unwrap();
                }
            }
        }
    }
    json::JsonValue::Null
}

pub fn fetch_data(_url: String, ignore_cache: bool, skip_write_cache: bool, process_all: bool)
    -> Vec<String>
{
    let (game, url, _gacha_type) = build_data_url(_url).unwrap();
    let _vec_banners = get_list_of_gacha_types(game, &_gacha_type, process_all);
    let mut _vec_urls : Vec<String> = Vec::new();
    for gacha_type in _vec_banners {
        let mut acc: json::JsonValue = json::JsonValue::new_array();
        let mut meta: json::JsonValue = json::JsonValue::new_object();
        let mut spinner: Spinner;

        // Sequence
        // 0. Which fucking game.
        // 1. Run it once, get the UID.
        spinner = Spinner::new_with_stream(
            spinners::Dots,
            format!("Fetching Metadata for gacha_type {gacha_type}..."),
            Color::Yellow,
            Streams::Stderr,
        );
        let _uid = fetch_data_rec_once(
            &mut acc, &mut meta, url.clone(), &gacha_type,
        );
        spinner.success("Metadata processed.");

        // 2. Load the local cache, if it exists
        let acc_local = load_local_cache_if_exists(game, &_uid, &gacha_type, ignore_cache);

        // 3. Run it.
        spinner = Spinner::new_with_stream(
            spinners::Dots8bit,
            format!("Fetching Data for gacha_type {}...", gacha_type.clone()),
            Color::Yellow, Streams::Stderr,
        );
        _vec_urls.push(
            fetch_data_rec(
                &mut acc, &mut meta, acc_local,
                url.clone(), &gacha_type,
                1, 5, String::from("0"),
                ignore_cache, // TODO: implement ignore_cache argument to ignore cache.
                //        Cache corruption and user option can happen.
            )
        );
        spinner.success(format!("Fetched data for {} successfully!", gacha_type.clone()).as_str());

        // 4. Write to storage.
        if !skip_write_cache && !_uid.contains("-1") {
            write_to_local_cache(game, &_uid, &gacha_type, &acc);
        }
    }
    _vec_urls
}
