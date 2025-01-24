use relm4::SharedState;
use stremio_core::{
    models::{
        ctx::Ctx,
        player::{Player, Selected},
    },
    runtime::msg::{Action, ActionLoad, ActionPlayer},
};
use url::Url;

use crate::{
    core::dispatch,
    model::LosangeModelField,
    types::{item::Item, stream::Stream},
};

#[derive(Default)]
pub struct PlayerState {
    pub uri: Option<Url>,
    pub title: String,
    pub time: f64,
}

pub static PLAYER_STATE: SharedState<PlayerState> = SharedState::new();

pub fn update(player: &Player, ctx: &Ctx) {
    let mut state = PLAYER_STATE.write();

    let uri = player.selected.as_ref().and_then(|selected| {
        selected
            .stream
            .streaming_url(Some(&ctx.profile.settings.streaming_server_url))
    });

    let item = player
        .meta_item
        .as_ref()
        .and_then(|meta_item| meta_item.content.as_ref())
        .and_then(|loadable| loadable.ready())
        .map(Item::from);

    let video = player
        .meta_item
        .as_ref()
        .and_then(|meta_item| meta_item.content.as_ref())
        .and_then(|loadable| loadable.ready())
        .and_then(|meta_item| {
            meta_item
                .videos
                .iter()
                .find(|video| video.series_info == player.series_info)
        });

    let title = item
        .as_ref()
        .map(|item| {
            video
                .and_then(|video| {
                    player.series_info.as_ref().map(|series| {
                        format!(
                            "{} - {} ({}x{})",
                            item.name, video.title, series.season, series.episode,
                        )
                    })
                })
                .unwrap_or(item.name.to_owned())
        })
        .unwrap_or_default();

    let time = player
        .library_item
        .as_ref()
        .map_or(0.0, |library_item| library_item.state.time_offset as f64);

    state.uri = uri;
    state.title = title;
    state.time = time;
}

pub fn load(stream: Stream) {
    dispatch(
        Action::Load(ActionLoad::Player(Box::new(Selected {
            stream: stream.to_owned().into(),
            stream_request: Some(stream.stream_request),
            meta_request: Some(stream.meta_request),
            subtitles_path: None,
        }))),
        None,
    );
}

pub fn unload() {
    dispatch(Action::Unload, Some(LosangeModelField::Player));
}

pub fn update_paused(paused: bool) {
    dispatch(
        Action::Player(ActionPlayer::PausedChanged { paused }),
        Some(LosangeModelField::Player),
    );
}

pub fn update_time(time: f64, duration: f64) {
    dispatch(
        Action::Player(ActionPlayer::TimeChanged {
            time: time as u64,
            duration: duration as u64,
            device: "linux".to_owned(),
        }),
        Some(LosangeModelField::Player),
    );
}

pub fn update_seek_time(time: f64, duration: f64) {
    dispatch(
        Action::Player(ActionPlayer::Seek {
            time: time as u64,
            duration: duration as u64,
            device: "linux".to_owned(),
        }),
        Some(LosangeModelField::Player),
    );
}
