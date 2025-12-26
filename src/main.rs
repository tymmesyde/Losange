mod app;
mod common;
mod components;
mod constants;
mod dialogs;
mod pages;
mod server;

use std::ptr;

use app::{App, AppMsg};
use clap::Parser;
use constants::APP_ID;

use gtk::prelude::ApplicationExt;
use relm4::{
    actions::{AccelsPlus, RelmAction, RelmActionGroup},
    gtk, MessageBroker, RelmApp,
};
use rust_i18n::i18n;

include!(concat!(env!("OUT_DIR"), "/icons.rs"));

i18n!("locales", fallback = "en");

relm4::new_action_group!(AppActionGroup, "app");
relm4::new_stateless_action!(QuitAction, AppActionGroup, "quit");

pub static APP_BROKER: MessageBroker<AppMsg> = MessageBroker::new();

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub open: Option<String>,
}

fn main() {
    tracing_subscriber::fmt::init();

    relm4_icons::initialize_icons(GRESOURCE_BYTES, RESOURCE_PREFIX);
    relm4::set_global_css(include_str!("style.css"));

    let language = gtk::default_language();
    rust_i18n::set_locale(&language.to_str());

    let library = unsafe { libloading::os::unix::Library::new("libepoxy.so.0") }
        .expect("Failed to load libepoxy");

    epoxy::load_with(|name| {
        unsafe { library.get::<_>(name.as_bytes()) }
            .map(|symbol| *symbol)
            .unwrap_or(ptr::null())
    });

    let app = relm4::main_application();
    let mut actions = RelmActionGroup::<AppActionGroup>::new();

    let quit_action = {
        let app = app.to_owned();
        RelmAction::<QuitAction>::new_stateless(move |_| {
            app.quit();
        })
    };

    actions.add_action(quit_action);
    actions.register_for_main_application();

    app.set_accelerators_for_action::<QuitAction>(&["<Control>q"]);

    let args = Args::parse();

    let app = RelmApp::new(APP_ID);
    app.with_args(vec![])
        .with_broker(&APP_BROKER)
        .run_async::<App>(args);
}
