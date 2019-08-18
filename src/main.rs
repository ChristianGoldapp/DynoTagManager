use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::error::Error;
use reqwest::{Client, header::{HeaderValue, HeaderMap, COOKIE, CONTENT_TYPE}, Response};
use serde_json::Value;
use serde::export::fmt::Debug;
use std::fmt::{Formatter, Display};
use std::fmt;
use std::env::args;

fn main() -> Result<(), Box<Error>> {
    let config = load_config()?;
    let args: Vec<_> = args().collect();
    println!("{:?}", args);
    if args[1] == "list" {
        println!("{:#?}", config.list_tags()?);
    } else if args[1] == "create" {
        let tag = Tag {
            tag: args[2].to_string(),
            content: args[3].to_string(),
        };
        match config.create_tag(tag) {
            Ok(_) => {println!("Tag successfully created!\n{}", tag)}
            Err(e) => Err(e)
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct DynoInstance {
    cookie: String,
    server: String,
}

#[derive(Serialize, Deserialize)]
struct Tag {
    tag: String,
    content: String,
}

impl Debug for Tag {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Name   : {}\nContent: {}", self.tag, self.content)
    }
}

impl DynoInstance {
    fn create_tag(&self, tag: Tag) -> Result<Response, Box<Error>> {
        let url = format!("https://dyno.gg/api/server/{}/tags/create", self.server);
        let mut headers = get_headers(&self)?;
        headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json")?);
        println!("{:?}", headers);
        let http = Client::builder().build()?;
        let body = serde_json::to_string(&tag)?;
        println!("{}", body);
        let request = http.post(url.as_str())
            .headers(headers)
            .body(body);
        println!("{:?}", request);
        let response = request.send()?;
        Ok(response)
    }

    fn list_tags(&self) -> Result<Vec<Tag>, Box<Error>> {
        let url = format!("https://dyno.gg/api/modules/{}/tags/list", self.server);
        let headers = get_headers(&self)?;
        let http = Client::builder().default_headers(headers).build()?;
        let mut response = http.get(url.as_str()).send()?;
        let json: Value = response.json()?;
        let tag_json: &Value = &json["tags"];
        let vec: Vec<Tag> = match &tag_json {
            &Value::Array(v) => {
                v.into_iter().map(|json| Tag { tag: json["tag"].to_string(), content: json["content"].to_string() }).collect()
            }
            _ => {
                eprintln!("Could not parse JSON returned from server!");
                vec![]
            }
        };
        Ok(vec)
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

