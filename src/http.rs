use reqwest::blocking::Client;
use reqwest::header;
use reqwest::header::HeaderValue;
use std::io::Read;
use error_chain::error_chain;
use urlencoding::encode;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}

lazy_static::lazy_static! {
    static ref IMG_CLIENT: Client = {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::HOST, HeaderValue::from_static("img.famly.de"));
        headers.insert(header::USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:102.0) Gecko/20100101 Firefox/102.0"));
        headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
        
        Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to create HTTP client for image downloading")
    };
}

pub fn create_web_client(access_token: String) -> Result<Client> {
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
    
    let client = Client::builder()
        .default_headers(headers)
        .build()?;

    Ok(client)
}

pub fn fetch_child_infos(client: &Client) -> Result<String> {
    let mut body = String::new();
    client
        .get("https://app.famly.de/api/v2/calendar/list")
        .send()?
        .read_to_string(&mut body)?;
    Ok(body)
}

pub fn fetch_feed(client: &Client, older_than: &Option<String>) -> Result<String> {
    let mut url = String::from("https://app.famly.de/api/feed/feed/feed");
    if let Some(date) =  older_than {
        url.push_str("?olderThan=");
        url.push_str(encode(date).into_owned().as_str());
    }

    let body = client
        .get(url)
        .send()?
        .text()?;
    Ok(body)
}

pub fn download_file<W: ?Sized>(_client: &Client, url: &String, writer: &mut W) -> Result<()>
    where W: std::io::Write,
{
    let mut r = IMG_CLIENT
        .get(url)
        .send()?
        .error_for_status()?;

    r.copy_to(writer)?;

    Ok(())
}
