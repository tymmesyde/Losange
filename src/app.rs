use std::{path::Path, process::Child};

use adw::prelude::*;
use gtk::{gio, glib};
use relm4::{
    abstractions::Toaster,
    actions::{RelmAction, RelmActionGroup},
    adw,
    component::{AsyncComponent, AsyncComponentParts},
    gtk,
    loading_widgets::LoadingWidgets,
    prelude::*,
    view, AsyncComponentSender, Component, ComponentController, Controller,
};
use rust_i18n::t;
use shellexpand::tilde;
use stremio_core_losange::{core, models, types::stream::Stream};
use url::Url;

use crate::{
    components::{header_menu::HeaderMenu, spinner::Spinner},
    constants::APP_ID,
    dialogs::{
        about::AboutDialog,
        login::{LoginDialog, LoginDialogInput},
        preferences::{PreferencesDialog, PreferencesDialogInput},
    },
    pages::{
        addon::{AddonPage, AddonPageInput},
        addons::Addons,
        details::{DetailsPage, DetailsPageInput},
        discover::DiscoverPage,
        home::HomePage,
        library::LibraryPage,
        player::{Player, PlayerInput},
        search::{SearchPage, SearchPageInput},
    },
    server, Args,
};

#[derive(Debug)]
pub enum AppMsg {
    Toast(String),
    OpenHome,
    OpenSearch(Option<String>),
    OpenDetails((String, String)),
    OpenAddons,
    OpenAddon(Url),
    OpenStream(Box<Stream>),
    OpenPreferences(Option<&'static str>),
    NavigateBack,
}

pub struct App {
    toaster: Toaster,
    header_menu: Controller<HeaderMenu>,
    navigation_view: adw::NavigationView,
    view_stack: adw::ViewStack,
    home_page: Controller<HomePage>,
    discover_page: Controller<DiscoverPage>,
    library_page: Controller<LibraryPage>,
    search_page: Controller<SearchPage>,
    details_page: AsyncController<DetailsPage>,
    addons_page: AsyncController<Addons>,
    addon_page: Controller<AddonPage>,
    player_page: Controller<Player>,
    login_dialog: Controller<LoginDialog>,
    preferences_dialog: Controller<PreferencesDialog>,
    about_dialog: Controller<AboutDialog>,
    server_process: Option<Child>,
}

relm4::new_action_group!(pub(super) WindowActionGroup, "win");
relm4::new_stateless_action!(pub(super) LoginAction, WindowActionGroup, "login");
relm4::new_stateless_action!(pub(super) LogoutAction, WindowActionGroup, "logout");
relm4::new_stateless_action!(pub(super) PreferencesAction, WindowActionGroup, "preferences");
relm4::new_stateless_action!(pub(super) AboutAction, WindowActionGroup, "about");

#[relm4::component(async pub)]
impl AsyncComponent for App {
    type Init = Args;
    type Input = AppMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        main_window = adw::ApplicationWindow {
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

            #[local_ref]
            toast_overlay -> adw::ToastOverlay {
                set_expand: true,

                #[local_ref]
                navigation_view -> adw::NavigationView {
                    set_pop_on_escape: true,

                    add = &adw::NavigationPage {
                        set_title: "Main",
                        set_tag: Some("main"),

                        adw::ToolbarView {
                            #[name(header_bar)]
                            add_top_bar = &adw::HeaderBar {
                                #[wrap(Some)]
                                set_title_widget = &adw::ViewSwitcher {
                                    set_policy: adw::ViewSwitcherPolicy::Wide,
                                    set_stack: Some(view_stack),
                                },

                                pack_start = &gtk::Button {
                                    set_icon_name: "system-search-symbolic",
                                    set_tooltip_text: Some(&t!("search")),
                                    connect_clicked => AppMsg::OpenSearch(None),
                                },

                                pack_end = model.header_menu.widget(),
                            },

                            #[wrap(Some)]
                            set_content = &gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,

                                #[local_ref]
                                view_stack -> adw::ViewStack {
                                    add_titled_with_icon[Some("home"), &t!("home"), "go-home-symbolic"] = model.home_page.widget(),
                                    add_titled_with_icon[Some("discover"), &t!("discover"), "compass2"] = model.discover_page.widget(),
                                    add_titled_with_icon[Some("library"), &t!("library"), "library"] = model.library_page.widget(),
                                }
                            },

                            #[name(switcher_bar)]
                            add_bottom_bar = &adw::ViewSwitcherBar {
                                set_stack: Some(view_stack),
                            },
                        }
                    },

                    add = model.search_page.widget(),
                    add = model.details_page.widget(),
                    add = model.addons_page.widget(),
                    add = model.addon_page.widget(),
                    add = model.player_page.widget(),
                }
            }
        }
    }

    fn init_loading_widgets(root: Self::Root) -> Option<LoadingWidgets> {
        view! {
            #[local_ref]
            root {
                set_default_size: (1200, 750),

                #[name = "toolbar_view"]
                adw::ToolbarView {
                    add_top_bar = &adw::HeaderBar {
                        set_show_title: false,
                    },

                    #[wrap(Some)]
                    set_content = &gtk::Box {
                        #[template]
                        Spinner {},
                    }
                }
            }
        }

        Some(LoadingWidgets::new(root, toolbar_view))
    }

    async fn init(
        args: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        Self::initialize_core().await;
        let server_process = Self::initialize_server().await;

        models::ctx::sync_with_api();

        let header_menu = HeaderMenu::builder().launch(()).detach();

        let navigation_view = adw::NavigationView::default();
        let view_stack = adw::ViewStack::default();

        let home_page = HomePage::builder().launch(()).detach();
        let discover_page = DiscoverPage::builder().launch(()).detach();
        let library_page = LibraryPage::builder().launch(()).detach();
        let search_page = SearchPage::builder().launch(()).detach();
        let details_page = DetailsPage::builder().launch(()).detach();
        let addons_page = Addons::builder().launch(()).detach();
        let addon_page = AddonPage::builder().launch(()).detach();
        let player_page = Player::builder().launch(()).detach();

        let login_dialog = LoginDialog::builder().launch(()).detach();
        let preferences_dialog = PreferencesDialog::builder().launch(()).detach();
        let about_dialog = AboutDialog::builder().launch(()).detach();

        let model = App {
            toaster: Toaster::default(),
            header_menu,
            navigation_view,
            view_stack,
            home_page,
            discover_page,
            library_page,
            search_page,
            details_page,
            addons_page,
            addon_page,
            player_page,
            login_dialog,
            preferences_dialog,
            about_dialog,
            server_process,
        };

        let navigation_view = &model.navigation_view;
        let view_stack = &model.view_stack;
        let toast_overlay = model.toaster.overlay_widget();
        let widgets = view_output!();

        let mut actions = RelmActionGroup::<WindowActionGroup>::new();

        let login_action = {
            let sender = model.login_dialog.sender().clone();
            RelmAction::<LoginAction>::new_stateless(move |_| {
                sender.emit(LoginDialogInput::Open);
            })
        };

        let logout_action = {
            RelmAction::<LogoutAction>::new_stateless(move |_| {
                models::ctx::logout();
            })
        };

        let preferences_action = {
            let sender = model.preferences_dialog.sender().clone();
            RelmAction::<PreferencesAction>::new_stateless(move |_| {
                sender.emit(PreferencesDialogInput::Open(None));
            })
        };

        let about_action = {
            let sender = model.about_dialog.sender().clone();
            RelmAction::<AboutAction>::new_stateless(move |_| {
                sender.emit(());
            })
        };

        actions.add_action(login_action);
        actions.add_action(logout_action);
        actions.add_action(preferences_action);
        actions.add_action(about_action);
        actions.register_for_widget(&widgets.main_window);

        widgets.load_window_state();

        if let Some(manifest_url) = args.open {
            if let Ok(transport_url) = Url::parse(&manifest_url.replace("stremio://", "https://")) {
                sender.input_sender().emit(AppMsg::OpenAddon(transport_url));
            }
        }

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            AppMsg::Toast(message) => {
                let toast = adw::Toast::builder().title(message).timeout(3000).build();
                self.toaster.add_toast(toast);
            }
            AppMsg::OpenHome => {
                self.view_stack.set_visible_child_name("home");
            }
            AppMsg::OpenSearch(query) => {
                if let Some(query) = query {
                    self.search_page.emit(SearchPageInput::Load(query));
                }

                self.navigate("search");
            }
            AppMsg::OpenDetails(item) => {
                self.details_page.emit(DetailsPageInput::Load(item));
                self.navigate("details");
            }
            AppMsg::OpenAddons => {
                self.navigate("addons");
            }
            AppMsg::OpenAddon(transport_url) => {
                self.addon_page.emit(AddonPageInput::Load(transport_url));
                self.navigate("addon");
            }
            AppMsg::OpenStream(stream) => {
                self.player_page.emit(PlayerInput::Load(stream));
                self.navigate("player");
            }
            AppMsg::OpenPreferences(name) => {
                self.preferences_dialog
                    .emit(PreferencesDialogInput::Open(name));
            }
            AppMsg::NavigateBack => {
                self.navigation_view.pop();
            }
        }
    }

    fn shutdown(&mut self, widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
        widgets
            .save_window_state()
            .expect("Failed to save window state");

        if let Some(mut process) = self.server_process.take() {
            let _ = process.kill();
        }
    }
}

