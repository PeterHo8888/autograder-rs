use curl::easy::{Easy, List};
use std::io::prelude::*;
use std::io::Read;

use super::{get_token, API};

/*
 * Retrieve file from URL
 */
pub fn fetch_file(url: &str) -> Vec<u8> {
    get(url, None)
}

/*
 * Helper function to retrieve raw JSON from Canvas LMS REST API
 */
pub fn fetch_api(path: &str) -> Vec<u8> {
    let mut list = List::new();
    list.append("Content-Type: application/json").unwrap();
    list.append("Charset: UTF-8").unwrap();
    list.append(get_token()).unwrap();
    // Set up URL
    let url = format!("{}{}", API, path);
    get(&url, Some(list))
}

/*
 * Helper function to PUT json to Canvas
 */
pub fn put_json(path: &str, data: &str) {
    let mut list = List::new();
    list.append(get_token()).unwrap();
    list.append("Content-Type: application/json").unwrap();
    list.append(&format!("Content-Length: {}", data.len())).unwrap();
    let url = format!("{}{}", API, path);
    let mut data = data.as_bytes();
    put(&url, &mut data, Some(list));
}

/*
 * PUT data given URL, data, and optional headers
 */
fn put(url: &str, mut json: &[u8], list: Option<List>) {
    let mut handle = Easy::new();

    handle.url(url).unwrap();
    handle.follow_location(true).unwrap(); // 3xx redirects
    handle.upload(true).unwrap();
    handle.custom_request("PUT").unwrap();
    handle.in_filesize(json.len() as u64).unwrap();

    if let Some(header) = list {
        handle.http_headers(header).unwrap();
    }

    loop {
        // Callback
        let mut transfer = handle.transfer();

        transfer.read_function(|buf| {
            Ok(json.read(buf).unwrap_or(0))
        }).unwrap();

        if let Ok(_) = transfer.perform() {
            break;
        }
    }
}

/*
 * Retrieve data from given URL and optional headers
 */
fn get(url: &str, list: Option<List>) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut handle = Easy::new();

    handle.url(url).unwrap();
    handle.follow_location(true).unwrap(); // 3xx redirects

    if let Some(header) = list {
        handle.http_headers(header).unwrap();
    }

    loop {
        // Callback
        let mut transfer = handle.transfer();

        transfer.write_function(|data| {
            buf.extend_from_slice(data);
            Ok(data.len())
        }).unwrap();

        if let Ok(_) = transfer.perform() {
            break;
        }
    }
    buf
}

