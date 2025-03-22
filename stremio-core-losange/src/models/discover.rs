use itertools::Itertools;
use relm4::SharedState;
use stremio_core::{
    models::catalog_with_filters::{
        CatalogWithFilters, SelectableCatalog, SelectableExtraOption, SelectableType, Selected,
    },
    runtime::msg::{Action, ActionCatalogWithFilters, ActionLoad},
    types::{addon::ResourceRequest, resource::MetaItemPreview},
};

use crate::{core::dispatch, model::LosangeModelField, types::item::Item};

#[derive(Default)]
pub struct DiscoverState {
    pub types: Vec<SelectableType>,
    pub catalogs: Vec<SelectableCatalog>,
    pub genres: Vec<SelectableExtraOption>,
    pub items: Vec<Item>,
}

pub static DISCOVER_STATE: SharedState<DiscoverState> = SharedState::new();

pub fn update(discover: &CatalogWithFilters<MetaItemPreview>) {
    let mut state = DISCOVER_STATE.write();

    let types = discover.selectable.types.to_owned();
    let catalogs = discover.selectable.catalogs.to_owned();
    let genres = discover
        .selectable
        .extra
        .iter()
        .find(|extra| extra.name == "genre")
        .map_or(vec![], |genre| genre.options.to_owned());

    let items = discover
        .catalog
        .iter()
        .flat_map(|resource| resource.content.as_ref())
        .flat_map(|loadable| loadable.ready())
        .flatten()
        .unique_by(|item| &item.id)
        .map(Item::from)
        .collect_vec();

    state.types = types;
    state.catalogs = catalogs;
    state.genres = genres;
    state.items = items;
}

pub fn load(request: Option<ResourceRequest>) {
    let selected = request.map(|request| Selected { request });

    dispatch(Action::Load(ActionLoad::CatalogWithFilters(selected)), None);
}

pub fn load_with_type(index: usize) {
    let state = DISCOVER_STATE.read_inner();

    if let Some(selectable) = state.types.get(index) {
        load(Some(selectable.request.to_owned()));
    }
}

pub fn load_with_catalog(index: usize) {
    let state = DISCOVER_STATE.read_inner();

    if let Some(selectable) = state.catalogs.get(index) {
        load(Some(selectable.request.to_owned()));
    }
}

pub fn load_with_genre(index: usize) {
    let state = DISCOVER_STATE.read_inner();

    if let Some(selectable) = state.genres.get(index) {
        load(Some(selectable.request.to_owned()));
    }
}

pub fn load_next_items() {
    dispatch(
        Action::CatalogWithFilters(ActionCatalogWithFilters::LoadNextPage),
        Some(LosangeModelField::Discover),
    );
}
