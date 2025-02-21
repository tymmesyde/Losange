use std::{
    path::Path,
    sync::{Arc, RwLock},
};

use futures::{future, try_join, StreamExt};
use lazy_static::lazy_static;

use stremio_core::{
    constants::{
        DISMISSED_EVENTS_STORAGE_KEY, LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY,
        NOTIFICATIONS_STORAGE_KEY, PROFILE_STORAGE_KEY, SEARCH_HISTORY_STORAGE_KEY,
        STREAMING_SERVER_URLS_STORAGE_KEY, STREAMS_STORAGE_KEY,
    },
    models::common::Loadable,
    runtime::{
        msg::{Action, Event},
        Env, EnvError, Runtime, RuntimeAction, RuntimeEvent,
    },
    types::{
        events::DismissedEventsBucket, library::LibraryBucket, notifications::NotificationsBucket,
        profile::Profile, search_history::SearchHistoryBucket, server_urls::ServerUrlsBucket,
        streams::StreamsBucket,
    },
};
use tracing::debug;

use crate::{
    emitter::Emitter,
    env::LosangeEnv,
    model::{LosangeModel, LosangeModelField},
};

lazy_static! {
    pub static ref EVENTS: Arc<Emitter<Event>> = Arc::new(Default::default());
}

lazy_static! {
    static ref RUNTIME: RwLock<Option<Loadable<Runtime<LosangeEnv, LosangeModel>, EnvError>>> =
        Default::default();
}

pub async fn initialize(data_location: &Path) {
    if RUNTIME.read().expect("runtime read failed").is_some() {
        panic!("runtime initialization has already started");
    };

    *RUNTIME.write().expect("runtime write failed") = Some(Loadable::Loading);

    let env_init_result = LosangeEnv::init(data_location).await;
    match env_init_result {
        Ok(_) => {
            let storage_result = try_join!(
                LosangeEnv::get_storage::<Profile>(PROFILE_STORAGE_KEY),
                LosangeEnv::get_storage::<LibraryBucket>(LIBRARY_RECENT_STORAGE_KEY),
                LosangeEnv::get_storage::<LibraryBucket>(LIBRARY_STORAGE_KEY),
                LosangeEnv::get_storage::<StreamsBucket>(STREAMS_STORAGE_KEY),
                LosangeEnv::get_storage::<ServerUrlsBucket>(STREAMING_SERVER_URLS_STORAGE_KEY),
                LosangeEnv::get_storage::<NotificationsBucket>(NOTIFICATIONS_STORAGE_KEY),
                LosangeEnv::get_storage::<SearchHistoryBucket>(SEARCH_HISTORY_STORAGE_KEY),
                LosangeEnv::get_storage::<DismissedEventsBucket>(DISMISSED_EVENTS_STORAGE_KEY),
            );

            match storage_result {
                Ok((
                    profile,
                    recent_bucket,
                    other_bucket,
                    streams_bucket,
                    server_urls_bucket,
                    notifications_bucket,
                    search_history_bucket,
                    dismissed_events_bucket,
                )) => {
                    let profile = profile.unwrap_or_default();
                    let mut library = LibraryBucket::new(profile.uid(), vec![]);
                    if let Some(recent_bucket) = recent_bucket {
                        library.merge_bucket(recent_bucket);
                    };
                    if let Some(other_bucket) = other_bucket {
                        library.merge_bucket(other_bucket);
                    };

                    let streams_bucket =
                        streams_bucket.unwrap_or(StreamsBucket::new(profile.uid()));

                    let server_urls_bucket =
                        server_urls_bucket
                            .unwrap_or(ServerUrlsBucket::new::<LosangeEnv>(profile.uid()));

                    let notifications_bucket = notifications_bucket.unwrap_or(
                        NotificationsBucket::new::<LosangeEnv>(profile.uid(), vec![]),
                    );

                    let search_history_bucket =
                        search_history_bucket.unwrap_or(SearchHistoryBucket::new(profile.uid()));

                    let dismissed_events_bucket = dismissed_events_bucket
                        .unwrap_or(DismissedEventsBucket::new(profile.uid()));

                    let (model, effects) = LosangeModel::new(
                        profile,
                        library,
                        streams_bucket,
                        server_urls_bucket,
                        notifications_bucket,
                        search_history_bucket,
                        dismissed_events_bucket,
                    );

                    let (runtime, rx) = Runtime::<LosangeEnv, _>::new(
                        model,
                        effects.into_iter().collect::<Vec<_>>(),
                        1000,
                    );

                    LosangeEnv::exec_concurrent(rx.for_each(move |event| {
                        match event {
                            RuntimeEvent::NewState(fields) => {
                                debug!("NewState {:?}", fields);
                                let runtime = RUNTIME.read().expect("runtime read failed");
                                let runtime = runtime
                                    .as_ref()
                                    .expect("runtime is not ready")
                                    .as_ref()
                                    .expect("runtime is not ready");

                                let model = runtime.model().expect("model read failed");
                                model.update(fields);
                            }
                            RuntimeEvent::CoreEvent(event) => {
                                debug!("CoreEvent {:?}", event);
                                EVENTS.emit(event);
                            }
                        }

                        future::ready(())
                    }));

                    *RUNTIME.write().expect("runtime write failed") =
                        Some(Loadable::Ready(runtime));
                }
                Err(error) => {
                    *RUNTIME.write().expect("runtime write failed") =
                        Some(Loadable::Err(error.to_owned()));
                }
            }
        }
        Err(error) => {
            *RUNTIME.write().expect("runtime write failed") = Some(Loadable::Err(error.to_owned()));
        }
    }
}

pub fn dispatch(action: Action, field: Option<LosangeModelField>) {
    let runtime = RUNTIME.read().expect("runtime read failed");
    let runtime = runtime
        .as_ref()
        .expect("runtime is not ready - None")
        .as_ref()
        .expect("runtime is not ready - Loading or Error");

    runtime.dispatch(RuntimeAction { action, field });
}
