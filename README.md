# Pastebin_rs

Just a small Pastebin crate to upload some pastes.

## How to use

```rust
use pastebin_rs::{ExpirationDate, PastebinBuilder, VisibilityLevel};

fn main() {
    let content = "Some cool stuff".to_string();
    let paste_name = "A cool title".to_string();
    let api_key = "YOUR DEVELOPER API KEY HERE".to_string();

    // Initializes a PastebinApi struct to upload some stuffs.
    let pastebin = PastebinBuilder::new(
        api_key,
        content,
        Some(paste_name),
        Some(VisibilityLevel::Unlisted),
        Some(format!("rust")),
        Some(ExpirationDate::TenMinutes),
    );

    let url = pastebin.execute().unwrap();
    println!("Your resulting pastebin url is {}", url);
}
```