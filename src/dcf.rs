use std::{fs::File, io::{self, BufRead}, path::Path};

use serde::Serialize;

#[derive(Debug)]
#[derive(Serialize)]
pub struct DCFKeyValue {
    pub(crate) key: String,
    pub(crate) value: String,
}

#[derive(Debug)]
#[derive(Serialize)]
pub struct DCFFile(pub Vec<DCFKeyValue>);

impl DCFFile {
    pub fn new() -> Self {
        DCFFile(Vec::new())
    }

    pub fn push(&mut self, item: DCFKeyValue) {
        self.0.push(item);
    }

    pub fn iter(&self) -> impl Iterator<Item = &DCFKeyValue> {
        self.0.iter()
    }

    pub fn get_all_values(&self, key: &str) -> Vec<&str> {
        self.iter()
            .filter(|kv| kv.key == key)
            .map(|kv| kv.value.as_str())
            .collect()
    }
}

pub fn parse_dcf(path: &Path) -> Result<DCFFile, io::Error> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    // let mut package = DCFFile::new();

    let mut dcf = DCFFile::new();

    let mut current_key = String::new();
    let mut current_value = String::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.splitn(2, ':').collect();

        if !line.trim().is_empty() {
            if line.starts_with(" ") || line.starts_with("\t") {
                // println!("indent {:?}", line);
                current_value = current_value + "\r\n" + &line;
            } else if parts.len() >= 2 {
                //if key not empty add to collection
                if !current_key.is_empty() {
                    let kv = DCFKeyValue {
                        key: current_key,
                        value: current_value,
                    };
                    // println!("add key value {:?}", kv);

                    dcf.push(kv);
                }
                // println!("new key {:?}", parts[0]);
                current_key = parts[0].to_string();
                current_value = parts[1].to_string();
            }
        } else {
            println!("enditem");
        }
    }

    Ok(dcf)
}