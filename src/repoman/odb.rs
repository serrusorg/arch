use flate2::read::GzDecoder;
use repodb_parser::pkg::Pkg;
use std::{ffi::OsStr, io::BufReader};
use tar::Archive;

use crate::downman;

pub struct OriginalDatabase {
    pub name: String,
    pub pkgs: Vec<Pkg>,
}

impl OriginalDatabase {
    pub fn new(name: &str, database: Vec<u8>) -> Self {
        let decoder = GzDecoder::new(database.as_slice());
        let mut pkgs = Vec::new();

        for entry in Archive::new(decoder).entries().unwrap() {
            let entry = entry.unwrap();
            let path = entry.path().unwrap().into_owned();

            if !entry.header().entry_type().is_file()
                || path.file_name() != Some(OsStr::new("desc"))
            {
                continue;
            }

            pkgs.push(Pkg::parse(BufReader::new(entry)).unwrap());
        }

        Self {
            name: name.to_string(),
            pkgs,
        }
    }

    pub fn download(name: &str, mirror_list: &Vec<String>) -> Option<Self> {
        for mirror in mirror_list {
            let url = format!("{}/{}/os/x86_64/{}.db", mirror, name, name);
            let body = match downman::download(&url, format!("{}.db", name).as_str()) {
                Some(body) => body,
                None => continue,
            };
            return Some(OriginalDatabase::new(&name.to_string(), body));
        }

        None
    }
}
