use bitcode::{Decode, Encode};
use repodb_parser::pkg::{Op, Rstr};

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
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

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct PackageDependencyMetadata {
    pub name: String,
    pub version: Option<String>,
    pub op: Option<PackageDependencyMetadataVersion>,
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

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct PackageDependency {
    pub runtime: Vec<PackageDependencyMetadata>,
    pub optional: Vec<PackageDependencyMetadata>,
    pub testing: Vec<PackageDependencyMetadata>,
    pub building: Vec<PackageDependencyMetadata>,
}

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct Package {
    pub name: String,
    pub file: String,

    pub version: String,

    pub license: Vec<String>,

    pub replaces: Vec<String>,
    pub conflicts: Vec<String>,
    pub provides: Vec<String>,
    pub dependencies: PackageDependency,
}
