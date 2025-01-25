mod sidebar;
mod tag;
mod tags;

use adw::prelude::*;
use gtk::gio;
use relm4::{
    adw, css,
    factory::FactoryVecDeque,
    gtk,
    prelude::{AsyncComponent, AsyncComponentController, AsyncComponentParts, AsyncController},
    AsyncComponentSender, Component, ComponentController, Controller, RelmWidgetExt,
};
use rust_i18n::t;
use sidebar::{Sidebar, SidebarInput};
use stremio_core_losange::models::{self, meta_details::META_DETAILS_STATE};
use tag::{Tag, TagOutput};
use tags::Tags;
use url::Url;

use crate::{
    app::AppMsg,
    common::{format::Format, image, net, style},
    components::image::{init::ImageInit, Image, ImageInput},
    constants::{APP_ID, DETAILS_LOGO_SIZE},
    APP_BROKER,
};

#[derive(Debug)]
pub enum DetailsPageInput {
    Load((String, String)),
    Unload,
    Update,
    AddToLibrary,
    RemoveFromLibrary,
    OpenSearch(String),
}

pub struct DetailsPage {
    settings: gio::Settings,
    css_provider: gtk::CssProvider,
    logo: AsyncController<Image>,
    genres: FactoryVecDeque<Tag>,
    directors: FactoryVecDeque<Tag>,
    writers: FactoryVecDeque<Tag>,
    actors: FactoryVecDeque<Tag>,
    sidebar: Controller<Sidebar>,
}

#[relm4::component(async pub)]
impl AsyncComponent for DetailsPage {
    type Init = ();
    type Input = DetailsPageInput;
    type Output = ();
    type CommandOutput = anyhow::Result<Vec<String>>;

    view! {
        adw::NavigationPage {
            set_title: "Details",
            set_tag: Some("details"),
            connect_hide => DetailsPageInput::Unload,

            adw::BreakpointBin {
                set_size_request: (150, 150),

                add_breakpoint = adw::Breakpoint::new(adw::BreakpointCondition::new_length(
                    adw::BreakpointConditionLengthType::MaxWidth,
                    800.0,
                    adw::LengthUnit::Sp,
                )) {
                    add_setter: (
                        &overlay_split_view,
                        "collapsed",
                        Some(&true.into()),
                    ),
                    add_setter: (
                        &overlay_split_view,
                        "show-sidebar",
                        Some(&true.into()),
                    ),
                },

                #[name = "overlay_split_view"]
                adw::OverlaySplitView {
                    add_css_class: "colors",
                    set_sidebar_position: gtk::PackType::End,
                    set_min_sidebar_width: 350.0,
                    set_max_sidebar_width: 450.0,

                    #[wrap(Some)]
                    set_content = &adw::ToolbarView {
                        add_top_bar = &adw::HeaderBar {
                            set_show_title: false,
                        },

                        #[wrap(Some)]
                        set_content = &gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_margin_all: 24,

                            #[transition = "Crossfade"]
                            match &meta_details.item {
                                Some(selected) => gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_expand: true,
                                    set_spacing: 36,

                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Vertical,
                                        set_spacing: 12,

                                        gtk::Box {
                                            model.logo.widget(),

                                            #[watch]
                                            set_visible: model.settings.boolean("details-content-logo"),
                                        },

                                        gtk::Label {
                                            add_css_class: relm4::css::classes::TITLE_1,
                                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                                            set_halign: gtk::Align::Start,
                                            #[watch]
                                            set_label: &selected.name,

                                            #[watch]
                                            set_visible: !model.settings.boolean("details-content-logo"),
                                        },

                                        gtk::Box {
                                            set_orientation: gtk::Orientation::Vertical,
                                            set_spacing: 16,

                                            match &selected.description {
                                                Some(description) => gtk::Label {
                                                    add_css_class: relm4::css::classes::DIM_LABEL,
                                                    set_halign: gtk::Align::Start,
                                                    set_wrap: true,
                                                    #[watch]
                                                    set_label: &description.no_line_breaks(),
                                                },
                                                None => gtk::Box,
                                            },

                                            #[local_ref]
                                            genres -> gtk::Box {
                                                set_spacing: 8,
                                                #[watch]
                                                set_visible: !selected.genres.is_empty(),
                                            }
                                        }
                                    },

                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Vertical,
                                        set_spacing: 24,

                                        #[template]
                                        Tags {
                                            #[watch]
                                            set_visible: !selected.directors.is_empty(),

                                            #[template_child]
                                            label {
                                                set_label: &t!("directors"),
                                            },

                                            #[local_ref]
                                            directors -> gtk::Box {
                                                set_spacing: 8,
                                            },
                                        },

                                        #[template]
                                        Tags {
                                            #[watch]
                                            set_visible: !selected.writers.is_empty(),

                                            #[template_child]
                                            label {
                                                set_label: &t!("writers"),
                                            },

                                            #[local_ref]
                                            writers -> gtk::Box {
                                                set_spacing: 8,
                                            },
                                        },

                                        #[template]
                                        Tags {
                                            #[watch]
                                            set_visible: !selected.actors.is_empty(),

                                            #[template_child]
                                            label {
                                                set_label: &t!("actors"),
                                            },

                                            #[local_ref]
                                            actors -> gtk::Box {
                                                set_spacing: 8,
                                            },
                                        }
                                    }
                                },
                                None => gtk::Box,
                            },

                            if meta_details.in_library {
                                gtk::Button {
                                    add_css_class: css::classes::PILL,
                                    set_halign: gtk::Align::Start,
                                    connect_clicked => DetailsPageInput::RemoveFromLibrary,

                                    adw::ButtonContent {
                                        set_icon_name: "list-remove-symbolic",
                                        set_label: &t!("remove_from_library"),
                                    }
                                }
                            } else {
                                gtk::Button {
                                    add_css_class: css::classes::PILL,
                                    set_halign: gtk::Align::Start,
                                    connect_clicked => DetailsPageInput::AddToLibrary,

                                    adw::ButtonContent {
                                        set_icon_name: "list-add-symbolic",
                                        set_label: &t!("add_to_library"),
                                    },
                                }
                            }
                        },
                    },

