use stremio_core::types::resource::Video as CoreVideo;

#[derive(Debug, Clone, PartialEq)]
pub struct Video {
    pub id: String,
    pub season: Option<u32>,
    pub episode: Option<u32>,
    pub name: String,
    pub description: String,
    pub image: Option<String>,
}

impl From<&CoreVideo> for Video {
    fn from(video: &CoreVideo) -> Self {
        let series_info = video
            .series_info
            .as_ref()
            .map(|series_info| (series_info.season, series_info.episode));

        Self {
            id: video.id.to_owned(),
            season: series_info.map(|(season, _)| season),
            episode: series_info.map(|(_, episode)| episode),
            name: video.title.to_owned(),
            description: video.overview.to_owned().unwrap_or_default(),
            image: video.thumbnail.to_owned(),
        }
    }
}
