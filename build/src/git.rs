use std::env::{current_dir, set_current_dir};
use std::process::Command;
use eyre::{Result};

// SEE: [How to get current platform end of line character sequence in Rust? - Stack Overflow](https://stackoverflow.com/a/47541878/9998350)
#[allow(dead_code)]
#[cfg(windows)]
pub const EOL: &'static str = "\r\n";
#[allow(dead_code)]
#[cfg(not(windows))]
pub const EOL: &'static str = "\n";

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