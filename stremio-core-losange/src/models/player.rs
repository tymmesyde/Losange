use relm4::SharedState;
use stremio_core::constants::SUBTITLES_RESOURCE_NAME;
use stremio_core::{
    models::{
        ctx::Ctx,
        player::{Player, Selected, VideoParams},
    },
    runtime::msg::{Action, ActionLoad, ActionPlayer},
    types::{
        addon::ResourcePath,
        resource::{StreamSource, Subtitles},
        streams::StreamItemState,
    },
};
use url::Url;

use crate::{
    core::dispatch,
    model::LosangeModelField,
    types::{item::Item, stream::Stream},
};

#[derive(Debug, Clone, PartialEq)]
pub struct ExternalSubtitle {
    pub url: Url,
    pub lang: String,
    pub title: String,
}

#[derive(Default)]
pub struct PlayerState {
    pub subtitles: Vec<ExternalSubtitle>,
    pub uri: Option<Url>,
    pub title: String,
    pub image: Option<Url>,
    pub time: f64,
    pub next_stream: Option<Stream>,
    pub stream_state: Option<StreamItemState>,
    pub torrent_info: Option<(String, u16)>,
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

    let image = item.as_ref().and_then(|item| item.image.to_owned());

    let time = player
        .library_item
        .as_ref()
        .map_or(0.0, |library_item| library_item.state.time_offset as f64);

    let next_stream = player
        .selected
        .as_ref()
        .zip(player.next_streams.as_ref())
        .and_then(|(selected, next_streams)| {
            next_streams
                .content
                .as_ref()
                .and_then(|content| content.ready())
                .zip(Some(&next_streams.request))
                .and_then(|(streams, request)| {
                    streams
                        .iter()
                        .find(|next_stream| next_stream.is_binge_match(&selected.stream))
                        .cloned()
                        .zip(Some(request))
                })
                .zip(selected.meta_request.as_ref())
                .map(|((next_stream, stream_request), meta_request)| {
                    Stream::new(&next_stream, meta_request, stream_request)
                })
        });

    let stream_state = player.stream_state.to_owned();

    let embedded_subtitles = player.selected.as_ref().into_iter().flat_map(|selected| {
        selected
            .stream
            .subtitles
            .iter()
            .map(|subtitle| external_subtitle(subtitle, "Stream".to_owned()))
    });

    let addon_subtitles = player.subtitles.iter().flat_map(|loadable| {
        let title = ctx
            .profile
            .addons
            .iter()
            .find(|addon| addon.transport_url == loadable.request.base)
            .map(|addon| addon.manifest.name.to_owned())
            .unwrap_or_else(|| "Addon".to_owned());

        loadable
            .content
            .as_ref()
            .and_then(|content| content.ready())
            .into_iter()
            .flatten()
            .map(move |subtitle| external_subtitle(subtitle, title.to_owned()))
    });

    let subtitles = embedded_subtitles.chain(addon_subtitles).collect();

    let torrent_info = player
        .selected
        .as_ref()
        .and_then(|selected| match selected.stream.source {
            StreamSource::Torrent {
                info_hash,
                file_idx: Some(file_idx),
                ..
            } => Some((hex::encode(info_hash), file_idx)),
            _ => None,
        });

    state.subtitles = subtitles;
    state.uri = uri;
    state.title = title;
    state.image = image;
    state.time = time;
    state.next_stream = next_stream;
    state.stream_state = stream_state;
    state.torrent_info = torrent_info;
}

fn external_subtitle(subtitle: &Subtitles, title: String) -> ExternalSubtitle {
    ExternalSubtitle {
        url: subtitle.url.to_owned(),
        lang: subtitle.lang.to_owned(),
        title,
    }
}

pub fn load(stream: Stream) {
    let subtitles_path = ResourcePath::without_extra(
        SUBTITLES_RESOURCE_NAME,
        &stream.stream_request.path.r#type,
        &stream.stream_request.path.id,
    );
    let filename = stream.behavior_hints.filename.to_owned();

    dispatch(
        Action::Load(ActionLoad::Player(Box::new(Selected {
            stream: stream.to_owned().into(),
            stream_request: Some(stream.stream_request),
            meta_request: Some(stream.meta_request),
            subtitles_path: Some(subtitles_path),
        }))),
        None,
    );

    // Addon subtitles are only requested once the video params are known
    dispatch(
        Action::Player(ActionPlayer::VideoParamsChanged {
            video_params: Some(VideoParams {
                hash: None,
                size: None,
                filename,
            }),
        }),
        Some(LosangeModelField::Player),
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

pub fn update_stream_state<T: FnOnce(StreamItemState) -> StreamItemState>(update: T) {
    let state = PLAYER_STATE.read_inner();
    let stream_state = match &state.stream_state {
        Some(state) => state.to_owned(),
        None => StreamItemState::default(),
    };
    let updated_stream_state = update(stream_state);

    dispatch(
        Action::Player(ActionPlayer::StreamStateChanged {
            state: updated_stream_state,
        }),
        Some(LosangeModelField::Player),
    );
}
