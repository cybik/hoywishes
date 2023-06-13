extern crate reqwest; // 0.9.18
use url::{Url, Host, Position};
use json;

pub fn fetch(url: String) -> Result<(), Box<dyn std::error::Error>> {
    let res = reqwest::blocking::get(url.as_str()).unwrap();

    println!("Status: {}", res.status());
    println!("Headers:\n{:#?}", res.headers());
    let body = res.text().unwrap();
    println!("Body:\n{}", body);

    Ok(())
}