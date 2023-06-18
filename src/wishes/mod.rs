pub mod data;

use rust_strings::{FileConfig, strings};
//use std::env;
use std::path::{/*Path,*/ PathBuf};
use regex::Regex;
use url::{Url};

enum Game {
    Genshin,
    HSR,
    Unsupported
}

pub fn get_wishes_url(cachepath: PathBuf) -> Option<String> {
    if !cachepath.exists() { // Check 1: does the cache even exist.
        println!("Honq");
        return None;
    }
    let config = FileConfig::new(cachepath.as_path()).with_min_length(25);
    let gatchaweb = Regex::new(r"https://.*/e.*gacha-v2/.*").unwrap();

    match strings(&config) {
        Ok(thestrings) => {
            let _filtered : _ = thestrings
                .iter()
                .filter(
                    |_e| gatchaweb.is_match(_e.0.as_str()) && _e.0.contains("webstatic-sea.hoyoverse.com")
                )
                .map(|_e|_e.0.clone().replacen("1/0/", "", 1))
                .collect::<Vec<String>>();
            return Some(String::from(_filtered.last().unwrap()));
        }
        Err(_) => {}
    }
    None // No wish URLs were found.
}

fn get_param(base_url: Url, param: &str) -> String {
    match base_url.query_pairs().find(
        |(e, _f)| e == param
    ) {
        Some((_r, _re)) => {
            return _re.to_string();
        }
        None => {
            return "".to_string();
        }
    }
}

fn get_game(_url : &Url) -> Game {
    if _url.path().contains("hkrpg") {
        return Game::HSR;
    }
    else if _url.path().contains("Genshin") {
        return Game::Genshin;
    }
    return Game::Unsupported;
}

pub fn get_data_url(cachepath: PathBuf) -> String {
    match Url::parse(get_wishes_url(cachepath).unwrap().as_str()) {
        Ok(base_url) => {
            let mut _url = base_url.clone();
            let mut _queryplus = "".to_string();
            _queryplus += _url.query().unwrap();
            match get_game(&_url) {
                Game::HSR => {
                    _url.set_host(Option::from("api-os-takumi.mihoyo.com")).expect("bonk");
                    _url.set_path("/common/gacha_record/api/getGachaLog");
                    _queryplus += format!("&gacha_type={}", get_param(base_url.clone(), "default_gacha_type")).as_str();
                }
                Game::Genshin => {
                    _url.set_host(Option::from("hk4e-api-os.hoyoverse.com")).expect("bonk");
                    _url.set_path("/event/gacha_info/api/getGachaLog");
                    _queryplus += format!("&gacha_type={}", get_param(base_url.clone(), "init_type")).as_str();
                }
                Game::Unsupported => {
                    panic!("Unsupported game. FOH.")
                }
            }
            return format!(
                "{0}://{1}{2}?{3}",
                _url.scheme(), _url.host_str().unwrap(), _url.path(), _queryplus
            );
        }
        Err(_) => {}
    }
    return "".to_string();
}
