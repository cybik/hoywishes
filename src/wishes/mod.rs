use rust_strings::{FileConfig, strings};
//use std::env;
use std::path::{/*Path,*/ PathBuf};
use regex::Regex;
use url::{Url, Host, Position};

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
        |(e, f)| e == param
    ) {
        Some((_r, _re)) => {
            return _re.to_string();
        }
        None => {
            return "".to_string();
        }
    }
}

pub fn get_data_url(cachepath: PathBuf, game: Option<&String>) -> String {
    match Url::parse(get_wishes_url(cachepath).unwrap().as_str()) {
        Ok(base_url) => {
            let mut _url = base_url.clone();
            let mut _queryplus = "".to_string();
            _queryplus += _url.query().unwrap();
            if game.unwrap() == "genshin" {
                _url.set_host(Option::from("hk4e-api-os.hoyoverse.com"));
                _url.set_path("/event/gacha_info/api/getGachaLog");
                _queryplus += format!("&gacha_type={}", get_param(base_url.clone(), "init_type")).as_str();
            } else if game.unwrap() == "hsr" {
                _url.set_host(Option::from("api-os-takumi.mihoyo.com"));
                _url.set_path("/common/gacha_record/api/getGachaLog");
                _queryplus += format!("&gacha_type={}", get_param(base_url.clone(), "default_gacha_type")).as_str();
            } else {
                // bonk
            }
            return format!(
                "{0}://{1}{2}?{3}&page=1&size=5&end_id=0",
                _url.scheme(), _url.host_str().unwrap(), _url.path(), _queryplus
            );
        }
        Err(_) => {}
    }
    return "".to_string();
}
