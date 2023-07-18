use dagger_sdk::Container;
use crate::cli::get_arch;

pub fn install_rfi(container: Container) -> Container {
    container
        .with_exec(vec!["bash", "-c", "apt-get update && apt-get install curl -y"])
        .with_exec(vec!["bash", "-c", &format!("curl -L {} --output '/usr/local/bin/restore_file_info' && chmod +x /usr/local/bin/restore_file_info", get_download_url())])
}

fn get_download_url() -> String {
    format!("https://github.com/subuta/restore_file_info/releases/download/v0.1.0/{}", get_file_name())
}

fn get_file_name() -> String {
    let arch = get_arch();
    if &arch == "x86_64" {
        return "restore_file_info_x64".to_string();
    } else if &arch == "aarch64" {
        return "restore_file_info_arm64".to_string();
    }
    "restore_file_info_arm64".to_string()
}