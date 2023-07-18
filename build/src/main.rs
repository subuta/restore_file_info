mod cache;
mod git;
mod cli;
mod cargo;

use std::collections::HashMap;
use eyre::{Result, ContextCompat};
use std::path::PathBuf;
use clap::Parser;
use dagger_sdk::HostDirectoryOpts;
use crate::cache::{DirCache, DirCacheOps};
use crate::cargo::get_package_version;
use crate::cli::Cli;
use crate::cli::ParseCli;
use crate::git::{ls_ignored_paths};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let client = dagger_sdk::connect().await?;

    let root_pb = PathBuf::from("../").canonicalize()?;
    let root_path = root_pb.to_str().context("path")?;

    let dir_cache = DirCache {
        path: "cache",
        client: &client,
        dirs: HashMap::from([("target", "/app/target"), ("registry", "/root/.cargo/registry")])
    };

    // Init cache dir.
    dir_cache.init()?;

    // Fetch git ignored files.
    let ignored_files_result = ls_ignored_paths(root_path)?;
    let ignored_files: Vec<&str> = ignored_files_result.iter().map(std::ops::Deref::deref).collect();

    let host_source_dir = client.host().directory_opts(
        root_path,
        HostDirectoryOpts {
            exclude: Some(ignored_files),
            include: None,
        },
    );

    let mut builder = client
        .container()
        .from("messense/cargo-zigbuild:0.16.12")
        .with_exec(vec!["bash", "-c", "apt-get update && apt-get install -y jq"])
        .with_mounted_directory("/app", host_source_dir.id().await?)
        .with_workdir("/app");

    builder = dir_cache.restore(builder).await?;
    let target = cli.get_target();

    let result = builder
        .with_env_variable("CARGO_HOME", "/root/.cargo")
        .with_exec(vec!["cargo", "zigbuild", "--release", "--target", &target]);

    // Export artifacts.
    result.file(&format!("/app/target/{}/release/restore_file_info", target))
        .export(&format!("./bin/{}/restore_file_info", target)).await?;

    let version = get_package_version(builder).await?;

    result.with_exec(vec!["bash", "-c", &format!("echo '{}' > /app/version.txt", version)])
        .file("/app/version.txt")
        .export(&format!("./bin/{}/version.txt", target)).await?;

    // Dump cache into host filesystem.
    dir_cache.dump(result).await?;

    Ok(())
}