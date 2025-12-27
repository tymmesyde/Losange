use adw::prelude::*;
use gtk::gio;
use relm4::{actions::RelmAction, adw, gtk, ComponentParts, ComponentSender, SimpleComponent};
use rust_i18n::t;
use stremio_core_losange::models::ctx::CTX_STATE;

use crate::{
    app::{AboutAction, AppMsg, LoginAction, LogoutAction, PreferencesAction, ShortcutsAction},
    constants::APP_NAME,
    APP_BROKER,
};

#[derive(Debug)]
pub enum HeaderMenuInput {
    Update,
}

pub struct HeaderMenu {
    menu_button: gtk::MenuButton,
}

#[relm4::component(pub)]
impl SimpleComponent for HeaderMenu {
    type Init = ();
    type Input = HeaderMenuInput;
    type Output = ();

    menu! {
        primary_menu: {
            section! {
                &t!("menu_login") => LoginAction,
            },
            section! {
                &t!("menu_preferences") => PreferencesAction,
                &t!("menu_shortcuts") => ShortcutsAction,
                &t!("menu_about", name = APP_NAME) => AboutAction,
            },
        }
    }

    view! {
        gtk::Box {
            set_spacing: 5,

            gtk::Button {
                set_icon_name: "puzzle-piece",
                set_tooltip_text: Some(&t!("addons")),
                connect_clicked => move |_| {
                    APP_BROKER.send(AppMsg::OpenAddons);
                },
            },

            #[local_ref]
            menu_button -> gtk::MenuButton {
                set_icon_name: "open-menu-symbolic",
                set_menu_model: Some(&primary_menu),
            }
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        CTX_STATE.subscribe(sender.input_sender(), |_| HeaderMenuInput::Update);

        let menu_button = gtk::MenuButton::default();

        let model = Self { menu_button };

        let menu_button = &model.menu_button;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            HeaderMenuInput::Update => {
                let ctx = CTX_STATE.read_inner();

                if let Some(menu_model) = self.menu_button.menu_model() {
                    if let Ok(menu) = menu_model.downcast::<gio::Menu>() {
                        let (title, item) = match &ctx.auth {
                            Some(auth) => (
                                Some(auth.user.email.as_str()),
                                RelmAction::<LogoutAction>::to_menu_item(&t!("menu_logout")),
                            ),
                            None => (
                                None,
                                RelmAction::<LoginAction>::to_menu_item(&t!("menu_login")),
                            ),
                        };

                        let section = gio::Menu::new();
                        section.append_item(&item);

                        menu.remove(0);
                        menu.insert_section(0, title, &section);
                    }
                }
            }
        }
    }
}
