use chrono::Datelike;
use itertools::Itertools;
use stremio_core::{
    models::continue_watching_preview::Item as ContinueWatchingItem,
    types::{
        library::LibraryItem,
        resource::{Link, MetaItem, MetaItemPreview, PosterShape, Video},
        streams::{StreamsBucket, StreamsItemKey},
    },
};
use url::Url;

use super::stream::Stream;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Item {
    pub id: String,
    pub r#type: String,
    pub logo: Option<Url>,
    pub name: String,
    pub description: Option<String>,
    pub released: Option<String>,
    pub runtime: Option<String>,
    pub imdb: Option<String>,
    pub genres: Vec<String>,
    pub directors: Vec<String>,
    pub writers: Vec<String>,
    pub actors: Vec<String>,
    pub image: Option<Url>,
    pub shape: PosterShape,
    pub videos: Vec<Video>,
    pub new_videos: usize,
    pub progress: f64,
    pub last_video_id: Option<String>,
    pub last_stream: Option<Stream>,
}

impl Item {
    pub fn with(mut self, streams: &StreamsBucket) -> Self {
        let last_stream = self
            .last_video_id
            .as_ref()
            .map(|video_id| StreamsItemKey {
                meta_id: self.id.to_owned(),
                video_id: video_id.to_owned(),
            })
            .and_then(|key| streams.items.get(&key))
            .map(Stream::from);

        self.last_stream = last_stream;
        self
    }
}

impl From<&MetaItemPreview> for Item {
    fn from(meta_item: &MetaItemPreview) -> Self {
        Self {
            id: meta_item.id.to_owned(),
            r#type: meta_item.r#type.to_owned(),
            logo: meta_item.logo.to_owned(),
            name: meta_item.name.to_owned(),
            description: meta_item.description.to_owned(),
            released: meta_item.released.map(|date| date.year().to_string()),
            runtime: meta_item.runtime.to_owned(),
            imdb: get_link(&meta_item.links, "imdb"),
            genres: get_links(&meta_item.links, "Genres"),
            directors: get_links(&meta_item.links, "Directors"),
            writers: get_links(&meta_item.links, "Writers"),
            actors: get_links(&meta_item.links, "Cast"),
            image: meta_item.poster.to_owned(),
            shape: meta_item.poster_shape.to_owned(),
            ..Default::default()
        }
    }
}

impl From<&MetaItem> for Item {
    fn from(meta_item: &MetaItem) -> Self {
        let mut item = Item::from(&meta_item.preview);
        item.videos = meta_item.videos.to_owned();

        item
    }
}

impl From<&LibraryItem> for Item {
    fn from(library_item: &LibraryItem) -> Self {
        Self {
            id: library_item.id.to_owned(),
            r#type: library_item.r#type.to_owned(),
            name: library_item.name.to_owned(),
            image: library_item.poster.to_owned(),
            shape: library_item.poster_shape.to_owned(),
            ..Default::default()
        }
    }
}

impl From<&ContinueWatchingItem> for Item {
    fn from(continue_watching_item: &ContinueWatchingItem) -> Self {
        let mut item = Item::from(&continue_watching_item.library_item);
        item.new_videos = continue_watching_item.notifications;
        item.progress = continue_watching_item.library_item.progress();
        item.last_video_id = continue_watching_item
            .library_item
            .state
            .video_id
            .to_owned();

        item
    }
}

fn get_link(links: &[Link], category: &str) -> Option<String> {
    links
        .iter()
        .find(|link| link.category == category)
        .map(|link| link.name.to_owned())
}

fn get_links(links: &[Link], category: &str) -> Vec<String> {
    links
        .iter()
        .filter(|link| link.category == category)
        .map(|link| link.name.to_owned())
        .collect_vec()
}
