use std::ops::Range;

use itertools::Itertools;
use relm4::SharedState;
use stremio_core::{
    models::{
        catalogs_with_extra::{CatalogsWithExtra, Selected},
        ctx::Ctx,
    },
    runtime::msg::{Action, ActionCatalogsWithExtra, ActionLoad},
    types::addon::ExtraValue,
};

use crate::{core::dispatch, model::LosangeModelField, types::catalog::Catalog};

#[derive(Default)]
pub struct SearchState {
    pub loading: bool,
    pub catalogs: Vec<Catalog>,
}

pub static SEARCH_STATE: SharedState<SearchState> = SharedState::new();

pub fn update(search: &CatalogsWithExtra, ctx: &Ctx) {
    let mut state = SEARCH_STATE.write();

    let loading = !search.catalogs.is_empty()
        && !search.catalogs.iter().any(|catalog| {
            catalog.iter().any(|resource| {
                resource
                    .content
                    .as_ref()
                    .is_some_and(|content| content.is_ready())
            })
        });

    let catalogs = search
        .catalogs
        .iter()
        .flat_map(|catalog| {
            catalog
                .iter()
                .filter(|resource| {
                    resource
                        .content
                        .as_ref()
                        .and_then(|content| content.ready())
                        .is_some()
                })
                .map(|resource| Catalog::new(resource, &ctx.profile.addons))
                .collect_vec()
        })
        .collect_vec();

    state.loading = loading;
    state.catalogs = catalogs;
}

pub fn load(query: String) {
    dispatch(
        Action::Load(ActionLoad::CatalogsWithExtra(Selected {
            r#type: None,
            extra: vec![ExtraValue {
                name: "search".to_owned(),
                value: query,
            }],
        })),
        Some(LosangeModelField::Search),
    );
}

pub fn load_catalog(start: usize, end: usize) {
    dispatch(
        Action::CatalogsWithExtra(ActionCatalogsWithExtra::LoadRange(Range { start, end })),
        Some(LosangeModelField::Search),
    );
}

pub fn unload() {
    dispatch(Action::Unload, Some(LosangeModelField::Search));
}
