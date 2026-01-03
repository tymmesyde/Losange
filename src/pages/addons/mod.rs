mod addon_row;

use addon_row::{AddonRow, AddonRowOutput};
use adw::prelude::*;
use relm4::prelude::*;
use relm4::{adw, factory::FactoryVecDeque, gtk, AsyncComponentSender, RelmWidgetExt};
use rust_i18n::t;
use stremio_core_losange::{
    models::{self, installed_addons::INSTALLED_ADDONS_STATE, remote_addons::REMOTE_ADDONS_STATE},
    stremio_core::constants::OFFICIAL_ADDONS,
    types::addon::Addon,
};

use crate::app::AppMsg;
use crate::components::spinner::Spinner;
use crate::constants::COMMUNITY_MANIFESTS;
use crate::APP_BROKER;

#[derive(Debug)]
pub enum AddonsInput {
    LoadInstalled,
    LoadOfficial,
    LoadCommunity,
    UpdateInstalled,
    UpdateCommunity,
    InstalledAddonClicked(usize),
    OfficialAddonClicked(usize),
    CommunityAddonClicked(usize),
    ManifestChanged(usize),
}

pub struct Addons {
    installed_list: FactoryVecDeque<AddonRow>,
    official_list: FactoryVecDeque<AddonRow>,
    community_list: FactoryVecDeque<AddonRow>,
    manifest_dropdown: gtk::DropDown,
}

#[relm4::component(pub async)]
impl AsyncComponent for Addons {
    type Init = ();
    type Input = AddonsInput;
    type Output = ();
    type CommandOutput = ();

    view! {
        adw::NavigationPage {
            set_title: "Addons",
            set_tag: Some("addons"),

            adw::BreakpointBin {
                set_size_request: (150, 150),

                add_breakpoint = adw::Breakpoint::new(adw::BreakpointCondition::new_length(
                    adw::BreakpointConditionLengthType::MaxWidth,
                    600.0,
                    adw::LengthUnit::Sp,
                )) {
                    add_setter: (
                        &header_bar,
                        "show-title",
                        Some(&false.into()),
                    ),
                    add_setter: (
                        &switcher_bar,
                        "reveal",
                        Some(&true.into()),
                    )
                },

                adw::ToolbarView {
                    #[name(header_bar)]
                    add_top_bar = &adw::HeaderBar {
                        #[wrap(Some)]
                        set_title_widget = &adw::ViewSwitcher {
                            set_policy: adw::ViewSwitcherPolicy::Wide,
                            set_stack: Some(&view_stack),
                        }
                    },

                    #[name = "view_stack"]
                    #[wrap(Some)]
                    set_content = &adw::ViewStack {
                        add_titled_with_icon[Some("installed"), &t!("installed_addons"), "folder-download-symbolic"] = &adw::PreferencesPage {
                            add = &adw::PreferencesGroup {
                                set_title: &t!("addons"),

                                #[local_ref]
                                installed_list -> gtk::ListBox {
                                    add_css_class: relm4::css::classes::BOXED_LIST,
                                    set_expand: true,
                                    set_valign: gtk::Align::Start,
                                },
                            },

                            connect_realize => AddonsInput::LoadInstalled,
                        },

                        add_titled_with_icon[Some("official"), &t!("official_addons"), "verified-checkmark"] = &adw::PreferencesPage {
                            add = &adw::PreferencesGroup {
                                set_title: &t!("addons"),

                                #[local_ref]
                                official_list -> gtk::ListBox {
                                    add_css_class: relm4::css::classes::BOXED_LIST,
                                    set_expand: true,
                                    set_valign: gtk::Align::Start,
                                },
                            },

                            connect_realize => AddonsInput::LoadOfficial,
                        },

                        add_titled_with_icon[Some("community"), &t!("community_addons"), "people"] = &adw::PreferencesPage {
                            add = &adw::PreferencesGroup {
                                set_title: &t!("addons"),

                                #[wrap(Some)]
                                set_header_suffix = &gtk::Box {
                                    #[local_ref]
                                    manifest_dropdown -> gtk::DropDown {
                                        add_css_class: relm4::css::classes::FLAT,
                                        connect_selected_item_notify[sender] => move |dropdown| {
                                            let id = dropdown.selected() as usize;
                                            sender.input_sender().emit(AddonsInput::ManifestChanged(id));
                                        }
                                    },
                                },

                                #[transition = "Crossfade"]
                                if remote_addons.loading {
                                    #[template]
                                    Spinner {}
                                } else {
                                    gtk::Box {
                                        #[local_ref]
                                        community_list -> gtk::ListBox {
                                            add_css_class: relm4::css::classes::BOXED_LIST,
                                            set_expand: true,
                                            set_valign: gtk::Align::Start,
                                        },
                                    }
                                }
                            },

                            connect_realize => AddonsInput::LoadCommunity,
                        },
                    },

                    #[name(switcher_bar)]
                    add_bottom_bar = &adw::ViewSwitcherBar {
                        set_stack: Some(&view_stack),
                    },
                }
            },
        }
    }

