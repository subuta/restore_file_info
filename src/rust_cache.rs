use std::env::{current_dir, set_current_dir};
use std::fs;
use std::fs::DirEntry;
use std::path::{PathBuf};
use std::str::FromStr;
use std::time::{SystemTime};
use anyhow::{Context, Result};
use filetime::FileTime;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use duct::cmd;

// Based on "rust-cache" implementation.
// SEE: [rust-cache/src/cleanup.ts at v2.5.1 · Swatinem/rust-cache](https://github.com/Swatinem/rust-cache/blob/v2.5.1/src/cleanup.ts)

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MetaTarget {
    kind: Vec<String>,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MetaPackage {
    name: String,
    version: String,
    manifest_path: String,
    targets: Vec<MetaTarget>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    packages: Vec<MetaPackage>
}

#[derive(Debug, Clone)]
pub struct PackageDefinition {
    name: String,
    version: String,
    #[allow(dead_code)]
    path: String,
    targets: Vec<String>,
}

pub type Packages = Vec<PackageDefinition>;

#[async_trait]
pub trait GetPackages {
    fn get_packages(&self, root: &str) -> Result<Packages>;
}

fn dir_name(path: &str) -> Result<String> {
    let mut path = PathBuf::from_str(path)?;
    path.pop();
    Ok(path.to_string_lossy().into_owned())
}

impl GetPackages for Meta {
    fn get_packages(&self, root: &str) -> Result<Packages> {
        let mut packages: Packages = vec![];

        let save_targets = vec!["lib".to_string(), "proc-macro".to_string()];

        for pkg in &self.packages {
            if pkg.manifest_path.starts_with(root) {
                continue;
            }
            let targets = pkg.targets.clone().into_iter().filter(|t| {
               t.kind.clone().into_iter().any(|kind| {
                   save_targets.contains(&kind)
               })
            }).map(|t| t.name).collect::<Vec<_>>();

            let _pkg = pkg.clone();

            packages.push(PackageDefinition {
                name: _pkg.name,
                version: _pkg.version,
                path: dir_name(&_pkg.manifest_path)?,
                targets,
            });
        }

        Ok(packages)
    }
}

pub fn clean_target_dir(target_dir: &str, packages: Packages, check_time_stamp: bool) -> Result<()> {
    let dir = fs::read_dir(target_dir)?;
    for _dirent in dir {
        let _packages = packages.clone();
        let dirent = _dirent?;
        let dirname = dirent.path().to_string_lossy().into_owned();
        let metadata = dirent.metadata()?;
        if metadata.is_dir() {
            let cachedir_tag_exists = dirent.path().join("CACHEDIR.TAG").exists();
            let rustc_info_exists = dirent.path().join(".rustc_info.json").exists();
            let is_nested_target = cachedir_tag_exists || rustc_info_exists;
            if is_nested_target {
                clean_target_dir(&dirname, _packages, check_time_stamp)?;
            } else {
                clean_profile_target(&dirname, _packages, check_time_stamp)?;
            }
        } else if dirent.file_name() != "CACHEDIR.TAG" {
            rm(dirent)?;
        }
    }
    Ok(())
}

pub fn clean_profile_target(profile_dir: &str, packages: Packages, check_time_stamp: bool) -> Result<()> {
    let keep_profile = vec!["build".to_string(), ".fingerprint".to_string(), "deps".to_string()];
    rm_except(profile_dir, keep_profile, false)?;

    let keep_pkg = packages.clone().into_iter().map(|p| p.name).collect::<Vec<_>>();
    rm_except(&format!("{}/build", profile_dir), keep_pkg.clone(), check_time_stamp)?;
    rm_except(&format!("{}/.fingerprint", profile_dir), keep_pkg.clone(), check_time_stamp)?;

    let keep_deps_nested = packages.into_iter().map(|p| {
        let mut names: Vec<String> = vec![];
        let mut targets = p.targets.clone();
        targets.push(p.name);
        for n in targets {
            let name = n.replace("-", "_");
            names.push(name.clone());
            names.push(format!("lib{}", name));
        }
        return names
    }).collect::<Vec<_>>();
    let keep_deps = keep_deps_nested.into_iter().flatten().collect::<Vec<_>>();

    rm_except(&format!("{}/deps", profile_dir), keep_deps, check_time_stamp)?;

    Ok(())
}

pub fn clean_registry(registry_dir: &str, packages: Packages, crates: bool) -> Result<()> {
    fs::remove_dir_all(format!("{}/src", registry_dir))?;

    let index_dir = fs::read_dir(format!("{}/index", registry_dir))?;
    for _dirent in index_dir {
        let dirent = _dirent?;
        let metadata = dirent.metadata()?;
        if metadata.is_dir() {
            // eg `.cargo/registry/index/github.com-1ecc6299db9ec823`
            // or `.cargo/registry/index/index.crates.io-e139d0d48fed7772`

            // for a git registry, we can remove `.cache`, as cargo will recreate it from git
            if dirent.path().join(".git").exists() {
                let path = dirent.path().to_string_lossy().into_owned();
                fs::remove_dir_all(format!("{}/.cache", path))?;
            }
        }
    }

    if !crates {
        return Ok(())
    }

    let pkg_set = packages.into_iter().map(|p| format!("{}-{}.crate", p.name, p.version)).collect::<Vec<_>>();
    let cache_dir = fs::read_dir(format!("{}/cache", registry_dir))?;
    for _dirent in cache_dir {
        let dirent = _dirent?;
        let metadata = dirent.metadata()?;
        if metadata.is_dir() {
            let dir = fs::read_dir(dirent.path())?;
            for _dirent in dir {
                let dirent = _dirent?;
                let metadata = dirent.metadata()?;
                let name = dirent.file_name().to_string_lossy().into_owned();
                // here we check that the downloaded `.crate` matches one from our dependencies
                if metadata.is_file() && !pkg_set.contains(&name) {
                    rm(dirent)?;
                }
            }
        }
    }

    // `.cargo/registry/cache`

    Ok(())
}

fn is_outdated(dirent: &DirEntry) -> Result<bool> {
    // Get mtime of dirent.
    let file_time = FileTime::from_last_modification_time(&dirent.metadata()?);

    // Get now.
    let now = SystemTime::now();

    // Get elapsed
    let elapsed_duration = now.duration_since(SystemTime::UNIX_EPOCH)?.as_secs() - file_time.seconds() as u64;

    // One week in seconds.
    let one_week_secs = 7 * 24 * 60 * 60;

    // Check if outdated(One week elapsed since last modification).
    Ok(elapsed_duration >= one_week_secs)
}

fn rm_except(dir_name: &str, keep_prefix: Vec<String>, check_time_stamp: bool) -> Result<()> {
    let dir = fs::read_dir(dir_name);
    // Process only if dir exists.
    if let Ok(_dir) = dir {
        for _dirent in dir {
            let dirent = _dirent?;
            if check_time_stamp {
                if is_outdated(&dirent)? {
                    rm(dirent)?;
                }
                continue;
            }

            let mut name = dirent.file_name().to_string_lossy().into_owned();

            // strip the trailing hash
            let _idx = name.rfind("-");
            if _idx.is_some() {
                let idx = _idx.context("idx")?;
                name = name[0..idx].parse()?;
            }

            if !keep_prefix.contains(&name) {
                rm(dirent)?;
            }
        }
    }
    Ok(())
}

pub fn rm(dirent: DirEntry) -> Result<()> {
    let path = dirent.path();
    return if dirent.metadata()?.is_dir() {
        Ok(fs::remove_dir_all(path)?)
    } else {
        Ok(fs::remove_file(path)?)
    }
}

pub fn get_packages(root_dir: &str) -> Result<Packages> {
    let cd = current_dir()?;

    assert!(set_current_dir(root_dir).is_ok());

    let metadata_result = cmd!("cargo", "metadata", "--all-features", "--format-version", "1").read()?;
    let metadata: Meta = serde_json::from_str(&metadata_result)?;

    let root = &cd.to_str().context("cd")?;
    let packages = metadata.get_packages(root.clone())?;

    assert!(set_current_dir(cd.clone()).is_ok());

    Ok(packages)
}