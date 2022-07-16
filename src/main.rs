mod config;

use reqwest::header;
use reqwest::header::HeaderValue;
use error_chain::error_chain;
use std::io::Read;

use config::Config;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}

fn create_web_client(access_token: String) -> Result<reqwest::blocking::Client> {
    let mut access_token_header_val = HeaderValue::from_str(access_token.as_str()).unwrap();
    access_token_header_val.set_sensitive(true);

    let mut headers = header::HeaderMap::new();
    headers.insert(header::HOST, HeaderValue::from_static("app.famly.de"));
    headers.insert(header::USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:102.0) Gecko/20100101 Firefox/102.0"));
    headers.insert(header::REFERER, HeaderValue::from_static("https://app.famly.de/"));
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("x-famly-accesstoken", access_token_header_val);
    headers.insert("x-famly-installationid", HeaderValue::from_static("297e6a1d-d070-4e54-b6a4-3a73a325ccc1"));
    headers.insert("x-famly-platform", HeaderValue::from_static("html"));
    headers.insert("x-famly-version", HeaderValue::from_static("2153d828df"));
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    
    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()?;

    Ok(client)
}

fn main() -> Result<()> {
    let env = Config::new();

    let client = create_web_client(env.access_token)?;

    let mut body = String::new();
    client
        //.get("https://app.famly.de/api/feed/feed/feed?olderThan=2022-07-09T11%3A36%3A29%2B00%3A00")
        .get("https://app.famly.de/api/v2/calendar/list")
        .send()?
        .read_to_string(&mut body)?;

    println!("{}", body);

    Ok(())
}
