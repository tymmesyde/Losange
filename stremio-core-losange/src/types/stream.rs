use stremio_core::types::{
    addon::ResourceRequest,
    resource::{Stream as CoreStream, StreamBehaviorHints, StreamSource, Subtitles},
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
