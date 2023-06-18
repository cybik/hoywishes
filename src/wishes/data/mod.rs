extern crate reqwest;

use std::thread::sleep;
use std::time::Duration;
use json;

fn urlgen(url: String, szgate: usize, page: u8, end_id: String) -> String {
    let mut url_construct = url.clone();
    url_construct += format!("&size={}&page={}&end_id={}", szgate, page, end_id).as_str();
    return url_construct;
}

fn fetch_data_rec(
    acc: &mut json::JsonValue, meta: &mut json::JsonValue,
    url: String, page: u8, szgate: usize, end_id: String
) {
    let mut parseddata : json::JsonValue = json::parse(
        reqwest::blocking::get(urlgen(url.clone(), szgate, page, end_id).as_str())
            .unwrap().text().unwrap().as_str()
    ).unwrap()["data"].clone();
    let listed : json::JsonValue = parseddata["list"].clone();
    parseddata.remove("list");
    *meta = parseddata.clone();
    acc.push(listed[0].clone()).expect("Bonk");
    sleep(Duration::from_secs(2));
    if listed.len() == szgate {
        fetch_data_rec(
            acc, meta, url, page+1, szgate,
            String::from(listed[listed.len()-1]["id"].as_str().unwrap())
        );
    }
}

pub fn fetch_data_recursive(url: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut acc : json::JsonValue = json::JsonValue::new_array();
    let mut meta : json::JsonValue = json::JsonValue::new_object();
    fetch_data_rec(
        &mut acc, &mut meta,
        url, 1, 5, String::from("0")
    );
    meta["uid"] = json::JsonValue::from(acc[acc.len() - 1]["uid"].as_str());
    meta.remove("page");
    meta.remove("size");
    meta["total"] = json::JsonValue::from(acc.len());

    println!("JSON:\n{}", acc.pretty(2));
    println!("Metadata:\n{}", meta.pretty(2));
    Ok(())
}