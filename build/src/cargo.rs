use dagger_sdk::Container;
use crate::cli::EOL;
use eyre::{Result};

pub async fn get_package_version(container: Container) -> Result<String> {
    let version_result = container
        .with_exec(vec!["bash", "-c", &format!("cargo metadata --format-version=1 --no-deps | jq -r '.packages[0].version'")]).stdout().await?;
    let version = version_result.strip_suffix(EOL).unwrap_or(&version_result);
    Ok(format!("v{}", version))
}