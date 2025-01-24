use std::sync::Arc;
use std::time::Duration;
use std::{convert::TryFrom, path::Path};

use futures::future;
use http::{Method, Request};
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache};
use reqwest::{Body, Client};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use stremio_core::runtime::{EnvError, EnvFutureExt, TryEnvFuture};

pub struct Fetch {
    client: Arc<ClientWithMiddleware>,
}

impl Fetch {
    pub fn new(location: &str) -> Result<Self, EnvError> {
        let path = Path::new(&location)
            .join("cache")
            .into_os_string()
            .into_string()
            .map_err(|_| EnvError::Fetch("Failed to create cache path".to_owned()))?;

        let client = ClientBuilder::new(
            Client::builder()
                .use_rustls_tls()
                .pool_max_idle_per_host(10)
                .connect_timeout(Duration::from_secs(30))
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        )
        .with(Cache(HttpCache {
            mode: CacheMode::Reload,
            manager: CACacheManager { path },
            options: None,
        }))
        .build();

        Ok(Self {
            client: Arc::new(client),
        })
    }

    pub fn fetch<IN, OUT>(&self, request: Request<IN>) -> TryEnvFuture<OUT>
    where
        IN: Serialize + Send + 'static,
        OUT: for<'de> Deserialize<'de> + Send + 'static,
    {
        let (parts, body) = request.into_parts();
        let body = match serde_json::to_string(&body) {
            Ok(body) if body != "null" && parts.method != Method::GET => Body::from(body),
            Ok(_) => Body::from(vec![]),
            Err(error) => return future::err(EnvError::Serde(error.to_string())).boxed_env(),
        };

        let request = Request::from_parts(parts, body);
        let request = match reqwest::Request::try_from(request) {
            Ok(request) => request,
            Err(error) => return future::err(EnvError::Fetch(error.to_string())).boxed_env(),
        };

        let client = Arc::clone(&self.client);

        async move {
            let response = client
                .execute(request)
                .await
                .map_err(|error| EnvError::Fetch(error.to_string()))?;

            if !response.status().is_success() {
                return Err(EnvError::Fetch(format!(
                    "Unexpected HTTP status code {}",
                    response.status().as_u16(),
                )));
            }

            let body = response
                .bytes()
                .await
                .map_err(|error| EnvError::Fetch(error.to_string()))?;

            let mut deserializer = Deserializer::from_slice(&body);
            cfg_if::cfg_if! {
                if #[cfg(debug_assertions)] {
                    let result = serde_path_to_error::deserialize::<_, OUT>(&mut deserializer);
                } else {
                    let result = OUT::deserialize(&mut deserializer);
                }
            }

            result.map_err(|error| EnvError::Serde(error.to_string()))
        }
        .boxed_env()
    }
}
