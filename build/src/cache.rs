use std::collections::HashMap;
use std::{fs, io};
use dagger_sdk::{Container, DaggerConn};
use eyre::{Result};
use async_trait::async_trait;

pub struct DirCache<'a> {
    pub path: &'a str,
    pub client: &'a DaggerConn,
    // alias: dir
    pub dirs: HashMap<&'a str, &'a str>
}

#[async_trait]
pub trait DirCacheOps {
    // Get alias directory path
    fn alias_path (&self, alias: &str) -> String;

    // Initialize cache on host filesystem.
    fn init (&self) -> Result<()>;

    // Restore cache into host filesystem.
    async fn restore (&self, container: Container) -> Result<Container>;

    // Dump cache into host filesystem.
    async fn dump (&self, container: Container) -> Result<()>;
}

// "mkdir -p"
// SEE: [rust - How to check if a directory exists and create a new one if it doesn't? - Stack Overflow](https://stackoverflow.com/a/48053959/9998350)
pub fn mkdirp(path: &str) -> io::Result<()> {
    fs::create_dir_all(&path)?;
    Ok(())
}


#[async_trait]
impl DirCacheOps for DirCache<'_> {
    fn alias_path(&self, alias: &str) -> String {
        format!("./{}/{}", self.path, alias)
    }

    fn init(&self) -> Result<()> {
        // Create cache directory to be mounted if not exists.
        for (alias, _dir) in self.dirs.clone() {
            mkdirp(&self.alias_path(alias))?;
        }
        Ok(())
    }

    async fn restore(&self, container: Container) -> Result<Container> {
        let cache_dir = self.client.host().directory(self.path);
        // Get last work_dir.
        let work_dir = container.workdir().await?;
        let mut mounted = container;
        for (alias, dir) in self.dirs.clone() {
            // Mount cache dir
            mounted = mounted.with_mounted_directory(dir, cache_dir.directory(alias).id().await?)
                // and restore file_info
                .with_workdir(dir)
                .with_exec(vec!["restore_file_info"]);
        }
        // Restore last work_dir.
        mounted.with_workdir(work_dir);
        Ok(mounted)
    }

    async fn dump(&self, container: Container) -> Result<()> {
        // Get last work_dir.
        for (alias, dir) in self.dirs.clone() {
            container
                // Dump file_info.
                .with_workdir(dir)
                .with_exec(vec!["restore_file_info", "dump"])
                // and dump it to host with rfi csv.
                .directory(dir).export(&self.alias_path(alias)).await?;
        }
        Ok(())
    }
}
