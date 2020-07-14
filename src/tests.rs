use crate::{ExpirationDate, PastebinApi, VisibilityLevel};

use std::collections::HashMap;

#[test]
fn test_upload() {
    let api_key = std::env::var("PASTEBIN_API_KEY").unwrap();
    let api = PastebinApi::new(api_key);

    println!("Test");

    let result = api.upload(
        Some(format!("Test paste")),
        Some(VisibilityLevel::Unlisted),
        format!("Ahahah XDXD"),
        Some(format!("rust")),
        Some(ExpirationDate::TenMinutes),
    );

    println!("{:?}", result);
}

#[cfg(test)]
pub fn print_json(map: &HashMap<&str, String>) {
    for (k, v) in map {
        println!("{}: {}", k, v);
    }
}