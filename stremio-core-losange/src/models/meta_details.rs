use itertools::Itertools;
use relm4::SharedState;
use stremio_core::{
    constants::{META_RESOURCE_NAME, STREAM_RESOURCE_NAME},
    models::{
        common::Loadable,
        ctx::Ctx,
        meta_details::{MetaDetails, Selected},
    },
    runtime::msg::{Action, ActionCtx, ActionLoad},
    types::{addon::ResourcePath, resource::MetaItem},
};

use crate::{
    core::dispatch,
    model::LosangeModelField,
    types::{item::Item, stream::Stream, video::Video},
};

#[derive(Default)]
pub enum MetaDetailsStatus {
    #[default]
    Loading,
    Ready,
    Error,
}

#[derive(Default)]
pub struct MetaDetailsState {
    pub status: MetaDetailsStatus,
    pub meta_item: Option<MetaItem>,
    pub item: Option<Item>,
    pub videos: Vec<(u32, Vec<Video>)>,
    pub streams_loading: bool,
    pub streams: Vec<(String, Vec<Stream>)>,
    pub in_library: bool,
}

pub static META_DETAILS_STATE: SharedState<MetaDetailsState> = SharedState::new();

pub fn update(meta_details: &MetaDetails, ctx: &Ctx) {
    let resource_loadable = meta_details
        .meta_items
        .iter()
        .find(|resource| matches!(&resource.content, Some(Loadable::Ready(_))));

    let meta_item = resource_loadable
        .and_then(|resource| resource.content.as_ref())
        .and_then(|loadable| loadable.ready());

    let item = meta_item.map(Item::from);

    let videos: Vec<(u32, Vec<Video>)> = item
        .as_ref()
        .map_or(vec![], |item| {
            item.videos.iter().map(Video::from).collect_vec()
        })
        .iter()
        .chunk_by(|video| {
            video
                .series_info
                .as_ref()
                .map_or(0, |series_info| series_info.season)
        })
        .into_iter()
        .map(|(season, group)| (season, group.map(|video| video.to_owned()).collect()))
        .collect();

    let streams = resource_loadable.map_or(vec![], |resource| {
        meta_details
            .streams
            .iter()
            .filter_map(|resource| {
                resource
                    .content
                    .as_ref()
                    .map(|content| (resource.request.to_owned(), content))
            })
            .filter_map(|(stream_request, loadable)| {
                loadable.ready().map(|streams| (stream_request, streams))
            })
            .map(|(stream_request, streams)| {
                (
                    stream_request.base.to_owned(),
                    streams
                        .iter()
                        .map(|stream| Stream::new(stream, &resource.request, &stream_request))
                        .collect_vec(),
                )
            })
            .map(|(transport_url, streams)| {
                (
                    ctx.profile
                        .addons
                        .iter()
                        .find(|addon| addon.transport_url == transport_url)
                        .map(|addon| addon.manifest.name.to_owned())
                        .unwrap_or_default(),
                    streams,
                )
            })
            .collect_vec()
    });

    let streams_loading = meta_details
        .streams
        .iter()
        .any(|resource| resource.content == Some(Loadable::Loading));

    let in_library = meta_details
        .library_item
        .as_ref()
        .is_some_and(|library_item| !library_item.temp && !library_item.removed);

    let status = if streams.is_empty() {
        if streams_loading {
            MetaDetailsStatus::Loading
        } else {
            MetaDetailsStatus::Error
        }
    } else {
        MetaDetailsStatus::Ready
    };

    let mut state = META_DETAILS_STATE.write();
    state.status = status;
    state.meta_item = meta_item.cloned();
    state.item = item;
    state.videos = videos;
    state.streams = streams;
    state.in_library = in_library;
}

pub fn load(r#type: &str, id: &str, video_id: Option<&str>) {
    dispatch(
        Action::Load(ActionLoad::MetaDetails(Selected {
            meta_path: ResourcePath::without_extra(META_RESOURCE_NAME, r#type, id),
            stream_path: Some(ResourcePath::without_extra(
                STREAM_RESOURCE_NAME,
                r#type,
                video_id.unwrap_or(id),
            )),
            guess_stream: false,
        })),
        None,
    );
}

pub fn add_to_library() {
    let state = META_DETAILS_STATE.read_inner();

    if let Some(meta_item) = &state.meta_item {
        dispatch(
            Action::Ctx(ActionCtx::AddToLibrary(meta_item.preview.to_owned())),
            Some(LosangeModelField::Ctx),
        );
    }
}

pub fn remove_from_library() {
    let state = META_DETAILS_STATE.read_inner();

    if let Some(item) = &state.item {
        dispatch(
            Action::Ctx(ActionCtx::RemoveFromLibrary(item.id.to_owned())),
            Some(LosangeModelField::Ctx),
        );
    }
}
