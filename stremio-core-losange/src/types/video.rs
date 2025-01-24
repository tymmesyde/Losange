use stremio_core::types::resource::{SeriesInfo, Video as CoreVideo};

#[derive(Debug, Clone, PartialEq)]
pub struct Video {
    pub id: String,
    pub name: String,
    pub description: String,
    pub image: Option<String>,
    pub series_info: Option<SeriesInfo>,
}

impl From<&CoreVideo> for Video {
    fn from(video: &CoreVideo) -> Self {
        Self {
            id: video.id.to_owned(),
            name: video.title.to_owned(),
            description: video.overview.to_owned().unwrap_or_default(),
            image: video.thumbnail.to_owned(),
            series_info: video.series_info.to_owned(),
        }
    }
}
