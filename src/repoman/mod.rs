pub mod odb;

use std::collections::BTreeMap;

use crate::pkgman::pkg::{Package, PackageDependency, PackageDependencyMetadata};
pub use crate::repoman::odb::OriginalDatabase;
use bitcode::{Decode, Encode, decode, encode};

#[derive(Decode, Encode, PartialEq, Eq)]
pub struct Repository {
    pub name: String,
    pub packages: BTreeMap<String, Package>,
    pub provides_reverse_table: BTreeMap<String, String>,
}

impl Repository {
    pub fn load(data: &[u8]) -> Result<Self, &str> {
        decode(data).map_err(|_| "decoding error")
    }

    pub fn save(&self) -> Vec<u8> {
        encode(self)
    }
}

pub fn convert(database: OriginalDatabase) -> Repository {
    let mut packages = BTreeMap::new();
    let mut provides_reverse_table = BTreeMap::new();
    for package in database.pkgs {
        let pkg = Package {
            name: package.name.clone(),
            file: package.file_name,

            version: package.version,

            license: package.license,

            replaces: package.replaces,
            provides: package.provides.clone(),
            conflicts: package.conflicts,
            dependencies: PackageDependency {
                runtime: package
                    .deps
                    .iter()
                    .map(|x| PackageDependencyMetadata::new(x.pkg_name.clone(), x.rstr.clone()))
                    .collect(),
                optional: package
                    .opt_deps
                    .iter()
                    .map(|x| PackageDependencyMetadata::new(x.pkg_name.clone(), x.rstr.clone()))
                    .collect(),
                testing: package
                    .check_deps
                    .iter()
                    .map(|x| PackageDependencyMetadata::new(x.pkg_name.clone(), x.rstr.clone()))
                    .collect(),
                building: package
                    .make_deps
                    .iter()
                    .map(|x| PackageDependencyMetadata::new(x.pkg_name.clone(), x.rstr.clone()))
                    .collect(),
            },
        };

        packages.insert(package.name.clone(), pkg);

        for provided in package.provides {
            provides_reverse_table.insert(provided, package.name.clone());
        }
    }

    Repository {
        name: database.name,
        packages,
        provides_reverse_table,
    }
}

pub fn download(database: &str, mirrors: &Vec<String>) -> Option<Repository> {
    let database = match OriginalDatabase::download(database, &mirrors) {
        Some(db) => db,
        None => return None,
    };

    let converted = convert(database);

    Some(converted)
}
