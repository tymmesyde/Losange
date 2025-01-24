use itertools::Itertools;
use relm4::SharedState;
use stremio_core::{
    models::{
        addon_details::{AddonDetails, Selected},
        installed_addons_with_filters::InstalledAddonsWithFilters,
    },
    runtime::msg::{Action, ActionCtx, ActionLoad},
    types::addon::Descriptor,
};
use url::Url;

use crate::{core::dispatch, model::LosangeModelField, types::addon::Addon};

#[derive(Default)]
pub struct AddonsState {
    pub descriptor: Option<Descriptor>,
    pub addon: Option<Addon>,
}

pub static ADDON_DETAILS_STATE: SharedState<AddonsState> = SharedState::new();

pub fn update(installed_addons: &InstalledAddonsWithFilters, addon_details: &AddonDetails) {
    let mut state = ADDON_DETAILS_STATE.write();

    let installed = installed_addons.catalog.iter().collect_vec();

    let descriptor = addon_details
        .remote_addon
        .as_ref()
        .and_then(|descriptor| descriptor.content.ready());

    let addon = descriptor.map(Addon::from).map(|mut addon| {
        if installed
            .iter()
            .any(|descriptor| descriptor.transport_url == addon.manifest_url)
        {
            addon.installed = true;
        }

        addon
    });

    state.descriptor = descriptor.cloned();
    state.addon = addon;
}

pub fn load(transport_url: &Url) {
    dispatch(
        Action::Load(ActionLoad::AddonDetails(Selected {
            transport_url: transport_url.to_owned(),
        })),
        None,
    );
}

pub fn install() {
    let state = ADDON_DETAILS_STATE.read_inner();
    if let Some(descriptor) = &state.descriptor {
        dispatch(
            Action::Ctx(ActionCtx::InstallAddon(descriptor.to_owned())),
            Some(LosangeModelField::Ctx),
        );
    }
}

pub fn uninstall() {
    let state = ADDON_DETAILS_STATE.read_inner();
    if let Some(descriptor) = &state.descriptor {
        dispatch(
            Action::Ctx(ActionCtx::UninstallAddon(descriptor.to_owned())),
            Some(LosangeModelField::Ctx),
        );
    }
}
