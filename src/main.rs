mod rust_cache;
mod fs_util;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use std::env::{current_dir};
use std::fs;
use std::path::{Path, PathBuf};
use rust_cache::{clean_registry, clean_target_dir, get_packages};
use crate::fs_util::{chmod, file_content_hash, FileInfo, ls_files, touch_mtime};

#[derive(Debug, Parser)]
#[command(name = "restore_file_info", author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Dump file_info csv
    Dump(Dump),
    /// Restore file_info csv
    Restore,
    /// Clean cargo target_dir
    #[clap(name="cargo_clean_target_dir")]
    CargoCleanTargetDir(CargoCleanTargetDir),
    /// Clean cargo registry
    #[clap(name="cargo_clean_registry")]
    CargoCleanRegistry(CargoCleanRegistry),
}

#[derive(Debug, Args)]
pub struct CargoCleanTargetDir {
    #[arg(short, long = "target-dir")]
    /// Target dir path of "cargo build", defaults to "$(pwd)/target"
    target_dir: Option<String>,
}

#[derive(Debug, Args)]
pub struct CargoCleanRegistry {
    #[arg(short, long = "registry-dir")]
    /// Cargo registry dir, defaults to "/root/.cargo/registry"
    registry_dir: Option<String>,
}

#[derive(Debug, Args)]
pub struct Dump {
    #[arg(short, long = "gi", action)]
    /// Ignore git-ignore-ed files.
    gitignore: bool,
}

fn dump_file_info_csv(args: &Dump) -> Result<()> {
    let files = ls_files(args.gitignore)?;
    let path = String::from("restore_file_info.csv");

    let mut writer = csv::Writer::from_path(path)?;
    for file in files {
        writer.serialize(file)?;
    }

    writer.flush()?;

    Ok(())
}

fn restore_file_info_csv() -> Result<()> {
    let path = String::from("restore_file_info.csv");

    if !Path::new(&path).exists() {
        println!("'restore_file_info.csv' not found, no action");
        return Ok(());
    }

    let mut reader = csv::Reader::from_path(path)?;
    for result in reader.deserialize() {
        let info: FileInfo = result?;
        let content_hash = file_content_hash(&info.file)?;
        // Apply modification only if content-hash is matched.
        if info.hash == content_hash {
            touch_mtime(&info.file, &info.mtime_seconds.to_string())?;
            chmod(&info.file, info.mode)?;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Dump(args)) => dump_file_info_csv(args)?,
        Some(Commands::Restore) => restore_file_info_csv()?,
        Some(Commands::CargoCleanTargetDir(args)) => {
            let cd = current_dir()?;
            let root = &cd.to_str().context("cd")?;

            let mut target = format!("{}/target", root.clone());
            if args.target_dir.is_some() {
                let target_dir = args.target_dir.clone().context("target_dir")?;
                target = fs::canonicalize(&PathBuf::from(target_dir))?.to_string_lossy().into_owned();
            }

            let packages = get_packages(root)?;
            clean_target_dir(&target, packages, false)?;
        },
        Some(Commands::CargoCleanRegistry(args)) => {
            let cd = current_dir()?;
            let root = &cd.to_str().context("cd")?;
            let packages = get_packages(root)?;

            let mut registry = "/root/.cargo/registry".to_string();
            if args.registry_dir.is_some() {
                let registry_dir = args.registry_dir.clone().context("registry_dir")?;
                registry = fs::canonicalize(&registry_dir)?.to_string_lossy().into_owned();
            }

            clean_registry(&registry, packages, false)?;
        },
        None => restore_file_info_csv()?,
    }

    Ok(())
}
