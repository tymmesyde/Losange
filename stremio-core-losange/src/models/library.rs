use itertools::Itertools;
use relm4::SharedState;
use stremio_core::{
    models::library_with_filters::{
        LibraryRequest, LibraryWithFilters, NotRemovedFilter, SelectableSort, SelectableType,
        Selected,
    },
    runtime::msg::{Action, ActionLibraryWithFilters, ActionLoad},
};

use crate::{core::dispatch, model::LosangeModelField, types::item::Item};

#[derive(Default)]
pub struct LibraryState {
    pub types: Vec<SelectableType>,
    pub orders: Vec<SelectableSort>,
    pub items: Vec<Item>,
}

pub static LIBRARY_STATE: SharedState<LibraryState> = SharedState::new();

pub fn update(library: &LibraryWithFilters<NotRemovedFilter>) {
    let mut state = LIBRARY_STATE.write();

    let types = library.selectable.types.to_owned();
    let orders = library.selectable.sorts.to_owned();
    let items = library.catalog.iter().map(Item::from).collect_vec();

    state.types = types;
    state.orders = orders;
    state.items = items;
}

pub fn load(request: Option<LibraryRequest>) {
    dispatch(
        Action::Load(ActionLoad::LibraryWithFilters(Selected {
            request: request.map_or(
                LibraryRequest {
                    r#type: Default::default(),
                    sort: Default::default(),
                    page: Default::default(),
                },
                |request| request,
            ),
        })),
        None,
    );
}

pub fn load_with_type(index: usize) {
    let state = LIBRARY_STATE.read_inner();

    if let Some(selectable) = state.types.get(index) {
        load(Some(selectable.request.to_owned()));
    }
}

pub fn load_with_order(index: usize) {
    let state = LIBRARY_STATE.read_inner();

    if let Some(selectable) = state.orders.get(index) {
        load(Some(selectable.request.to_owned()));
    }
}

pub fn load_next_items() {
    dispatch(
        Action::LibraryWithFilters(ActionLibraryWithFilters::LoadNextPage),
        Some(LosangeModelField::Library),
    );
}
