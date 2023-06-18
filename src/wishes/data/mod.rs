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
    (*meta) = json::parse(
        reqwest::blocking::get(urlgen(url.clone(), szgate, page, end_id).as_str())
            .unwrap().text().unwrap().as_str()
    ).unwrap()["data"].clone();
    let count = ((*meta)["list"]).len();
    for i in 0..((*meta)["list"]).len() {
        acc.push(((*meta)["list"])[i].clone()).expect("Bonk");
    }
    (*meta).remove("list");
    sleep(Duration::from_secs(2));
    if count == szgate {
        fetch_data_rec(
            acc, meta, url, page+1, szgate,
            String::from(acc[acc.len()-1]["id"].as_str().unwrap())
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