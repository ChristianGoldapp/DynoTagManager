use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::error::Error;

fn main() {
    let config = load_config();
    match config {
        Ok(c) => {
            println!("{:?}", c);
        }
        Err(e) => {
            eprintln!("Error reading config file! {}", e);
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    cookie: String,
    server: String,
}

fn load_config() -> Result<Config, Box<Error>> {
    let file = File::open("DynoTagManagerConfig.json")?;
    let reader = BufReader::new(file);
    let c: Config = serde_json::from_reader(reader)?;
    Ok(c)
}