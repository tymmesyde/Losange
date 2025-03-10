use std::ops::Range;

use itertools::Itertools;
use relm4::SharedState;
use stremio_core::{
    models::{
        catalogs_with_extra::{CatalogsWithExtra, Selected},
        continue_watching_preview::ContinueWatchingPreview,
        ctx::Ctx,
    },
    runtime::msg::{Action, ActionCatalogsWithExtra, ActionLoad},
};

use crate::{
    core::dispatch,
    model::LosangeModelField,
    types::{catalog::Catalog, item::Item},
};

#[derive(Default)]
pub struct HomeState {
    pub catalogs: Vec<Catalog>,
}

pub static HOME_STATE: SharedState<HomeState> = SharedState::new();

pub fn update(home: &CatalogsWithExtra, continue_watching: &ContinueWatchingPreview, ctx: &Ctx) {
    let mut state = HOME_STATE.write();

    let continue_watching_catalog = ctx.profile.auth.as_ref().map(|_| Catalog {
        items: continue_watching
            .items
            .iter()
            .map(|continue_watching_item| Item::from(continue_watching_item).with(&ctx.streams))
            .collect_vec(),
        ..Default::default()
    });

    let mut catalogs = home
        .catalogs
        .iter()
        .flat_map(|catalog| {
            catalog
                .iter()
                .map(|resource| Catalog::new(resource, &ctx.profile.addons))
                .collect_vec()
        })
        .collect_vec();

    if let Some(continue_watching_catalog) = continue_watching_catalog {
        catalogs.insert(0, continue_watching_catalog);
    }

    state.catalogs = catalogs;
}

pub fn load() {
    dispatch(
        Action::Load(ActionLoad::CatalogsWithExtra(Selected {
            r#type: None,
            extra: vec![],
        })),
        Some(LosangeModelField::Home),
    );
}

pub fn load_catalog(start: usize, end: usize) {
    dispatch(
        Action::CatalogsWithExtra(ActionCatalogsWithExtra::LoadRange(Range { start, end })),
        Some(LosangeModelField::Home),
    );
}

pub fn unload() {
    dispatch(Action::Unload, Some(LosangeModelField::Home));
}
