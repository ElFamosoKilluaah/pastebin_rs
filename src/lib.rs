#[cfg(test)]
mod tests;

use std::collections::HashMap;

use tokio::runtime::Runtime;

static API_URL: &'static str = "https://pastebin.com/api/api_post.php";
static PASTEBIN_URL_START: &'static str = "https://pastebin.com";

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum VisibilityLevel {
    Public = 0,
    Unlisted = 1,
}

impl VisibilityLevel {
    pub fn from(s: String) -> Option<VisibilityLevel> {
        let from = s.to_lowercase();

        match from.as_str() {
            "public"  | "0" => Some(VisibilityLevel::Public),
            "unlisted" | "1" => Some(VisibilityLevel::Unlisted),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PastebinError {
    InvalidKey,
    BlockedIP,
    EmptyPasteContent,
    PasteTooBig,
    InvalidPasteFormat,
    Unknown(String),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ExpirationDate {
    Never,
    TenMinutes,
    OneHour,
    OneDay,
    OneWeek,
    TwoWeeks,
    OneMonth,
    SixMonth,
    OneYear,
}

impl ExpirationDate {
    pub fn inspect(&self) -> &str {
        match self {
            ExpirationDate::Never => "N",
            ExpirationDate::TenMinutes => "10M",
            ExpirationDate::OneHour => "1H",
            ExpirationDate::OneDay => "1D",
            ExpirationDate::OneWeek => "1W",
            ExpirationDate::TwoWeeks => "2W",
            ExpirationDate::OneMonth => "1M",
            ExpirationDate::SixMonth => "6M",
            ExpirationDate::OneYear => "1Y",
        }
    }

    pub fn from(s: String) -> Option<ExpirationDate> {
        let from = s.to_lowercase();

        match from.as_str() {
            "n" => Some(ExpirationDate::Never),
            "10m" => Some(ExpirationDate::TenMinutes),
            "1h" => Some(ExpirationDate::OneHour),
            "1d" => Some(ExpirationDate::OneDay),
            "1w" => Some(ExpirationDate::OneWeek),
            "2w" => Some(ExpirationDate::TwoWeeks),
            "1m" => Some(ExpirationDate::OneMonth),
            "6m" => Some(ExpirationDate::SixMonth),
            "1y" => Some(ExpirationDate::OneYear),
            _ => None,
        }
    }
}

pub struct PastebinBuilder {
    api: PastebinApi,
    paste_name: Option<String>,
    visibility: Option<VisibilityLevel>,
    text: String,
    format: Option<String>,
    expire_date: Option<ExpirationDate>,
}

impl PastebinBuilder {
    pub fn new(
        api_key: String,
        text: String,
        paste_name: Option<String>,
        visibility: Option<VisibilityLevel>,
        format: Option<String>,
        expire_date: Option<ExpirationDate>,
    ) -> PastebinBuilder {
        PastebinBuilder {
            api: PastebinApi::new(api_key),
            paste_name,
            visibility,
            text,
            format,
            expire_date,
        }
    }

    pub fn execute(&self) -> Result<String, PastebinError> {
        self.api.upload(
            self.paste_name.clone(),
            self.visibility.clone(),
            self.text.clone(),
            self.format.clone(),
            self.expire_date.clone(),
        )
    }
}

pub struct PastebinApi {
    api_key: String,
}

impl PastebinApi {
    pub fn new(api_key: String) -> PastebinApi {
        PastebinApi { api_key }
    }

    async fn call_api(
        &self,
        json: &HashMap<&str, String>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let client = reqwest::Client::new();
        client.post(API_URL).form(json).send().await
    }

    pub fn upload(
        &self,
        paste_name: Option<String>,
        visibility: Option<VisibilityLevel>,
        text: String,
        format: Option<String>,
        expire_date: Option<ExpirationDate>,
    ) -> Result<String, PastebinError> {
        // Those errors are predictables before sending the request.
        if text.len() > 512000 {
            return Err(PastebinError::PasteTooBig);
        }

        if text.len() == 0 {
            return Err(PastebinError::EmptyPasteContent);
        }

        let mut map = HashMap::new();

        // Sets the api type to paste.
        map.insert("api_option", "paste".to_string());

        // Sets the api key
        map.insert("api_dev_key", self.api_key.clone());

        // Sets the paste content.
        map.insert("api_paste_code", format!("{}", text));

        // Adds the visibility key to the request if not None.
        if let Some(v) = visibility {
            map.insert("api_paste_private", format!("{}", v as u8));
        }

        // Adds the paste name key to the request if not None.
        if let Some(name) = paste_name {
            map.insert("api_paste_name", name);
        }

        // Adds the expiration date key to the request if not None.
        if let Some(exp_date) = expire_date {
            map.insert("api_paste_expire_date", format!("{}", exp_date.inspect()));
        }

        // Adds the text format key to the request if not None.
        if let Some(f) = format {
            map.insert("api_paste_format", f);
        }

        #[cfg(test)]
        tests::print_json(&map);

        let response = Runtime::new()
            .expect("Can\'t initialize the Tokio's runtime.")
            .block_on(self.call_api(&map));

        match response {
            Ok(r) => {
                let plain_text = futures::executor::block_on(r.text()).unwrap();

                if plain_text.starts_with(PASTEBIN_URL_START) {
                    return Ok(plain_text);
                } 
                
                else {
                    #[cfg(test)]
                    println!("Error: {}", &plain_text);

                    // let (_, reason) = plain_text.split_at(16);
                    let reason = plain_text.as_str();

                    match reason {
                        "invalid api_dev_key" => return Err(PastebinError::InvalidKey),
                        "IP blocked" => return Err(PastebinError::BlockedIP),
                        "invalid api_paste_format" => {
                            return Err(PastebinError::InvalidPasteFormat)
                        }
                        _ => return Err(PastebinError::Unknown(format!("{}", reason))),
                    }
                }
            }

            Err(e) => return Err(PastebinError::Unknown(format!("{:?}", e))),
        }
    }
}
