use std::{
    fs,
    path::Path,
    process::{Child, Command},
};

use anyhow::Context;
use serde::Deserialize;
use url::Url;

use crate::constants::{SERVER_DOWNLOAD_ENDPOINT, SERVER_UPDATER_ENDPOINT};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ServerUpdaterResponse {
    latest_version: String,
}

pub async fn initialize(storage_location: &str) -> anyhow::Result<Child> {
    let file_path = Path::new(&storage_location).join("server.js");
    let version_path = Path::new(&storage_location).join("server_version");

    let latest_version = reqwest::get(SERVER_UPDATER_ENDPOINT)
        .await?
        .json::<ServerUpdaterResponse>()
        .await?
        .latest_version;

    let should_download = fs::read_to_string(&version_path)
        .map_or(true, |current_version| current_version != latest_version);

    if should_download {
        let download_url = Url::parse(
            SERVER_DOWNLOAD_ENDPOINT
                .replace("VERSION", &latest_version)
                .as_str(),
        )?;

        let latest_file = reqwest::get(download_url)
            .await?
            .bytes()
            .await
            .context("Failed to fetch server file")?;

        fs::write(&file_path, latest_file).context("Failed to write server file")?;
        fs::write(&version_path, &latest_version).context("Failed to write version file")?;
    }

    let process = Command::new("node")
        .arg(file_path.as_os_str())
        .spawn()
        .context("Failed to start server")?;

    Ok(process)
}
