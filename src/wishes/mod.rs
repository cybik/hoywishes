use rust_strings::{FileConfig, strings};
//use std::env;
use std::path::{/*Path,*/ PathBuf};
use regex::Regex;
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
