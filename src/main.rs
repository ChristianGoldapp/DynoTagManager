use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::error::Error;
use reqwest::{Client, header::{HeaderValue, HeaderMap, COOKIE, CONTENT_TYPE}, Response};
use serde_json::Value;
use serde::export::fmt::{Debug, Display};
use std::fmt::Formatter;
use std::fmt;
use std::env::args;
use simple_error::SimpleError;

fn main() -> Result<(), Box<Error>> {
    let config = load_config()?;
    let dyno = DynoInstance::build(config)?;
    let args: Vec<_> = args().collect();
    println!("{:?}", args);
    if args[1] == "list" {
        println!("{:#?}", dyno.list_tags()?);
    } else if args[1] == "create" {
        let tag = Tag {
            tag: args[2].to_string(),
            content: args[3].to_string(),
            id: "Not a real Id!".to_string(),
        };
        match dyno.create_tag(&tag) {
            Ok(_) => { println!("Tag successfully created!\n{}", &tag) }
            Err(e) => { return Err(e); }
        }
    } else if args[1] == "delete" {
        let name = args[2].to_string();
        match dyno.delete_tag(&name) {
            Ok(_) => { println!("Tag successfully deleted!\n{}", &name) }
            Err(e) => { return Err(e); }
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct DynoConfig {
    cookie: String,
    server: String,
}

struct DynoInstance {
    cookie: String,
    server: String,
    http: Client,
}

#[derive(Serialize, Deserialize)]
struct Tag {
    tag: String,
    content: String,
    id: String,
}

#[derive(Serialize, Deserialize)]
struct TagRef {
    tag: String,
    name: String,
}

impl Debug for Tag {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Name   : {}\nContent: {}", self.tag, self.content)
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Name   : {}\nContent: {}", self.tag, self.content)
    }
}

impl DynoInstance {
    fn build(instance: DynoConfig) -> Result<DynoInstance, Box<Error>> {
        Ok(DynoInstance {
            cookie: instance.cookie,
            server: instance.server,
            http: Client::builder().build()?,
        })
    }

    fn create_tag(&self, tag: &Tag) -> Result<Response, Box<Error>> {
        let url = format!("https://dyno.gg/api/server/{}/tags/create", self.server);
        let mut headers = get_headers(&self)?;
        headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json")?);
        println!("{:?}", headers);
        let body = serde_json::to_string(&tag)?;
        println!("{}", body);
        let request = self.http.post(url.as_str())
            .headers(headers)
            .body(body);
        println!("{:?}", request);
        let response = request.send()?;
        Ok(response)
    }

    fn list_tags(&self) -> Result<Vec<Tag>, Box<Error>> {
        let url = format!("https://dyno.gg/api/modules/{}/tags/list", self.server);
        let headers = get_headers(&self)?;
        let mut response = self.http.get(url.as_str()).headers(headers).send()?;
        let json: Value = response.json()?;
        let tag_json: &Value = &json["tags"];
        let vec: Vec<Tag> = match &tag_json {
            &Value::Array(v) => {
                v.into_iter().map(|json|
                    {
                        let tag = Tag {
                            tag: as_string(&json["tag"]),
                            content: as_string(&json["content"]),
                            id: as_string(&json["_id"]),
                        };
                        println!("{}", &tag);
                        tag
                    }).collect()
            }
            _ => {
                eprintln!("Could not parse JSON returned from server!");
                vec![]
            }
        };
        Ok(vec)
    }

    fn delete_tag(&self, name: &String) -> Result<Response, Box<Error>> {
        let url = format!("https://dyno.gg/api/server/{}/tags/delete", self.server);
        let mut headers = get_headers(&self)?;
        headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json")?);

        let tagslist = self.list_tags()?;
        let to_be_deleted =
            match tagslist.into_iter().find(|t| t.tag == name.to_string()) {
                None => { return Err(Box::new(SimpleError::new(format!("Tag named {} has not been found.", name)))); }
                Some(tag) => tag
            };
        let tag_ref = TagRef {
            tag: to_be_deleted.id.to_string(),
            name: to_be_deleted.tag.to_string(),
        };
        let body = serde_json::to_string(&tag_ref)?;
        let request = self.http.post(url.as_str())
            .headers(headers)
            .body(body);
        let response = request.send()?;
        Ok(response)
    }
}

fn load_config() -> Result<DynoConfig, Box<Error>> {
    let file = File::open("DynoTagManagerConfig.json")?;
    let reader = BufReader::new(file);
    let c: DynoConfig = serde_json::from_reader(reader)?;
    Ok(c)
}

fn get_headers(instance: &DynoInstance) -> Result<HeaderMap, Box<Error>> {
    let mut headers = HeaderMap::new();
    let cookie = instance.cookie.as_str();
    let cookie_header = HeaderValue::from_str(cookie)?;
    headers.insert(COOKIE, cookie_header);
    Ok(headers)
}

fn as_string(value: &Value) -> String {
    match value.as_str() {
        Some(str) => str.to_string(),
        _ => value.to_string()
    }
}

