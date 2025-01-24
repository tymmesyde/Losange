use relm4::SharedState;
use stremio_core::{
    models::streaming_server::StreamingServer,
    runtime::msg::{Action, ActionStreamingServer},
};

use crate::{core::dispatch, model::LosangeModelField};

#[derive(Default)]
pub struct ServerState {
    pub online: bool,
}

pub static SERVER_STATE: SharedState<ServerState> = SharedState::new();

pub fn update(server: &StreamingServer) {
    let mut state = SERVER_STATE.write();

    let online = server.settings.is_ready();

    state.online = online;
}

pub fn reload() {
    dispatch(
        Action::StreamingServer(ActionStreamingServer::Reload),
        Some(LosangeModelField::Server),
    );
}
