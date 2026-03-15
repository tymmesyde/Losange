use relm4::SharedState;
use stremio_core::{
    models::streaming_server::StreamingServer,
    runtime::msg::{Action, ActionStreamingServer},
    types::streaming_server::StatisticsRequest,
};

use crate::{core::dispatch, model::LosangeModelField};

const KB: f64 = 1024.0;
const MB: f64 = KB * KB;

#[derive(Default)]
pub struct ServerState {
    pub online: bool,
    pub torrent_progress: Option<f64>,
}

pub static SERVER_STATE: SharedState<ServerState> = SharedState::new();

pub fn update(server: &StreamingServer) {
    let mut state = SERVER_STATE.write();

    let online = server.settings.is_ready();

    let torrent_progress = server
        .statistics
        .as_ref()
        .and_then(|statistics| statistics.ready())
        .map(|statistics| {
            let mut progress = 0.0;

            progress += (statistics.peers as f64 / 8.0).min(1.0) * 20.0;

            if statistics.stream_len > 0 {
                let mut min_download = statistics.stream_len as f64 * 0.008;
                min_download = min_download.min(8.0 * MB);
                min_download = min_download.max(2.0 * MB);
                progress += (statistics.downloaded as f64 / min_download).min(1.0) * 70.0;
            }

            progress += (statistics.download_speed / (300.0 * KB)).min(1.0) * 10.0;

            progress
        });

    state.online = online;
    state.torrent_progress = torrent_progress;
}

pub fn reload() {
    dispatch(
        Action::StreamingServer(ActionStreamingServer::Reload),
        Some(LosangeModelField::Server),
    );
}

pub fn update_statistics(info_hash: &str, file_idx: u16) {
    dispatch(
        Action::StreamingServer(ActionStreamingServer::GetStatistics(StatisticsRequest {
            info_hash: info_hash.to_owned(),
            file_idx,
        })),
        Some(LosangeModelField::Server),
    );
}
