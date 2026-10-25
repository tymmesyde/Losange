use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use futures_util::StreamExt;
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use relm4::{
    gtk::{gio, prelude::SettingsExt},
    once_cell::sync::OnceCell,
};
use reqwest::{Client, Response};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use shellexpand::tilde;
use tokio::{fs, io::AsyncWriteExt};
use url::Url;

use crate::constants::APP_ID;

static CLIENT: OnceCell<ClientWithMiddleware> = OnceCell::new();

pub async fn fetch(url: Url) -> anyhow::Result<Response> {
    let client = CLIENT.get_or_init(|| {
        let settings = gio::Settings::new(APP_ID);
        let storage_location = settings.string("storage-location");

        let expanded_path = tilde(&storage_location).to_string();
        let path = Path::new(&expanded_path).join("cache");

        ClientBuilder::new(
            Client::builder()
                .use_rustls_tls()
                .pool_max_idle_per_host(10)
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        )
        .with(Cache(HttpCache {
            mode: CacheMode::Default,
            manager: CACacheManager::new(path, false),
            options: HttpCacheOptions::default(),
        }))
        .build()
    });

    let response = client
        .get(url)
        .header(reqwest::header::ACCEPT_ENCODING, "gzip")
        .send()
        .await?;

    Ok(response)
}

pub async fn download(file_path: PathBuf, url: Url) -> anyhow::Result<()> {
    let tmp_file_path = file_path.with_added_extension("part");

    let response = reqwest::get(url.as_str()).await?.error_for_status()?;
    let mut stream = response.bytes_stream();

    let mut file = fs::File::create(&tmp_file_path).await?;
    while let Some(chunk) = stream.next().await {
        file.write_all(&chunk?).await?;
    }

    file.flush().await?;
    drop(file);

    fs::rename(&tmp_file_path, &file_path).await?;
    Ok(())
}
