use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use duct::cmd;
use filetime::FileTime;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{BufReader, Read};
use std::os::unix::prelude::PermissionsExt;
use std::path::Path;

// SEE: [How to get current platform end of line character sequence in Rust? - Stack Overflow](https://stackoverflow.com/a/47541878/9998350)
#[allow(dead_code)]
#[cfg(windows)]
pub const EOL: &'static str = "\r\n";
#[allow(dead_code)]
#[cfg(not(windows))]
pub const EOL: &'static str = "\n";

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub file: String,
    pub mtime_seconds: i64,
    pub mode: u32,
    pub hash: String,
}

pub type FileInfoList = Vec<FileInfo>;

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
}

#[derive(Debug, Args)]
pub struct Dump {
    #[arg(short, long = "gi", action)]
    /// Ignore git-ignore-ed files.
    gitignore: bool,
}

// SEE: [(Option to) Fingerprint by file contents instead of mtime · Issue #6529 · rust-lang/cargo](https://github.com/rust-lang/cargo/issues/6529)
fn file_content_hash(path: &str) -> Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut content = Vec::new();
    reader.read_to_end(&mut content)?;

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);

    Ok(hasher.finish().to_string())
}

fn ls_files(use_gitignore: bool) -> Result<FileInfoList> {
    let stdout: String;
    let files: Vec<&str>;
    if use_gitignore {
        // Fetch list of files with "gitignore".
        stdout = cmd!("git", "ls-files").read()?;
        files = stdout.split(EOL).collect();
    } else {
        // Fetch list of files
        stdout = cmd!("find", ".", "-type", "f").read()?;
        files = stdout
            .split(EOL)
            .map(|file| file.strip_prefix("./").unwrap_or(file))
            .collect();
    }

    let mut list: FileInfoList = vec![];
    for file in files.clone() {
        let metadata = fs::metadata(file)?;
        let mtime = FileTime::from_last_modification_time(&metadata);
        let mode = metadata.permissions().mode();

        list.push(FileInfo {
            file: file.to_string(),
            mtime_seconds: mtime.seconds(),
            mode,
            hash: file_content_hash(file)?,
        })
    }

    Ok(list)
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

fn touch_mtime(file: &str, unix_seconds: &str) -> Result<()> {
    if cfg!(windows) {
        panic!("No support for windows currently.");
    } else if cfg!(unix) {
        if cfg!(target_os = "macos") {
            // For macOS.
            let timestamp = cmd!("date", "-r", unix_seconds, "+%Y%m%d%H%M.%S").read()?;
            cmd!("touch", "-t", timestamp.trim(), file).run()?;
        } else {
            // For Linux.
            cmd!("touch", "-d", format!("@{}", unix_seconds), file).run()?;
        }
    }

    Ok(())
}

fn chmod(file: &str, mode: u32) -> Result<()> {
    fs::set_permissions(&file, fs::Permissions::from_mode(mode))?;
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Dump(args)) => dump_file_info_csv(args)?,
        Some(Commands::Restore) => restore_file_info_csv()?,
        None => restore_file_info_csv()?,
    }

    Ok(())
}
