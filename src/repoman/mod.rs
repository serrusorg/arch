mod odb;

use std::collections::BTreeMap;

use bitcode::{Decode, Encode, decode, encode};
pub use odb::OriginalDatabase;
use repodb_parser::pkg::{Op, Rstr};

#[derive(Decode, Encode, PartialEq, Eq)]
pub enum PackageDependencyMetadataVersion {
    Eq,
    Ge,
    Gt,
    Le,
    Lt,
}

impl From<Op> for PackageDependencyMetadataVersion {
    fn from(op: Op) -> Self {
        match op {
            Op::Eq => PackageDependencyMetadataVersion::Eq,
            Op::Ge => PackageDependencyMetadataVersion::Ge,
            Op::Gt => PackageDependencyMetadataVersion::Gt,
            Op::Le => PackageDependencyMetadataVersion::Le,
            Op::Lt => PackageDependencyMetadataVersion::Lt,
        }
    }
}

#[derive(Decode, Encode, PartialEq, Eq)]
pub struct PackageDependencyMetadata {
    name: String,
    version: Option<String>,
    op: Option<PackageDependencyMetadataVersion>,
}

impl PackageDependencyMetadata {
    pub fn new(name: String, version: Option<Rstr>) -> Self {
        let (version, op) = if let Some(version) = version {
            (
                Some(version.version),
                Some(PackageDependencyMetadataVersion::from(version.op)),
            )
        } else {
            (None, None)
        };

        PackageDependencyMetadata { name, version, op }
    }
}

#[derive(Decode, Encode, PartialEq, Eq)]
pub struct PackageDependency {
    runtime: Vec<PackageDependencyMetadata>,
    optional: Vec<PackageDependencyMetadata>,
    testing: Vec<PackageDependencyMetadata>,
    building: Vec<PackageDependencyMetadata>,
}

#[derive(Decode, Encode, PartialEq, Eq)]
pub struct Package {
    pub name: String,
    pub file: String,

    pub version: String,

    pub license: Vec<String>,

    replaces: Vec<String>,
    conflicts: Vec<String>,
    provides: Vec<String>,
    dependencies: PackageDependency,
}

#[derive(Decode, Encode, PartialEq, Eq)]
pub struct Database {
    pub name: String,
    pub packages: BTreeMap<String, Package>,
}

impl Database {
    pub fn load(data: &[u8]) -> Result<Self, &str> {
        decode(data).map_err(|_| "decoding error")
    }

    pub fn save(&self) -> Vec<u8> {
        encode(self)
    }
}

pub fn convert(database: OriginalDatabase) -> Database {
    let mut packages = BTreeMap::new();
    for package in database.pkgs {
        let pkg = Package {
            name: package.name.clone(),
            file: package.file_name,

            version: package.version,

            license: package.license,

            replaces: package.replaces,
            provides: package.provides,
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

        packages.insert(package.name, pkg);
    }

    Database {
        name: database.name,
        packages,
    }
}

pub fn download(database: &str, mirrors: &Vec<String>) -> Option<Database> {
    let database = match OriginalDatabase::download(database, &mirrors) {
        Some(db) => db,
        None => return None,
    };

    let converted = convert(database);

    Some(converted)
}