impl App {
    async fn initialize_core() {
        let settings = gio::Settings::new(APP_ID);
        let storage_location = settings.string("storage-location");

        let expanded_path = tilde(&storage_location).to_string();
        let data_location = Path::new(&expanded_path);

        core::initialize(data_location).await;
    }

    async fn initialize_server() -> Option<Child> {
        let settings = gio::Settings::new(APP_ID);
        let autostart = settings.boolean("autostart-server");
        let storage_location = settings.string("storage-location");

        let expanded_path = tilde(&storage_location).to_string();
        let data_location = Path::new(&expanded_path);

        if autostart {
            return server::initialize(data_location).await.ok();
        }

        None
    }

    fn navigate(&self, tag: &str) {
        if let Some(page) = self.navigation_view.find_page(tag) {
            if self.navigation_view.previous_page(&page).is_some() {
                self.navigation_view.pop_to_tag(tag);
            } else {
                self.navigation_view.push_by_tag(tag)
            }
        }
    }
}

impl AppWidgets {
    fn save_window_state(&self) -> Result<(), glib::BoolError> {
        let settings = gio::Settings::new(APP_ID);
        settings.set_boolean("is-maximized", self.main_window.is_maximized())?;

        Ok(())
    }

    fn load_window_state(&self) {
        let settings = gio::Settings::new(APP_ID);
        if settings.boolean("is-maximized") {
            self.main_window.maximize();
        }
    }
}
