pub mod downman;
mod pkgman;
mod repoman;

use std::cmp::Reverse;
use std::fmt;

use clap::{Parser, Subcommand};
use libc::geteuid;
use pubgrub::{
    DefaultStringReporter, Dependencies, DependencyConstraints, DependencyProvider, PubGrubError,
    Ranges, Reporter, resolve,
};
use versions::Versioning;

use crate::{pkgman::pkg::Package, repoman::Repository};

#[derive(Debug)]
pub struct ResolverError(pub String);

impl fmt::Display for ResolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ResolverError {}

#[derive(Parser)]
#[command(
    name = "arch",
    version,
    about = "A package manager for your Arch system"
)]
struct Tool {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Update,

    Install {
        #[arg(required = true, num_args = 1..)]
        packages: Vec<String>,
    },
}

pub struct DependencyResolver {
    repositories: Vec<Repository>,
}

impl DependencyResolver {
    pub fn new(repositories: Vec<Repository>) -> Self {
        Self { repositories }
    }

    fn query(&self, name: &str) -> Option<&Package> {
        if let Some(pkg) = self
            .repositories
            .iter()
            .find_map(|repo| repo.packages.get(name))
        {
            return Some(pkg);
        }

        for repo in self.repositories.iter().clone() {
            for pkgs in repo.provides_reverse_table.clone() {
                if pkgs.0.contains(name) {
                    return self.query(pkgs.1.as_str());
                } else {
                    continue;
                }
            }
        }

        None
    }

    pub fn resolve_dependencies(
        &self,
        root_package: String,
        root_version: Versioning,
    ) -> Result<Vec<(String, Versioning)>, PubGrubError<DependencyResolver>> {
        resolve(self, root_package, root_version).map(|map| map.into_iter().collect())
    }
}

impl DependencyProvider for DependencyResolver {
    type P = String;
    type V = Versioning;
    type VS = Ranges<Versioning>;
    type Priority = Reverse<Versioning>;
    type M = String;
    type Err = ResolverError;

    fn prioritize(
        &self,
        _package: &Self::P,
        _range: &Self::VS,
        _package_conflicts_counts: &pubgrub::PackageResolutionStatistics,
    ) -> Self::Priority {
        Reverse(Versioning::new("0.0.0").unwrap())
    }

    fn choose_version(
        &self,
        package: &Self::P,
        range: &Self::VS,
    ) -> Result<Option<Self::V>, Self::Err> {
        let pkg = match self.query(package) {
            Some(p) => p,
            None => return Ok(None),
        };

        let parsed_ver = Versioning::new(&pkg.version)
            .ok_or_else(|| ResolverError(format!("Invalid version string: {}", pkg.version)))?;

        if range.contains(&parsed_ver) {
            Ok(Some(parsed_ver))
        } else {
            Ok(None)
        }
    }

    fn get_dependencies(
        &self,
        package: &Self::P,
        _version: &Self::V,
    ) -> Result<Dependencies<Self::P, Self::VS, Self::M>, Self::Err> {
        let pkg = self
            .query(package)
            .ok_or_else(|| ResolverError(format!("Package not found: {}", package)))?;

        let constraints: DependencyConstraints<Self::P, Self::VS> = pkg
            .dependencies
            .runtime
            .iter()
            .map(|dep| {
                let canonical_name = self
                    .query(&dep.name)
                    .expect("cannot find virtual dependency")
                    .name
                    .to_string();

                let range = match (&dep.version, &dep.op) {
                    _ => Ranges::full(),
                };

                (canonical_name, range)
            })
            .collect();

        Ok(Dependencies::Available(constraints))
    }
}

fn main() {
    if unsafe { geteuid() } != 0 {
        eprintln!("Please run again as root")
    }

    let tool = Tool::parse();

    match tool.command {
        Commands::Update => {
            let mirrors = vec!["https://geo.mirror.pkgbuild.com".to_string()];

            let core = repoman::download("core", &mirrors)
                .expect("Unable to download core repository")
                .save();
            let extra = repoman::download("extra", &mirrors)
                .expect("Unable to download extra repository")
                .save();

            if let Err(e) = std::fs::write("/var/lib/arch/core.db", core) {
                eprintln!("Failed to save core.db: {}", e);
            }
            if let Err(e) = std::fs::write("/var/lib/arch/extra.db", extra) {
                eprintln!("Failed to save extra.db: {}", e);
            }

            println!("Repositories updated successfully.");
        }
        Commands::Install { packages } => {
            let core_data = std::fs::read("/var/lib/arch/core.db").unwrap_or_default();
            let extra_data = std::fs::read("/var/lib/arch/extra.db").unwrap_or_default();

            if core_data.is_empty() && extra_data.is_empty() {
                eprintln!("No local repository databases found. Please run update first.");
                return;
            }

            let mut repositories = Vec::new();
            if let Ok(core) = Repository::load(&core_data) {
                repositories.push(core);
            }
            if let Ok(extra) = Repository::load(&extra_data) {
                repositories.push(extra);
            }

            let resolver = DependencyResolver::new(repositories);

            for pkg_name in packages {
                match resolver.query(&pkg_name) {
                    Some(pkg) => {
                        let parsed_ver = match Versioning::new(&pkg.version) {
                            Some(v) => v,
                            None => {
                                eprintln!(
                                    "Failed to parse version '{}' for {}",
                                    pkg.version, pkg_name
                                );
                                continue;
                            }
                        };

                        match resolver.resolve_dependencies(pkg_name.clone(), parsed_ver) {
                            Ok(solution) => {
                                println!("Resolved dependencies for {}:", pkg_name);
                                for (dep, dep_ver) in solution {
                                    println!(" - {} @ {}", dep, dep_ver);
                                }
                            }
                            Err(PubGrubError::NoSolution(derivation_tree)) => {
                                let report = DefaultStringReporter::report(&derivation_tree);
                                println!("Resolution failed due to:\n{}", report);
                            }
                            Err(other_error) => {
                                eprintln!("An unexpected error occurred: {:?}", other_error);
                            }
                        }
                    }
                    None => eprintln!("Package '{}' not found in repositories.", pkg_name),
                }
            }
        }
    }
}
