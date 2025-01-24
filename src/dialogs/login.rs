use adw::prelude::*;
use relm4::{adw, css, gtk, Component, ComponentParts, ComponentSender, RelmWidgetExt};
use rust_i18n::t;
use stremio_core_losange::{
    core::EVENTS,
    models::{self, ctx::CTX_STATE},
    stremio_core::runtime::msg::Event,
};

use crate::components::spinner::Spinner;

#[derive(Debug)]
pub enum LoginDialogInput {
    Open,
    Update,
    Login,
    Error,
}

pub struct LoginDialog {
    email: adw::EntryRow,
    password: adw::PasswordEntryRow,
    loading: bool,
    error: bool,
}

#[relm4::component(pub)]
impl Component for LoginDialog {
    type Init = ();
    type Input = LoginDialogInput;
    type Output = ();
    type CommandOutput = ();

    view! {
        adw::Dialog {
            set_content_width: 325,
            set_title: &t!("login"),

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar,

                #[wrap(Some)]
                set_content = &gtk::Box {
                    set_margin_all: 24,

                    #[transition = "Crossfade"]
                    if model.loading {
                        #[template]
                        Spinner {}
                    } else {
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 24,

                            gtk::ListBox {
                                add_css_class: css::classes::BOXED_LIST,

                                #[local_ref]
                                email -> adw::EntryRow {
                                    set_title: &t!("email"),
                                },

                                #[local_ref]
                                password -> adw::PasswordEntryRow {
                                    set_title: &t!("password"),
                                },
                            },

                            gtk::Label {
                                add_css_class: css::classes::ERROR,
                                set_label: &t!("incorrect_credentials"),
                                #[watch]
                                set_visible: model.error,
                            },

                            gtk::Button {
                                set_css_classes: &[css::classes::PILL, css::classes::SUGGESTED_ACTION],
                                set_label: &t!("continue"),

                                connect_clicked => LoginDialogInput::Login,
                            },
                        }
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        CTX_STATE.subscribe(sender.input_sender(), |_| LoginDialogInput::Update);
        EVENTS.subscribe(sender.input_sender(), |event| {
            if let Event::Error { source, .. } = event {
                if let Event::UserAuthenticated { .. } = source.as_ref() {
                    return Some(LoginDialogInput::Error);
                }
            }

            None
        });

        let email = adw::EntryRow::default();
        let password = adw::PasswordEntryRow::default();

        let model = Self {
            email,
            password,
            loading: false,
            error: false,
        };

        let email = &model.email;
        let password = &model.password;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
            LoginDialogInput::Open => {
                let window = relm4::main_application().active_window();
                root.present(window.as_ref());
            }
            LoginDialogInput::Update => {
                let ctx = CTX_STATE.read_inner();

                if ctx.auth.is_some() {
                    self.loading = false;
                    self.error = false;
                    self.password.set_text("");

                    root.close();
                }
            }
            LoginDialogInput::Login => {
                self.error = false;
                self.loading = true;

                let (email, password) = (self.email.text(), self.password.text());

                models::ctx::login(email.to_string(), password.to_string());
            }
            LoginDialogInput::Error => {
                self.loading = false;
                self.error = true;
            }
        }
    }
}