    async fn init(
        _: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let remote_addons = REMOTE_ADDONS_STATE.read_inner();

        INSTALLED_ADDONS_STATE.subscribe(sender.input_sender(), |_| AddonsInput::UpdateInstalled);
        REMOTE_ADDONS_STATE.subscribe(sender.input_sender(), |_| AddonsInput::UpdateCommunity);

        let installed_list = FactoryVecDeque::builder()
            .launch(gtk::ListBox::new())
            .forward(sender.input_sender(), |msg| match msg {
                AddonRowOutput::Clicked(index) => AddonsInput::InstalledAddonClicked(index),
            });

        let official_list = FactoryVecDeque::builder()
            .launch(gtk::ListBox::new())
            .forward(sender.input_sender(), |msg| match msg {
                AddonRowOutput::Clicked(index) => AddonsInput::OfficialAddonClicked(index),
            });

        let community_list = FactoryVecDeque::builder()
            .launch(gtk::ListBox::new())
            .forward(sender.input_sender(), |msg| match msg {
                AddonRowOutput::Clicked(index) => AddonsInput::CommunityAddonClicked(index),
            });

        let manifest_dropdown = gtk::DropDown::from_strings(&[&t!("default"), &t!("alternative")]);

        let model = Addons {
            installed_list,
            official_list,
            community_list,
            manifest_dropdown,
        };

        let installed_list = model.installed_list.widget();
        let official_list = model.official_list.widget();
        let community_list = model.community_list.widget();
        let manifest_dropdown = &model.manifest_dropdown;
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn pre_view() {
        let remote_addons = REMOTE_ADDONS_STATE.read_inner();
    }

    async fn update(
        &mut self,
        message: Self::Input,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            AddonsInput::LoadInstalled => {
                models::installed_addons::load();
            }
            AddonsInput::LoadOfficial => {
                self.official_list.guard().clear();
                self.official_list
                    .extend(OFFICIAL_ADDONS.iter().map(Addon::from));
            }
            AddonsInput::LoadCommunity => {
                models::remote_addons::load(COMMUNITY_MANIFESTS[0]);
            }
            AddonsInput::UpdateInstalled => {
                let state = INSTALLED_ADDONS_STATE.read_inner();

                self.installed_list.guard().clear();
                self.installed_list.extend(state.addons.to_owned());
            }
            AddonsInput::UpdateCommunity => {
                let state = REMOTE_ADDONS_STATE.read_inner();

                self.community_list.guard().clear();
                self.community_list.extend(state.addons.to_owned());
            }
            AddonsInput::InstalledAddonClicked(index) => {
                let state = INSTALLED_ADDONS_STATE.read_inner();
                if let Some(addon) = state.addons.get(index) {
                    APP_BROKER.send(AppMsg::OpenAddon(addon.manifest_url.to_owned()));
                }
            }
            AddonsInput::OfficialAddonClicked(index) => {
                if let Some(addon) = OFFICIAL_ADDONS.get(index) {
                    APP_BROKER.send(AppMsg::OpenAddon(addon.transport_url.to_owned()));
                }
            }
            AddonsInput::CommunityAddonClicked(index) => {
                let state = REMOTE_ADDONS_STATE.read_inner();
                if let Some(addon) = state.addons.get(index) {
                    APP_BROKER.send(AppMsg::OpenAddon(addon.manifest_url.to_owned()));
                }
            }
            AddonsInput::ManifestChanged(id) => {
                if let Some(manifest) = COMMUNITY_MANIFESTS.get(id) {
                    models::remote_addons::load(manifest);
                }
            }
        }
    }
}
