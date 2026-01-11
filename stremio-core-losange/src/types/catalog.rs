use itertools::Itertools;
use stremio_core::{
    constants::OFFICIAL_ADDONS,
    models::common::ResourceLoadable,
    types::{
        addon::{Descriptor, ResourceRequest},
        resource::MetaItemPreview,
    },
};
use url::Url;

use super::item::Item;

const STREMIO_LOGO: &str = "https://stremio.com/website/stremio-logo-small.png";

#[derive(Debug, Clone, Default)]
pub struct Catalog {
    pub request: Option<ResourceRequest>,
    pub icon: Option<Url>,
    pub addon_name: Option<String>,
    pub name: Option<String>,
    pub r#type: Option<String>,
    pub loading: bool,
    pub items: Vec<Item>,
}

impl Catalog {
    pub fn new(resource: &ResourceLoadable<Vec<MetaItemPreview>>, addons: &[Descriptor]) -> Self {
        let addon = addons
            .iter()
            .find(|descriptor| descriptor.transport_url == resource.request.base);

        let logo = addon.and_then(|addon| {
            addon.manifest.logo.to_owned().or_else(|| {
                OFFICIAL_ADDONS
                    .iter()
                    .map(|official| &official.manifest.id)
                    .find(|&id| id == &addon.manifest.id)
                    .and_then(|_| Url::parse(STREMIO_LOGO).ok())
            })
        });

        let addon_name = addon.map(|addon| addon.manifest.name.to_owned());

        let name = addon
            .and_then(|addon| {
                addon.manifest.catalogs.iter().find(|catalog| {
                    catalog.id == resource.request.path.id
                        && catalog.r#type == resource.request.path.r#type
                })
            })
            .and_then(|catalog| catalog.name.to_owned())
            .or(addon.map(|descriptor| descriptor.manifest.name.to_owned()));

        Self {
            request: Some(resource.request.clone()),
            icon: logo,
            addon_name,
            name,
            r#type: Some(resource.request.path.r#type.to_owned()),
            loading: resource
                .content
                .as_ref()
                .is_some_and(|content| content.is_loading()),
            items: resource
                .content
                .as_ref()
                .and_then(|content| content.ready())
                .map_or(vec![], |meta_items| {
                    meta_items.iter().map(Item::from).collect_vec()
                }),
        }
    }
}
