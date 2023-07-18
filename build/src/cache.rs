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
        let mut mounted = container;
        for (alias, dir) in self.dirs.clone() {
            mounted = mounted.with_mounted_directory(dir, cache_dir.directory(alias).id().await?)
        }
        Ok(mounted)
    }

    async fn dump(&self, container: Container) -> Result<()> {
        for (alias, dir) in self.dirs.clone() {
            container.directory(dir).export(&self.alias_path(alias)).await?;
        }
        Ok(())
    }
}
