use stremio_core::{
    constants::{META_RESOURCE_NAME, STREAM_RESOURCE_NAME},
    types::{
        addon::{ResourcePath, ResourceRequest},
        resource::{Stream as CoreStream, StreamBehaviorHints, StreamSource, Subtitles},
        streams::StreamsItem,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct Stream {
    pub name: String,
    pub description: String,
    pub source: StreamSource,
    pub subtitles: Vec<Subtitles>,
    pub meta_request: ResourceRequest,
    pub stream_request: ResourceRequest,
}

impl Stream {
    pub fn new(
        stream: &CoreStream,
        meta_request: &ResourceRequest,
        stream_request: &ResourceRequest,
    ) -> Self {
        Self {
            name: stream.name.to_owned().unwrap_or_default(),
            description: stream.description.to_owned().unwrap_or_default(),
            source: stream.source.to_owned(),
            subtitles: stream.subtitles.to_owned(),
            meta_request: meta_request.to_owned(),
            stream_request: stream_request.to_owned(),
        }
    }
}

impl From<&StreamsItem> for Stream {
    fn from(streams_item: &StreamsItem) -> Self {
        let meta_request = ResourceRequest::new(
            streams_item.meta_transport_url.to_owned(),
            ResourcePath::without_extra(
                META_RESOURCE_NAME,
                &streams_item.r#type,
                &streams_item.meta_id,
            ),
        );

        let stream_request = ResourceRequest::new(
            streams_item.stream_transport_url.to_owned(),
            ResourcePath::without_extra(
                STREAM_RESOURCE_NAME,
                &streams_item.r#type,
                &streams_item.video_id,
            ),
        );

        Stream::new(&streams_item.stream, &meta_request, &stream_request)
    }
}

impl From<Stream> for CoreStream {
    fn from(stream: Stream) -> Self {
        Self {
            source: stream.source,
            name: None,
            description: None,
            subtitles: stream.subtitles,
            thumbnail: None,
            behavior_hints: StreamBehaviorHints::default(),
        }
    }
}
