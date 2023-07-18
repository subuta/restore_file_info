use std::env::{current_dir, set_current_dir};
use std::process::Command;
use eyre::{Result};
use crate::cli::EOL;

pub fn ls_ignored_paths (dir: &str) -> Result<Vec<String>> {
    let cd = current_dir()?;

    // pushd dir
    assert!(set_current_dir(dir).is_ok());

    let result = Command::new("git")
        .arg("ls-files")
        .arg("--ignored")
        .arg("--exclude-standard")
        .arg("--others")
        .arg("--directory")
        .arg("--no-empty-directory")
        .output()?;

    let stdout =  String::from_utf8_lossy(&*result.stdout);

    // popd
    assert!(set_current_dir(cd).is_ok());

    Ok(stdout.split(EOL).map(|path| path.to_string()).collect())
}