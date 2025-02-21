use std::{path::Path, sync::RwLock};

use chrono::{DateTime, Utc};
use futures::{future, Future, FutureExt, TryFutureExt};
use http::Request;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use tracing::debug;

use stremio_core::{
    models::{ctx::Ctx, streaming_server::StreamingServer},
    runtime::{Env, EnvFuture, EnvFutureExt, TryEnvFuture},
};

use crate::{fetch::Fetch, storage::Storage};

const INSTALLATION_ID_STORAGE_KEY: &str = "installation_id";

lazy_static! {
    static ref CONCURRENT_RUNTIME: RwLock<tokio::runtime::Runtime> = RwLock::new(
        tokio::runtime::Builder::new_multi_thread()
            .thread_name("CONCURRENT_RUNTIME_THREAD")
            .worker_threads(30)
            .enable_all()
            .build()
            .expect("CONCURRENT_RUNTIME create failed")
    );
    static ref SEQUENTIAL_RUNTIME: RwLock<tokio::runtime::Runtime> = RwLock::new(
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .thread_name("SEQUENTIAL_RUNTIME_THREAD")
            .enable_all()
            .build()
            .expect("SEQUENTIAL_RUNTIME create failed")
    );
    static ref STORAGE: RwLock<Option<Storage>> = Default::default();
    static ref FETCH: RwLock<Option<Fetch>> = Default::default();
}

pub enum LosangeEnv {}

impl LosangeEnv {
    pub fn init(data_location: &Path) -> TryEnvFuture<()> {
        *STORAGE.write().expect("STORAGE write failed") =
            Some(Storage::new(data_location).expect("Create Storage failed"));

        *FETCH.write().expect("STORAGE write failed") =
            Some(Fetch::new(data_location).expect("Create Fetch failed"));

        LosangeEnv::migrate_storage_schema()
            .inspect(|migration_result| debug!("Migration result: {migration_result:?}",))
            .and_then(|_| LosangeEnv::get_storage::<String>(INSTALLATION_ID_STORAGE_KEY))
            .inspect(|installation_id_result| debug!("Migration: {installation_id_result:?}"))
            .map_ok(|installation_id| {
                installation_id.or_else(|| Some(hex::encode(LosangeEnv::random_buffer(10))))
            })
            .and_then(|installation_id| {
                LosangeEnv::set_storage(INSTALLATION_ID_STORAGE_KEY, installation_id.as_ref())
            })
            .boxed_env()
    }

    pub fn random_buffer(len: usize) -> Vec<u8> {
        let mut buffer = vec![0u8; len];
        getrandom::getrandom(buffer.as_mut_slice()).expect("generate random buffer failed");
        buffer
    }
}

impl Env for LosangeEnv {
    fn fetch<IN: Serialize + Send + 'static, OUT: for<'de> Deserialize<'de> + Send + 'static>(
        request: Request<IN>,
    ) -> TryEnvFuture<OUT> {
        let fetch = FETCH.read().expect("FETCH read failed");
        let fetch = fetch.as_ref().expect("FETCH not initialized");
        fetch.fetch(request)
    }

    fn get_storage<T: for<'de> Deserialize<'de> + Send + 'static>(
        key: &str,
    ) -> TryEnvFuture<Option<T>> {
        let storage = STORAGE.read().expect("STORAGE read failed");
        let storage = storage.as_ref().expect("STORAGE not initialized");
        storage.get(key)
    }

    fn set_storage<T: Serialize>(key: &str, value: Option<&T>) -> TryEnvFuture<()> {
        let storage = STORAGE.read().expect("STORAGE read failed");
        let storage = storage.as_ref().expect("STORAGE not initialized");
        storage.set(key, value)
    }

    fn exec_concurrent<F: Future<Output = ()> + Send + 'static>(future: F) {
        CONCURRENT_RUNTIME
            .read()
            .expect("CONCURRENT_RUNTIME read failed")
            .spawn(future);
    }

    fn exec_sequential<F: Future<Output = ()> + Send + 'static>(future: F) {
        SEQUENTIAL_RUNTIME
            .read()
            .expect("SEQUENTIAL_RUNTIME read failed")
            .spawn(future);
    }

    fn now() -> DateTime<Utc> {
        Utc::now()
    }

    #[cfg(debug_assertions)]
    fn log(message: String) {
        debug!("{}", message);
    }

    fn flush_analytics() -> EnvFuture<'static, ()> {
        future::ready(()).boxed_env()
    }

    fn analytics_context(
        _ctx: &Ctx,
        _streaming_server: &StreamingServer,
        _path: &str,
    ) -> serde_json::Value {
        serde_json::Value::Null
    }
}
