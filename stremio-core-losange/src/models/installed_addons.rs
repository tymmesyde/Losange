use itertools::Itertools;
use relm4::SharedState;
use stremio_core::{
    models::installed_addons_with_filters::{
        InstalledAddonsRequest, InstalledAddonsWithFilters, Selected,
    },
    runtime::msg::{Action, ActionLoad},
};

use crate::{core::dispatch, model::LosangeModelField, types::addon::Addon};

#[derive(Default)]
pub struct InstalledAddons {
    pub addons: Vec<Addon>,
}

pub static INSTALLED_ADDONS_STATE: SharedState<InstalledAddons> = SharedState::new();

pub fn update(installed_addons: &InstalledAddonsWithFilters) {
    let mut state = INSTALLED_ADDONS_STATE.write();

    let addons = installed_addons
        .catalog
        .iter()
        .map(Addon::from)
        .collect_vec();

    state.addons = addons;
}

pub fn load() {
    dispatch(
        Action::Load(ActionLoad::InstalledAddonsWithFilters(Selected {
            request: InstalledAddonsRequest { r#type: None },
        })),
        None,
    );
}

pub fn unload() {
    dispatch(Action::Unload, Some(LosangeModelField::InstalledAddons));
}
