use itertools::Itertools;
use relm4::SharedState;
use stremio_core::{
    models::{
        catalog_with_filters::{CatalogWithFilters, Selected},
        common::Loadable,
    },
    runtime::msg::{Action, ActionCtx, ActionLoad},
    types::addon::{Descriptor, Manifest, ResourcePath, ResourceRequest},
};
use url::Url;

use crate::{core::dispatch, model::LosangeModelField, types::addon::Addon};

#[derive(Default)]
pub struct RemoteAddonsState {
    pub loading: bool,
    pub addons: Vec<Addon>,
}

pub static REMOTE_ADDONS_STATE: SharedState<RemoteAddonsState> = SharedState::new();

pub fn update(remote_addons: &CatalogWithFilters<Descriptor>) {
    let mut state = REMOTE_ADDONS_STATE.write();

    let loading = remote_addons
        .catalog
        .iter()
        .all(|loadable| loadable.content == Some(Loadable::Loading));

    let addons = remote_addons
        .catalog
        .iter()
        .filter_map(|resource| resource.content.as_ref())
        .filter_map(|loadable| loadable.ready())
        .flatten()
        .map(Addon::from)
        .collect_vec();

    state.loading = loading;
    state.addons = addons;
}

pub fn load(manifest_url: &'static str) {
    if let Ok(transport_url) = Url::parse(manifest_url) {
        dispatch(
            Action::Load(ActionLoad::CatalogWithFilters(Some(Selected {
                request: ResourceRequest {
                    base: transport_url,
                    path: ResourcePath::without_extra("addon_catalog", "all", "community"),
                },
            }))),
            None,
        );
    }
}

pub fn unload() {
    dispatch(Action::Unload, Some(LosangeModelField::RemoteAddons));
}

pub async fn install(transport_url: Url) -> Result<(), reqwest::Error> {
    let response = reqwest::get(transport_url.clone()).await?;
    let manifest = response.json::<Manifest>().await?;

    let descriptor = Descriptor {
        transport_url,
        manifest,
        flags: Default::default(),
    };

    dispatch(
        Action::Ctx(ActionCtx::InstallAddon(descriptor.to_owned())),
        Some(LosangeModelField::Ctx),
    );

    Ok(())
}
