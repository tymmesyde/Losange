use crate::constants::{APP_ID, SUBTITLES_MAX_SIZE, SUBTITLES_MIN_SIZE};
use adw::prelude::*;
use gtk::gio;
use relm4::{adw, gtk, Component, ComponentParts, ComponentSender};
use rust_i18n::t;
use stremio_core_losange::models::{self, ctx::CTX_STATE, server::SERVER_STATE};

#[derive(Debug)]
pub enum PreferencesDialogInput {
    Open(Option<&'static str>),
    Update,
    CatalogsIconChanged(bool),
    CatalogsAddonNameChanged(bool),
    ContentTitlesBelowChanged(bool),
    DetailsContentColorsChanged(bool),
    DetailsContentLogoChanged(bool),
    PlayerSubtitlesSizeChanged(f64),
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
                set_name: Some("appereance"),
                set_title: &t!("appereance"),
                set_icon_name: Some("preferences-desktop-appearance-symbolic"),
                set_margin_bottom: 26,

                add = &adw::PreferencesGroup {
                    set_title: &t!("general"),

                    adw::SwitchRow {
                        set_title: &t!("content_title_below"),
                        set_active: model.settings.boolean("content-title-below"),
                        connect_active_notify[sender] => move |row| {
                            let value = row.is_active();
                            sender.input(PreferencesDialogInput::ContentTitlesBelowChanged(value));
                        }
                    },
                },

                add = &adw::PreferencesGroup {
                    set_title: &t!("catalog"),

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
                },

                add = &adw::PreferencesGroup {
                    set_title: &t!("details"),

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
                }
            },

            add = &adw::PreferencesPage {
                set_name: Some("player"),
                set_title: &t!("player"),
                set_icon_name: Some("play"),
                set_margin_bottom: 26,

                add = &adw::PreferencesGroup {
                    adw::SpinRow::with_range(SUBTITLES_MIN_SIZE as f64, SUBTITLES_MAX_SIZE as f64, 25.0) {
                        set_title: &t!("subtitles_size"),

                        #[watch]
                        #[block_signal(subtitles_size_handler)]
                        set_value: ctx.settings.subtitles_size.into(),

                        connect_value_notify[sender] => move |row| {
                            let value = row.value();
                            sender.input(PreferencesDialogInput::PlayerSubtitlesSizeChanged(value));
                        } @subtitles_size_handler,
                    }
                },
            },

            add = &adw::PreferencesPage {
                set_name: Some("server"),
                set_title: &t!("server"),
                set_icon_name: Some("network-server-symbolic"),
                set_margin_bottom: 26,

                add = &adw::PreferencesGroup {
                    adw::ActionRow {
                        set_title: &t!("status"),
                        add_suffix = &gtk::Label {

                            #[watch]
                            set_label: &if server.online {
                                t!("online")
                            } else {
                                t!("offline")
                            }
                        }
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
            },

            add = &adw::PreferencesPage {
                set_name: Some("storage"),
                set_title: &t!("storage"),
                set_icon_name: Some("drive-harddisk-symbolic"),
                set_margin_bottom: 26,

                add = &adw::PreferencesGroup {
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

        CTX_STATE.subscribe(sender.input_sender(), |_| PreferencesDialogInput::Update);
        SERVER_STATE.subscribe(sender.input_sender(), |_| PreferencesDialogInput::Update);

        let settings = gio::Settings::new(APP_ID);

        let model = Self { settings };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn pre_view() {
        let ctx = CTX_STATE.read_inner();
        let server = SERVER_STATE.read_inner();
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
            PreferencesDialogInput::Open(name) => {
                let window = relm4::main_application().active_window();
                root.present(window.as_ref());
                models::server::reload();

                if let Some(name) = name {
                    root.set_visible_page_name(name);
                }
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
            PreferencesDialogInput::PlayerSubtitlesSizeChanged(value) => {
                models::ctx::update_subtitles_size(value);
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
