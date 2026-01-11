use std::{
    fs,
    path::Path,
    process::{Child, Command},
};

use anyhow::Context;
use url::Url;

use crate::constants::{SERVER_DOWNLOAD_ENDPOINT, SERVER_VERSION};

pub async fn initialize(data_location: &Path) -> anyhow::Result<Child> {
    let file_path = Path::new(&data_location).join("server.js");
    let version_path = Path::new(&data_location).join("server_version");

    let should_download = fs::read_to_string(&version_path)
        .map_or(true, |current_version| current_version != SERVER_VERSION);

    if should_download {
        let download_url = Url::parse(
            SERVER_DOWNLOAD_ENDPOINT
                .replace("VERSION", SERVER_VERSION)
                .as_str(),
        )?;

        let latest_file = reqwest::get(download_url)
            .await?
            .bytes()
            .await
            .context("Failed to fetch server file")?;

        fs::write(&file_path, latest_file).context("Failed to write server file")?;
        fs::write(&version_path, SERVER_VERSION).context("Failed to write version file")?;
    }

    let process = Command::new("node")
        .arg(file_path.as_os_str())
        .spawn()
        .context("Failed to start server")?;

    Ok(process)
}
