use relm4::SharedState;
use stremio_core::{
    models::ctx::Ctx,
    runtime::msg::{Action, ActionCtx},
    types::{
        api::AuthRequest,
        profile::{Auth, Settings},
    },
};
use url::Url;

use crate::{core::dispatch, model::LosangeModelField};

#[derive(Default)]
pub struct CtxState {
    pub auth: Option<Auth>,
    pub settings: Settings,
}

pub static CTX_STATE: SharedState<CtxState> = SharedState::new();

pub fn update(ctx: &Ctx) {
    let mut state = CTX_STATE.write();

    let auth = ctx.profile.auth.to_owned();
    let settings = ctx.profile.settings.to_owned();

    state.auth = auth;
    state.settings = settings;
}

pub fn sync_with_api() {
    dispatch(
        Action::Ctx(ActionCtx::PullUserFromAPI),
        Some(LosangeModelField::Ctx),
    );
    dispatch(
        Action::Ctx(ActionCtx::PullAddonsFromAPI),
        Some(LosangeModelField::Ctx),
    );
    dispatch(
        Action::Ctx(ActionCtx::SyncLibraryWithAPI),
        Some(LosangeModelField::Ctx),
    );
    dispatch(
        Action::Ctx(ActionCtx::PullNotifications),
        Some(LosangeModelField::Ctx),
    );
}

pub fn login(email: String, password: String) {
    dispatch(
        Action::Ctx(ActionCtx::Authenticate(AuthRequest::Login {
            email,
            password,
            facebook: false,
        })),
        Some(LosangeModelField::Ctx),
    );
}

pub fn logout() {
    dispatch(Action::Ctx(ActionCtx::Logout), Some(LosangeModelField::Ctx));
}

pub fn update_server_url(url: String) {
    let state = CTX_STATE.read_inner();
    let mut settings = state.settings.to_owned();
    settings.streaming_server_url = Url::parse(&url).expect("Failed to parse server url");

    dispatch(
        Action::Ctx(ActionCtx::UpdateSettings(settings)),
        Some(LosangeModelField::Ctx),
    );
}
