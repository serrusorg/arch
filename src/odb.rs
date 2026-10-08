use flate2::read::GzDecoder;
use repodb_parser::pkg::Pkg;
use std::{ffi::OsStr, io::BufReader};
use tar::Archive;

struct OriginalDatabase {
    name: String,
    pkgs: Vec<Pkg>,
}

impl OriginalDatabase {
    pub fn new(name: String, database: Vec<u8>) -> Self {
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

        Self { name, pkgs }
    }

    pub fn download(name: String, mirror_list: &Vec<String>) -> Self {
        let mut bytes = Vec::new();
        for mirror in mirror_list {
            let url = format!("{}/x86_64/os/{}.db", mirror, name);

            if let Ok(response) = ureq::get(&url).call() {
                if let Ok(mut body) = response.into_body().read_to_vec() {
                    bytes.append(&mut body);
                    break;
                }
            }
        }

        OriginalDatabase::new(name, bytes)
    }
}
