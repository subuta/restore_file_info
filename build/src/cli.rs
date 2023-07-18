use std::env::consts::{ARCH};
use std::fmt;
use clap::{Parser, ValueEnum};

// SEE: [How to get current platform end of line character sequence in Rust? - Stack Overflow](https://stackoverflow.com/a/47541878/9998350)
#[allow(dead_code)]
#[cfg(windows)]
pub const EOL: &'static str = "\r\n";
#[allow(dead_code)]
#[cfg(not(windows))]
pub const EOL: &'static str = "\n";

#[derive(ValueEnum, Debug, Clone)]
enum Target {
    #[value(name="x64_linux")]
    X64,
    #[value(name="arm64_linux")]
    Arm64,
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Target::X64 => write!(f, "x86_64-unknown-linux-musl"),
            Target::Arm64 => write!(f, "aarch64-unknown-linux-musl"),
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "build_restore_file_info", author, version, about, long_about = None)]
pub struct Cli {
    #[clap(value_enum)]
    /// Rust build target
    target: Option<Target>,
}

pub trait ParseCli {
    fn get_default_target(&self) -> String;
    fn get_target(&self) -> String;
}

impl ParseCli for Cli {
    fn get_default_target(&self) -> String {
        format!("{}-unknown-linux-musl", get_arch())
    }
    fn get_target(&self) -> String {
        // Defaults to "linux-musl" of current architecture.
        let mut target = self.get_default_target();
        if let Some(_target) = &self.target {
            target = _target.to_string()
        }
        target
    }
}

pub fn get_arch() -> String {
    let mut arch = "x86_64";
    if ARCH == "aarch64" || ARCH == "arm" {
        arch = "aarch64";
    }
    arch.to_string()
}