                    #[wrap(Some)]
                    set_sidebar = model.sidebar.widget(),
                },
            }
        }
    }

    async fn init(
        _: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let settings = gio::Settings::new(APP_ID);

        let meta_details = META_DETAILS_STATE.read_inner();

        META_DETAILS_STATE.subscribe(sender.input_sender(), |_| DetailsPageInput::Update);

        let css_provider = style::create_css_provider();

        let logo = Image::builder()
            .launch(
                ImageInit::builder()
                    .size(DETAILS_LOGO_SIZE)
                    .align(gtk::Align::Start)
                    .content_fit(gtk::ContentFit::Contain)
                    .build(),
            )
            .detach();

        let genres = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .detach();

        let directors = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .forward(sender.input_sender(), |msg| match msg {
                TagOutput::Clicked(value) => DetailsPageInput::OpenSearch(value),
            });

        let writers = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .forward(sender.input_sender(), |msg| match msg {
                TagOutput::Clicked(value) => DetailsPageInput::OpenSearch(value),
            });

        let actors = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .forward(sender.input_sender(), |msg| match msg {
                TagOutput::Clicked(value) => DetailsPageInput::OpenSearch(value),
            });

        let sidebar = Sidebar::builder().launch(()).detach();

        let model = DetailsPage {
            settings,
            css_provider,
            logo,
            genres,
            directors,
            writers,
            actors,
            sidebar,
        };

        let genres = model.genres.widget();
        let directors = model.directors.widget();
        let writers = model.writers.widget();
        let actors = model.actors.widget();
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    fn pre_view() {
        let meta_details = META_DETAILS_STATE.read_inner();
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            DetailsPageInput::Load((id, r#type)) => {
                self.sidebar.emit(SidebarInput::Reset);
                models::meta_details::load(&r#type, &id, None);
            }
            DetailsPageInput::Unload => {
                self.logo.emit(ImageInput::Unload);
            }
            DetailsPageInput::Update => {
                let state = META_DETAILS_STATE.read_inner();

                if let Some(item) = &state.item {
                    if self.settings.boolean("details-content-colors") {
                        sender.oneshot_command(Self::update_colors(
                            item.image.to_owned(),
                            style::has_dark_theme(),
                        ));
                    }

                    self.logo.emit(ImageInput::Update(item.logo.to_owned()));

                    Self::populate_tags(&mut self.genres, &item.genres);
                    Self::populate_tags(&mut self.directors, &item.directors);
                    Self::populate_tags(&mut self.writers, &item.writers);
                    Self::populate_tags(&mut self.actors, &item.actors);
                }
            }
            DetailsPageInput::AddToLibrary => models::meta_details::add_to_library(),
            DetailsPageInput::RemoveFromLibrary => models::meta_details::remove_from_library(),
            DetailsPageInput::OpenSearch(value) => APP_BROKER.send(AppMsg::OpenSearch(Some(value))),
        }
    }

    async fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        if let Ok(colors) = message {
            let css = style::colors_to_css(colors);
            self.css_provider.load_from_string(&css);
        }
    }
}

impl DetailsPage {
    fn populate_tags(factory: &mut FactoryVecDeque<Tag>, tags: &Vec<String>) {
        factory.guard().clear();
        for tag in tags {
            factory.guard().push_back(tag.to_owned());
        }
    }

    async fn update_colors(source: Option<Url>, dark_theme: bool) -> anyhow::Result<Vec<String>> {
        if let Some(uri) = source {
            let response = net::fetch(uri).await?;
            let bytes = response.bytes().await?;

            let pixbuf = image::pixbuf_from_bytes(bytes)?;

            if let Some(bytes) = pixbuf.pixel_bytes() {
                let colors = image::colors_from_bytes(&bytes, dark_theme)?;

                return Ok(colors);
            }
        }

        Err(anyhow::Error::msg("Failed to get the color palette"))
    }
}
