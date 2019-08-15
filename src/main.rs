use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::error::Error;
use reqwest::{Client, header::{HeaderValue, HeaderMap, COOKIE}, Response};

fn main() -> Result<(), Box<Error>>{
    let config = load_config()?;
    println!("{:?}", config);

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct DynoInstance {
    cookie: String,
    server: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Tag {
    tag: String,
    content: String,
}

impl DynoInstance {
    fn create_tag(&self, tag: Tag) -> Result<Response, Box<Error>>{
        let url = format!("https://dyno.gg/api/server/{}/tags/create", self.server);
        let headers = get_headers(&self)?;
        let http = Client::builder().default_headers(headers).build()?;
        let response = http.post(url.as_str()).body(
            serde_json::to_string(&tag)?
        ).send()?;
        Ok(response)
    }
}

fn load_config() -> Result<DynoInstance, Box<Error>> {
    let file = File::open("DynoTagManagerConfig.json")?;
    let reader = BufReader::new(file);
    let c: DynoInstance = serde_json::from_reader(reader)?;
    Ok(c)
}

fn get_headers(config: &DynoInstance) -> Result<HeaderMap, Box<Error>> {
    let mut headers = HeaderMap::new();
    let cookie = config.cookie.as_str();
    let cookie_header = HeaderValue::from_str(cookie)?;
    headers.insert(COOKIE, cookie_header);
    Ok(headers)
}

