mod menu_option;

use itertools::Itertools;
use menu_option::{MenuOption, MenuOptionInit, MenuOptionOutput};
use relm4::{
    gtk::{
        self,
        prelude::{OrientableExt, WidgetExt},
    },
    prelude::FactoryVecDeque,
    ComponentParts, ComponentSender, SimpleComponent,
};

use super::video::MediaTrack;

type TracksMenuInit = &'static str;

#[derive(Debug)]
pub enum TracksMenuInput {
    Update(Vec<MediaTrack>),
    TrackClicked(i32),
}

#[derive(Debug)]
pub enum TracksMenuOutput {
    TrackChanged(i32),
}

pub struct TracksMenu {
    icon: &'static str,
    group: gtk::CheckButton,
    tracks: FactoryVecDeque<MenuOption>,
}

#[relm4::component(pub)]
impl SimpleComponent for TracksMenu {
    type Init = TracksMenuInit;
    type Input = TracksMenuInput;
    type Output = TracksMenuOutput;

    view! {
        gtk::MenuButton {
            add_css_class: relm4::css::classes::OSD,
            set_icon_name: model.icon,

            #[watch]
            set_visible: !model.tracks.is_empty(),

            #[wrap(Some)]
            set_popover = &gtk::Popover {
                gtk::ScrolledWindow {
                    set_hscrollbar_policy: gtk::PolicyType::Never,
                    set_propagate_natural_height: true,
                    set_max_content_height: 300,
                    set_min_content_width: 200,

                    #[local_ref]
                    tracks -> gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_vexpand: true,
                        set_hexpand: true,
                    }
                }
            }
        },
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let tracks = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .forward(sender.input_sender(), |msg| match msg {
                MenuOptionOutput::Clicked(index) => TracksMenuInput::TrackClicked(index),
            });

        let group = gtk::CheckButton::new();

        let model = TracksMenu {
            icon: init,
            group,
            tracks,
        };

        let tracks = model.tracks.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            TracksMenuInput::Update(media_tracks) => {
                let tracks = media_tracks
                    .iter()
                    .sorted_by(|a, b| Ord::cmp(&a.label, &b.label))
                    .collect_vec();

                for (i, track) in tracks.iter().enumerate() {
                    if i >= self.tracks.len() {
                        self.tracks.guard().push_back(MenuOptionInit {
                            id: track.id,
                            label: track.label.to_owned(),
                            active: track.active,
                            group: self.group.to_owned(),
                        });
                    } else if tracks[i].id != self.tracks[i].id {
                        self.tracks.guard().insert(
                            i,
                            MenuOptionInit {
                                id: track.id,
                                label: track.label.to_owned(),
                                active: track.active,
                                group: self.group.to_owned(),
                            },
                        );
                    }
                }

                while self.tracks.len() > tracks.len() {
                    self.tracks.guard().pop_back();
                }
            }
            TracksMenuInput::TrackClicked(index) => {
                sender
                    .output_sender()
                    .emit(TracksMenuOutput::TrackChanged(index));
            }
        }
    }
}
