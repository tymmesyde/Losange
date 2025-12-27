mod stream_row;
mod video_row;

use std::time::Duration;

use adw::prelude::*;
use itertools::Itertools;
use relm4::{
    adw, css,
    gtk::{self, glib},
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmWidgetExt,
    Sender, SimpleComponent,
};
use rust_i18n::t;
use stream_row::StreamRow;
use stremio_core_losange::{
    models::{
        self,
        meta_details::{MetaDetailsStatus, META_DETAILS_STATE},
    },
    stremio_core::types::resource::StreamSource,
    types::video::Video,
};
use video_row::VideoRow;

use crate::{
    app::AppMsg,
    components::{
        dropdown::{DropDown, DropDownInput, DropDownOutput},
        header_menu::HeaderMenu,
        list::{List, ListInput, ListOutput},
        spinner::Spinner,
    },
    APP_BROKER,
};

#[derive(Debug)]
pub enum SidebarInput {
    Update,
    Clear,
    OpenAddons,
    SeasonChanged(usize),
    VideoClicked(usize),
    AddonChanged(usize),
    StreamClicked(usize),
    Reset,
}

pub struct Sidebar {
    header_menu: Controller<HeaderMenu>,
    selected_season: usize,
    seasons: Controller<DropDown>,
    videos: Controller<List<VideoRow>>,
    debounce_videos: Option<glib::JoinHandle<()>>,
    selected_addon: usize,
    addons: Controller<DropDown>,
    streams: Controller<List<StreamRow>>,
    debounce_streams: Option<glib::JoinHandle<()>>,
    selected_video: Option<Video>,
}

#[relm4::component(pub)]
impl SimpleComponent for Sidebar {
    type Init = ();
    type Input = SidebarInput;
    type Output = ();

