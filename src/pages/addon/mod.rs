use adw::prelude::*;
use relm4::{
    adw, css, gtk,
    prelude::{AsyncComponent, AsyncComponentController, AsyncController},
    ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent,
};
use rust_i18n::t;
use stremio_core_losange::models::{self, addon_details::ADDON_DETAILS_STATE};
use url::Url;

use crate::components::{
    image::{init::ImageInit, Image, ImageInput},
    spinner::Spinner,
};

#[derive(Debug)]
pub enum AddonPageInput {
    Load(Url),
    Update,
    Install,
    Uninstall,
    Configure,
}

pub struct AddonPage {
    icon: AsyncController<Image>,
}

#[relm4::component(pub)]
impl SimpleComponent for AddonPage {
    type Init = ();
    type Input = AddonPageInput;
    type Output = ();

    view! {
        adw::NavigationPage {
            set_title: "Addon",
            set_tag: Some("addon"),

            adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    set_show_title: false,
                },

                #[wrap(Some)]
                set_content = &adw::Clamp {
                    set_valign: gtk::Align::Start,
                    set_maximum_size: 860,
                    set_tightening_threshold: 576,
                    set_margin_horizontal: 12,

                    match &state.addon {
                        Some(addon) => gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_margin_top: 26,
                            set_spacing: 18,

                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_halign: gtk::Align::Fill,
                                set_valign: gtk::Align::Start,
                                set_vexpand: true,
                                set_spacing: 12,

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Horizontal,
                                    set_align: gtk::Align::Start,
                                    set_hexpand: true,
                                    set_spacing: 32,

                                    gtk::Box {
                                        add_css_class: "large-icon",
                                        set_height_request: 110,
                                        set_width_request: 110,
                                        set_overflow: gtk::Overflow::Hidden,

                                        #[local_ref]
                                        icon -> adw::Clamp,
                                    },

                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Vertical,
                                        set_align: gtk::Align::Start,
                                        set_margin_top: 6,
                                        set_spacing: 6,

                                        gtk::Label {
                                            add_css_class: css::classes::TITLE_1,
                                            set_halign: gtk::Align::Start,
                                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                                            #[watch]
                                            set_label: &addon.name,
                                        },
                                        gtk::Label {
                                            add_css_class: css::classes::DIM_LABEL,
                                            set_halign: gtk::Align::Start,
                                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                                            #[watch]
                                            set_label: &addon.domain,
                                        },

                                        gtk::Box {
                                            set_orientation: gtk::Orientation::Horizontal,
                                            #[watch]
                                            set_visible: addon.official,
                                            set_spacing: 3,

                                            gtk::Image {
                                                add_css_class: css::classes::ACCENT,
                                                set_icon_name: Some("verified-checkmark-symbolic"),
                                            },
                                            gtk::Label {
                                                add_css_class: css::classes::ACCENT,
                                                set_halign: gtk::Align::Start,
                                                set_label: &t!("official"),
                                            },
                                        }
                                    },
                                },

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Horizontal,
                                    set_halign: gtk::Align::End,
                                    set_valign: gtk::Align::Center,
                                    set_spacing: 12,

                                    gtk::Button {
                                        set_width_request: 105,
                                        set_label: &t!("uninstall"),
                                        #[watch]
                                        set_visible: addon.installed,
                                        #[watch]
                                        set_sensitive: !addon.protected,
                                        connect_clicked => AddonPageInput::Uninstall,
                                    },

                                    gtk::Button {
                                        add_css_class: css::classes::SUGGESTED_ACTION,
                                        set_width_request: 105,
                                        set_label: &t!("configure"),
                                        #[watch]
                                        set_visible: addon.configuration_required && !addon.installed,
                                        connect_clicked => AddonPageInput::Configure,
                                    },

                                    gtk::Button {
                                        add_css_class: css::classes::SUGGESTED_ACTION,
                                        set_width_request: 105,
                                        set_label: &t!("install"),
                                        #[watch]
                                        set_visible: !addon.configuration_required && !addon.installed,
                                        connect_clicked => AddonPageInput::Install,
                                    },

                                    gtk::Button {
                                        set_icon_name: "emblem-system-symbolic",
                                        #[watch]
                                        set_visible: !addon.configuration_required && addon.configurable,
                                        connect_clicked => AddonPageInput::Configure,
                                    }
                                }
                            },

                            gtk::Label {
                                set_halign: gtk::Align::Start,
                                set_wrap: true,
                                #[watch]
                                set_label: &addon.description,
                            }
                        },
                        None => {
                            #[template]
                            Spinner {}
                        }
                    }
                }
            },
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let state = ADDON_DETAILS_STATE.read_inner();

        ADDON_DETAILS_STATE.subscribe(sender.input_sender(), |_| AddonPageInput::Update);

        let icon = Image::builder()
            .launch(
                ImageInit::builder()
                    .size(110)
                    .placeholder("puzzle-piece")
                    .placeholder_size(70)
                    .build(),
            )
            .detach();

        let model = AddonPage { icon };

        let icon = model.icon.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn pre_view() {
        let state = ADDON_DETAILS_STATE.read_inner();
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AddonPageInput::Load(transport_url) => {
                self.icon.emit(ImageInput::Unload);
                models::addon_details::load(&transport_url);
            }
            AddonPageInput::Update => {
                let state = ADDON_DETAILS_STATE.read_inner();
                if let Some(addon) = &state.addon {
                    self.icon.emit(ImageInput::Update(addon.icon.to_owned()))
                }
            }
            AddonPageInput::Install => {
                models::addon_details::install();
            }
            AddonPageInput::Uninstall => {
                models::addon_details::uninstall();
            }
            AddonPageInput::Configure => {
                let state = ADDON_DETAILS_STATE.read_inner();
                if let Some(addon) = &state.addon {
                    let configure_url = addon
                        .manifest_url
                        .as_str()
                        .replace("manifest.json", "configure");
                    let _ = open::that(configure_url);
                }
            }
        }
    }
}
