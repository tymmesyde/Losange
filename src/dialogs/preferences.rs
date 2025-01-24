use adw::prelude::*;
use gtk::gio;
use relm4::{adw, gtk, Component, ComponentParts, ComponentSender};
use rust_i18n::t;
use stremio_core_losange::models::{self, ctx::CTX_STATE, server::SERVER_STATE};

use crate::constants::APP_ID;

#[derive(Debug)]
pub enum PreferencesDialogInput {
    Open,
    Update,
    CatalogsIconChanged(bool),
    CatalogsAddonNameChanged(bool),
    ContentTitlesBelowChanged(bool),
    DetailsContentColorsChanged(bool),
    DetailsContentLogoChanged(bool),
    ServerUrlChanged(String),
    ServerEnabledChanged(bool),
    StorageLocationChanged(String),
}

pub struct PreferencesDialog {
    settings: gio::Settings,
}

#[relm4::component(pub)]
impl Component for PreferencesDialog {
    type Init = ();
    type Input = PreferencesDialogInput;
    type Output = ();
    type CommandOutput = ();

    view! {
        adw::PreferencesDialog {
            set_title: &t!("preferences"),

            add = &adw::PreferencesPage {
                set_margin_bottom: 26,

                add = &adw::PreferencesGroup {
                    set_title: &t!("appereance"),

                    adw::SwitchRow {
                        set_title: &t!("catalog_addon_icon"),
                        set_active: model.settings.boolean("catalog-addon-icon"),
                        connect_active_notify[sender] => move |row| {
                            let value = row.is_active();
                            sender.input(PreferencesDialogInput::CatalogsIconChanged(value));
                        }
                    },
                    adw::SwitchRow {
                        set_title: &t!("catalog_addon_name"),
                        set_active: model.settings.boolean("catalog-addon-name"),
                        connect_active_notify[sender] => move |row| {
                            let value = row.is_active();
                            sender.input(PreferencesDialogInput::CatalogsAddonNameChanged(value));
                        }
                    },
                    adw::SwitchRow {
                        set_title: &t!("content_title_below"),
                        set_active: model.settings.boolean("content-title-below"),
                        connect_active_notify[sender] => move |row| {
                            let value = row.is_active();
                            sender.input(PreferencesDialogInput::ContentTitlesBelowChanged(value));
                        }
                    },
                    adw::SwitchRow {
                        set_title: &t!("details_content_colors"),
                        set_active: model.settings.boolean("details-content-colors"),
                        connect_active_notify[sender] => move |row| {
                            let value = row.is_active();
                            sender.input(PreferencesDialogInput::DetailsContentColorsChanged(value));
                        }
                    },
                    adw::SwitchRow {
                        set_title: &t!("details_content_logo"),
                        set_active: model.settings.boolean("details-content-logo"),
                        connect_active_notify[sender] => move |row| {
                            let value = row.is_active();
                            sender.input(PreferencesDialogInput::DetailsContentLogoChanged(value));
                        }
                    },
                },

                add = &adw::PreferencesGroup {
                    set_title: &t!("server"),

                    #[wrap(Some)]
                    set_header_suffix = &gtk::Label {
                        #[watch]
                        set_label: &if server.online {
                            t!("online")
                        } else {
                            t!("offline")
                        },
                    },

                    adw::EntryRow {
                        set_title: &t!("url"),
                        set_text: &ctx.settings.streaming_server_url.as_str(),
                        connect_text_notify[sender] => move |row| {
                            let value = row.text().to_string();
                            sender.input(PreferencesDialogInput::ServerUrlChanged(value));
                        },
                    },
                    adw::SwitchRow {
                        set_title: &t!("autostart"),
                        set_active: model.settings.boolean("autostart-server"),
                        connect_active_notify[sender] => move |row| {
                            let value = row.is_active();
                            sender.input(PreferencesDialogInput::ServerEnabledChanged(value));
                        },
                    }
                },

                add = &adw::PreferencesGroup {
                    set_title: &t!("storage"),

                    adw::EntryRow {
                        set_title: &t!("location"),
                        set_text: &model.settings.string("storage-location"),
                        connect_text_notify[sender] => move |row| {
                            let value = row.text().to_string();
                            sender.input(PreferencesDialogInput::StorageLocationChanged(value));
                        },
                    }
                },
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let ctx = CTX_STATE.read_inner();
        let server = SERVER_STATE.read_inner();

        SERVER_STATE.subscribe(sender.input_sender(), |_| PreferencesDialogInput::Update);

        let settings = gio::Settings::new(APP_ID);

        let model = Self { settings };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn pre_view() {
        let server = SERVER_STATE.read_inner();
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
            PreferencesDialogInput::Open => {
                let window = relm4::main_application().active_window();
                root.present(window.as_ref());
                models::server::reload();
            }
            PreferencesDialogInput::Update => {}
            PreferencesDialogInput::CatalogsIconChanged(value) => {
                let _ = self.settings.set_boolean("catalog-addon-icon", value);
            }
            PreferencesDialogInput::CatalogsAddonNameChanged(value) => {
                let _ = self.settings.set_boolean("catalog-addon-name", value);
            }
            PreferencesDialogInput::ContentTitlesBelowChanged(value) => {
                let _ = self.settings.set_boolean("content-title-below", value);
            }
            PreferencesDialogInput::DetailsContentColorsChanged(value) => {
                let _ = self.settings.set_boolean("details-content-colors", value);
            }
            PreferencesDialogInput::DetailsContentLogoChanged(value) => {
                let _ = self.settings.set_boolean("details-content-logo", value);
            }
            PreferencesDialogInput::ServerUrlChanged(value) => {
                models::ctx::update_server_url(value);
            }
            PreferencesDialogInput::ServerEnabledChanged(value) => {
                let _ = self.settings.set_boolean("autostart-server", value);
            }
            PreferencesDialogInput::StorageLocationChanged(value) => {
                let _ = self.settings.set_string("storage-location", &value);
            }
        }
    }
}
