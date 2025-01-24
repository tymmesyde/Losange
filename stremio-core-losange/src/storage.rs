use std::{fs, ops::Deref, path::PathBuf, sync::Arc};

use futures::future;
use redb::{Database, TableDefinition};
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use stremio_core::runtime::{EnvError, EnvFutureExt, TryEnvFuture};

const TABLE: TableDefinition<&str, Option<String>> = TableDefinition::new("stremio");

pub struct Storage {
    db: Arc<Database>,
}

impl Storage {
    pub fn new(location: &str) -> Result<Self, EnvError> {
        fs::create_dir_all(location).map_err(|_| EnvError::StorageUnavailable)?;
        let path = PathBuf::from(location).join("stremio.redb");
        let db = Database::create(path).map_err(|_| EnvError::StorageUnavailable)?;

        let write_txn = db.begin_write().map_err(|_| EnvError::StorageUnavailable)?;

        {
            write_txn
                .open_table(TABLE)
                .map_err(|_| EnvError::StorageUnavailable)?;
        }

        write_txn
            .commit()
            .map_err(|_| EnvError::StorageUnavailable)?;

        Ok(Self { db: Arc::new(db) })
    }

    pub fn get<T: for<'de> Deserialize<'de> + Send + 'static>(
        &self,
        key: &str,
    ) -> TryEnvFuture<Option<T>> {
        let db = Arc::clone(&self.db);
        let key = key.to_owned();

        future::lazy(move |_| {
            let table = db
                .begin_read()
                .map_err(|e| EnvError::StorageReadError(e.to_string()))?
                .open_table(TABLE)
                .map_err(|e| EnvError::StorageReadError(e.to_string()))?;

            let value = table
                .get(key.deref())
                .map_err(|e| EnvError::StorageReadError(e.to_string()))?
                .and_then(|guard| guard.value());

            if let Some(value) = value {
                let mut deserializer = Deserializer::from_str(&value);
                let value = T::deserialize(&mut deserializer)
                    .map_err(|e| EnvError::Serde(e.to_string()))?;

                return Ok(Some(value));
            }

            Ok(None)
        })
        .boxed_env()
    }

    pub fn set<T: Serialize>(&self, key: &str, value: Option<&T>) -> TryEnvFuture<()> {
        let db = Arc::clone(&self.db);
        let key = key.to_owned();

        let value = value.map(|value| serde_json::to_string(&value).unwrap());

        future::lazy(move |_| {
            let write_txn = db
                .begin_write()
                .map_err(|e| EnvError::StorageWriteError(e.to_string()))?;

            {
                let mut table = write_txn
                    .open_table(TABLE)
                    .map_err(|e| EnvError::StorageWriteError(e.to_string()))?;

                table
                    .insert(key.deref(), value)
                    .map_err(|e| EnvError::StorageWriteError(e.to_string()))?;
            }

            write_txn
                .commit()
                .map_err(|e| EnvError::StorageWriteError(e.to_string()))?;

            Ok(())
        })
        .boxed_env()
    }
}