    view! {
        adw::ToolbarView {
            add_css_class: "toolbar",

            connect_map => SidebarInput::Update,
            connect_unmap => SidebarInput::Clear,

            add_top_bar = &adw::HeaderBar {
                add_css_class: relm4::css::classes::FLAT,
                set_show_back_button: false,
                set_show_title: false,

                pack_start = match &model.selected_video {
                    Some(video) => gtk::Box {
                        set_spacing: 5,

                        gtk::Button {
                            set_icon_name: "go-previous-symbolic",
                            connect_clicked => SidebarInput::Reset,
                        },
                        gtk::Label {
                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                            #[watch]
                            set_label: &video.name,
                        }
                    }
                    None => gtk::Box,
                },

                pack_end = model.header_menu.widget(),
            },

            #[wrap(Some)]
            set_content = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_horizontal: 14,
                set_margin_top: 6,
                set_margin_bottom: 14,

                #[transition = "Crossfade"]
                if !state.videos.is_empty() && model.selected_video.is_none() {
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_expand: true,
                        set_spacing: 6,

                        model.seasons.widget(),
                        model.videos.widget(),
                    }
                } else {
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_expand: true,

                        match state.status {
                            MetaDetailsStatus::Ready => {
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_expand: true,
                                    set_spacing: 6,

                                    model.addons.widget(),
                                    model.streams.widget(),
                                }
                            }
                            MetaDetailsStatus::Error => {
                                adw::StatusPage {
                                    add_css_class: css::classes::COMPACT,
                                    set_title: &t!("no_streams"),
                                    set_description: Some(&t!("no_streams_description")),

                                    gtk::Button {
                                        set_css_classes: &[css::classes::PILL, css::classes::SUGGESTED_ACTION],
                                        set_label: &t!("install_more_addons"),
                                        connect_clicked => SidebarInput::OpenAddons,
                                    }
                                }
                            }
                            MetaDetailsStatus::Loading => {
                                #[template]
                                Spinner {}
                            }
                        },
                    }
                }
            }
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        META_DETAILS_STATE.subscribe(sender.input_sender(), |_| SidebarInput::Update);

        let state = META_DETAILS_STATE.read_inner();

        let header_menu = HeaderMenu::builder().launch(()).detach();

        let seasons =
            DropDown::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    DropDownOutput::Selected(index) => SidebarInput::SeasonChanged(index),
                });

        let videos = List::<VideoRow>::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| match msg {
                ListOutput::Clicked(index) => SidebarInput::VideoClicked(index),
            });

        let addons =
            DropDown::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    DropDownOutput::Selected(index) => SidebarInput::AddonChanged(index),
                });

        let streams = List::<StreamRow>::builder().launch(()).forward(
            sender.input_sender(),
            |msg| match msg {
                ListOutput::Clicked(index) => SidebarInput::StreamClicked(index),
            },
        );

        let model = Sidebar {
            header_menu,
            selected_season: 0,
            seasons,
            videos,
            debounce_videos: None,
            selected_addon: 0,
            addons,
            streams,
            debounce_streams: None,
            selected_video: None,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn pre_view() {
        let state = META_DETAILS_STATE.read_inner();
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            SidebarInput::Update => {
                let state = META_DETAILS_STATE.read_inner();

                let seasons = state
                    .videos
                    .iter()
                    .map(|(season, ..)| match season {
                        0 => t!("special_season").to_string(),
                        _ => t!("season", n = season).to_string(),
                    })
                    .collect_vec();

                self.seasons.emit(DropDownInput::Update(seasons));

                self.update_videos(true);

                let addons = state
                    .streams
                    .iter()
                    .map(|(transport_url, ..)| transport_url.to_owned())
                    .collect_vec();

                self.addons.emit(DropDownInput::Update(addons));

                self.update_streams(true);
            }
            SidebarInput::Clear => {
                self.videos.emit(ListInput::Clear);
                self.streams.emit(ListInput::Clear);
            }
            SidebarInput::OpenAddons => {
                APP_BROKER.send(AppMsg::OpenAddons);
            }
            SidebarInput::SeasonChanged(index) => {
                self.selected_season = index;
                self.update_videos(false);
            }
            SidebarInput::VideoClicked(index) => {
                let state = META_DETAILS_STATE.read_inner();

                if let Some((.., videos)) = state.videos.get(self.selected_season) {
                    if let (Some(item), Some(video)) = (&state.item, videos.get(index)) {
                        self.selected_video = Some(video.to_owned());
                        models::meta_details::load(&item.r#type, &item.id, Some(&video.id));
                    }
                }
            }
            SidebarInput::AddonChanged(index) => {
                self.selected_addon = index;
                self.update_streams(false);
            }
            SidebarInput::StreamClicked(index) => {
                let state = META_DETAILS_STATE.read_inner();

                if let Some((.., streams)) = state.streams.get(self.selected_addon) {
                    if let Some(stream) = streams.get(index) {
                        match &stream.source {
                            StreamSource::External {
                                external_url: Some(url),
                                ..
                            } => {
                                let _ = open::that(url.as_str());
                            }
                            StreamSource::Url { .. } | StreamSource::Torrent { .. } => {
                                APP_BROKER.send(AppMsg::OpenStream(Box::new(stream.to_owned())));
                            }
                            _ => {}
                        }
                    }
                }
            }
            SidebarInput::Reset => {
                self.selected_video = None;
            }
        }
    }
}

impl Sidebar {
    fn update_videos(&mut self, debounced: bool) {
        let state = META_DETAILS_STATE.read_inner();

        if let Some((.., videos)) = state.videos.get(self.selected_season) {
            Self::update(
                videos.to_owned(),
                self.videos.sender().clone(),
                &mut self.debounce_videos,
                debounced,
            );
        }
    }

    fn update_streams(&mut self, debounced: bool) {
        let state = META_DETAILS_STATE.read_inner();

        if let Some((.., streams)) = state.streams.get(self.selected_addon) {
            Self::update(
                streams.to_owned(),
                self.streams.sender().clone(),
                &mut self.debounce_streams,
                debounced,
            );
        }
    }

    fn update<T: Clone + Send + 'static>(
        items: Vec<T>,
        sender: Sender<ListInput<T>>,
        debouncer: &mut Option<glib::JoinHandle<()>>,
        debounced: bool,
    ) {
        if debounced {
            if let Some(debounce) = debouncer.take() {
                debounce.abort();
            }

            *debouncer = Some(relm4::spawn_local(async move {
                tokio::time::sleep(Duration::from_millis(250)).await;
                sender.emit(ListInput::Update(items));
            }));
        } else {
            sender.emit(ListInput::Update(items));
        }
    }
}
