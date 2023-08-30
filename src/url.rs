use std::path::Path;

use crate::commands::data::Game;

use url::{Url};

/// Try to parse wishes URLs from the given data_2 file path
/// 
/// Return values from most recent to the oldest cached URL
pub fn parse_wishes_urls(data_path: impl AsRef<Path>) -> anyhow::Result<(Vec<String>, Option<Game>)> {
    // This should be written this way
    let data = std::fs::read(data_path)?;
    let data = String::from_utf8_lossy(&data);

    // TODO: somehow URLs that are actually irrelevant get caught up. Investigate.
    let urls = data.split('\n').rev()
        // Find last line with part of url we need
        .filter(|line| line.contains("gacha-v2/"))

        // And if we found one - split it by \0 char
        .filter_map(|url| url.split('\0')

        // First non-empty block (with 1/0/ prefix) is our url
        .find(|part| part.starts_with("1/0/")))

        // Strip useless 1/0/ prefix in found url
        .filter_map(|url| url.strip_prefix("1/0/"))

        // And convert &str to String (owned variant)
        .map(|url| url.to_string())

        // Convert found URLs to the vector
        .collect::<Vec<String>>();
    // Rust memory model, yey
    let first_url = match urls.is_empty() {
        false => Some(urls[0].clone()),
        true => None
    };

    Ok((
        urls,
        match first_url {
            None => None,
            Some(first) => Some(get_game(&Url::parse(first.as_str()).expect("Could not parse.")))
        }
    ))
}

fn get_game(_url : &Url) -> Game {
    if _url.path().contains("hkrpg") {
        return Game::HSR;
    } else if _url.path().contains("genshin") {
        return Game::Genshin;
    }
    return Game::Unsupported;
}

pub fn build_data_url(history_url: impl AsRef<str>) -> Option<(Game, String, String)> {
    let Some(query) = history_url.as_ref().split("/index.html?").last() else {
        return None;
    };
    match Url::parse(history_url.as_ref()) {
        Ok(base_url) => {
            #[inline]
            fn get_gacha_type<'a>(query: &'a str, key: &str) -> Option<&'a str> {
                // Split arguments string by key get request
                query.split(key)

                    // Take second value (something&key=[value]&something)
                    .nth(1)

                    // And split this value again by & and take the first part
                    // If there weren't & - then we'll get the value itself.
                    // Otherwise only needed query value
                    .and_then(|value| value.split('&').next())
            }
            match get_game(&base_url) {
                Game::Genshin => get_gacha_type(query, "init_type=")
                    .map(|value| (Game::Genshin, format!("https://hk4e-api-os.hoyoverse.com/event/gacha_info/api/getGachaLog?{query}"), String::from(value))),

                Game::HSR => get_gacha_type(query, "default_gacha_type=")
                    .map(|value| (Game::HSR, format!("https://api-os-takumi.mihoyo.com/common/gacha_record/api/getGachaLog?{query}"), String::from(value))),

                Game::Unsupported => {
                    panic!("Unsupported game. FOH.")
                }
            }
        }
        Err(_) => {
            None
        }
    }
}
