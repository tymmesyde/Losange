use std::{path::Path, time::Duration};

use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use relm4::{
    gtk::{gio, prelude::SettingsExt},
    once_cell::sync::OnceCell,
};
use reqwest::{Client, Response};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use url::Url;

use crate::constants::APP_ID;

static CLIENT: OnceCell<ClientWithMiddleware> = OnceCell::new();

pub async fn fetch(url: Url) -> anyhow::Result<Response> {
    let client = CLIENT.get_or_init(|| {
        let settings = gio::Settings::new(APP_ID);
        let storage_location = settings.string("storage-location");
        let path = Path::new(&storage_location).join("cache");

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
            manager: CACacheManager { path },
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
