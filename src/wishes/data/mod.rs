extern crate reqwest;

use std::thread::sleep;
use std::time::Duration;
use json;

fn generate(url: String, sizegate: usize, page: u8, end_id: String) -> String {
    let mut url_construct = url.clone();
    url_construct += format!("&size={}&page={}&end_id={}", sizegate, page, end_id).as_str();
    return url_construct;
}
pub fn fetch_deux(url: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut page = 1;
    let mut end_id: String = String::from("0");
    let sizegate : usize = 5;
    let mut last_seen_size: usize = 5;

    let mut accumulated : json::JsonValue = json::JsonValue::new_array();
    while last_seen_size == sizegate {
        let url_constructed = generate(url.clone(), sizegate, page, end_id);
        let res = reqwest::blocking::get(url_constructed.as_str()).unwrap();
        let mut parsed : json::JsonValue = json::parse( res.text().unwrap().as_str()).unwrap();
        let listed : json::JsonValue = parsed["data"]["list"].clone();
        parsed["data"].remove("list");
        end_id = String::from(listed[listed.len()-1]["id"].as_str().unwrap());
        page += 1;
        accumulated.push(listed[0].clone());
        last_seen_size = listed.len();
        sleep(Duration::from_secs(2)); // to kill this.
    }
    println!("JSON:\n{}", accumulated.pretty(2));

    Ok(())
}