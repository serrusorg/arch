pub mod downman;
mod pkgman;
mod repoman;

use std::process::exit;

use clap::{Parser, Subcommand};
use libc::geteuid;

use crate::repoman::Database;

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

fn main() {
    if unsafe { geteuid() } != 0 {
        println!("you're not running as superuser, please try again");
        return;
    }

    let tool = Tool::parse();

    match tool.command {
        Commands::Update => {
            let mirrors = vec!["https://geo.mirror.pkgbuild.com".to_string()];

            let core = match repoman::download("core", &mirrors) {
                Some(db) => db,
                None => {
                    println!("failed to download core.db");
                    exit(-1);
                }
            };
            let extra = match repoman::download("extra", &mirrors) {
                Some(db) => db,
                None => {
                    println!("failed to download core.db");
                    exit(-1);
                }
            };

            let core_saved = core.save();
            let extra_saved = extra.save();

            match std::fs::write("/var/lib/arch/core.db", core_saved) {
                Ok(_) => {}
                Err(_) => {
                    println!("failed to write to core.db");
                    return;
                }
            }
            match std::fs::write("/var/lib/arch/extra.db", extra_saved) {
                Ok(_) => {}
                Err(_) => {
                    println!("failed to write to extra.db");
                    return;
                }
            }

            println!("updated");
        }
        Commands::Install { packages } => {
            let core_data = match std::fs::read("/var/lib/arch/core.db") {
                Ok(data) => data,
                Err(_) => {
                    println!("failed to open core.db");
                    Vec::new()
                }
            };
            let extra_data = match std::fs::read("/var/lib/arch/extra.db") {
                Ok(data) => data,
                Err(_) => {
                    println!("failed to open extra.db");
                    Vec::new()
                }
            };

            let core = Database::load(&core_data).unwrap();
            let _ = Database::load(&extra_data).unwrap();

            for package in packages {
                let matched = match core.packages.get(&package) {
                    Some(matched) => matched,
                    None => {
                        println!("cannot find package {}", package);
                        exit(-1);
                    }
                };
                println!("{} {}", matched.name, matched.version);
            }
        }
    }
}
