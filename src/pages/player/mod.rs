mod tracks_menu;
mod video;

use std::time::Duration;

use adw::prelude::*;
use gtk::glib;
use relm4::{
    adw, gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller,
    JoinHandle, RelmWidgetExt, SimpleComponent,
};
use rust_i18n::t;
use stremio_core_losange::{
    models::{self, ctx::CTX_STATE, player::PLAYER_STATE},
    types::stream::Stream,
};
use tokio::time::sleep;
use tracks_menu::{TracksMenu, TracksMenuInput, TracksMenuOutput};
use video::{Video, VideoInput, VideoOutput, VIDEO_STATE};

use crate::{
    app::AppMsg,
    components::spinner::Spinner,
    constants::{SUBTITLES_FONT_SIZES, SUBTITLES_MIN_SIZE},
    APP_BROKER,
};

#[derive(Debug)]
pub enum PlayerInput {
    Load(Box<Stream>),
    Unload,
    UpdateVideo,
    UpdateView,
    MouseMove((f64, f64)),
    MouseEnterControls,
    MouseLeaveControls,
    Immersed,
    PlayPause,
    Seek,
    SeekPrev,
    SeekNext,
    Volume,
    TextTrackChanged(i32),
    AudioTrackChanged(i32),
    Fullscreen,
    PauseChanged(bool),
    TimeChanged(f64, f64),
    Ended,
    Error,
}

pub struct Player {
    video: Controller<Video>,
    immersed: bool,
    fullscreen: bool,
    start_time: bool,
    mouse_position: (f64, f64),
    immersed_timeout: Option<JoinHandle<()>>,
    hovering_controls: bool,
    seekbar: gtk::Scale,
    volume: gtk::ScaleButton,
    text_tracks_menu: Controller<TracksMenu>,
    audio_tracks_menu: Controller<TracksMenu>,
}

#[relm4::component(pub)]
impl SimpleComponent for Player {
    type Init = ();
    type Input = PlayerInput;
    type Output = ();

    view! {
        adw::NavigationPage {
            set_title: "Player",
            set_tag: Some("player"),
            set_focusable: true,

            connect_hidden => PlayerInput::Unload,

            add_controller = gtk::EventControllerMotion {
                connect_motion[sender] => move |_, x, y| {
                    sender.input_sender().emit(PlayerInput::MouseMove((x, y)));
                },
            },

            add_controller = gtk::EventControllerKey {
                connect_key_pressed[sender] => move |_, key, _, _| {
                    match key.name() {
                        Some(name) => match name.as_ref() {
                            "space" => {
                                sender.input_sender().emit(PlayerInput::PlayPause);
                                glib::Propagation::Stop
                            },
                            "Left" => {
                                sender.input_sender().emit(PlayerInput::SeekPrev);
                                glib::Propagation::Stop
                            },
                            "Right" => {
                                sender.input_sender().emit(PlayerInput::SeekNext);
                                glib::Propagation::Stop
                            },
                            _ => glib::Propagation::Proceed
                        }
                        None => glib::Propagation::Proceed,
                    }
                },
            },

            gtk::Box {
                add_css_class: relm4::css::classes::OSD,
                set_orientation: gtk::Orientation::Vertical,
                set_expand: true,
                #[watch]
                set_cursor_from_name: model.immersed.then_some("none"),

                add_controller = gtk::GestureClick {
                    connect_pressed[sender] => move |_, clicks, _, _| {
                        if clicks == 2 {
                            sender.input_sender().emit(PlayerInput::Fullscreen);
                        }
                    }
                },

                gtk::Overlay {
                    add_overlay = &gtk::Revealer {
                        set_valign: gtk::Align::Start,
                        set_transition_type: gtk::RevealerTransitionType::Crossfade,
                        #[watch]
                        set_reveal_child: state.paused || !model.immersed,

                        adw::HeaderBar {
                            add_css_class: relm4::css::classes::FLAT,
                            set_valign: gtk::Align::Start,

                            #[wrap(Some)]
                            set_title_widget = &gtk::Label {
                                #[watch]
                                set_label: &player.title,
                            },

                            pack_end = &gtk::Button {
                                #[watch]
                                set_icon_name: match model.fullscreen {
                                    true => "view-restore-symbolic",
                                    false => "view-fullscreen-symbolic",
                                },
                                connect_clicked => PlayerInput::Fullscreen,
                            }
                        }
                    },

                    add_overlay = &gtk::Revealer {
                        set_valign: gtk::Align::Center,
                        set_transition_type: gtk::RevealerTransitionType::Crossfade,
                        #[watch]
                        set_reveal_child: state.buffering,

                        #[template]
                        Spinner {}
                    },

                    add_overlay = &gtk::Revealer {
                        set_valign: gtk::Align::End,
                        set_transition_type: gtk::RevealerTransitionType::Crossfade,

                        #[watch]
                        set_reveal_child: state.paused || !model.immersed,

                        gtk::Box {
                            set_css_classes: &[relm4::css::classes::OSD, relm4::css::classes::TOOLBAR],
                            set_valign: gtk::Align::End,
                            set_margin_all: 12,

                            add_controller = gtk::EventControllerMotion {
                                connect_enter[sender] => move |_, _, _| {
                                    sender.input_sender().emit(PlayerInput::MouseEnterControls);
                                },
                                connect_leave[sender] => move |_| {
                                    sender.input_sender().emit(PlayerInput::MouseLeaveControls);
                                },
                            },

                            gtk::Button {
                                #[watch]
                                set_icon_name: match state.paused {
                                    true => "media-playback-start-symbolic",
                                    false => "media-playback-pause-symbolic",
                                },

                                connect_clicked => PlayerInput::PlayPause,
                            },

                            #[local_ref]
                            volume -> gtk::ScaleButton {
                                set_icons: &[
                                    "audio-volume-muted",
                                    "audio-volume-high",
                                    "audio-volume-low",
                                    "audio-volume-medium",
                                ],

                                #[watch]
                                #[block_signal(volume_handler)]
                                set_value: state.volume,

                                connect_value_changed[sender] => move |_, _| {
                                    sender.input(PlayerInput::Volume);
                                } @volume_handler,
                            },

                            gtk::Label {
                                set_align: gtk::Align::Center,
                                set_width_request: 80,
                                #[watch]
                                set_label: &Self::ms_to_clock(state.time),
                            },

                            #[local_ref]
                            seekbar -> gtk::Scale {
                                set_hexpand: true,

                                #[watch]
                                set_range: (0.0, state.duration),

                                #[watch]
                                #[block_signal(time_handler)]
                                set_value: state.time,

                                connect_change_value[sender] => move |_, _, _| {
                                    sender.input(PlayerInput::Seek);
                                    glib::Propagation::Proceed
                                } @time_handler,
                            },

                            gtk::Label {
                                set_align: gtk::Align::Center,
                                set_width_request: 80,

                                #[watch]
                                set_label: &Self::ms_to_clock(state.duration),

                                #[watch]
                                set_visible: state.duration.gt(&0.0),
                            },

                            model.text_tracks_menu.widget(),
                            model.audio_tracks_menu.widget(),

                            gtk::Button {
                                set_icon_name: "settings",
                                connect_clicked => move |_| {
                                    APP_BROKER.send(AppMsg::OpenPreferences(Some("player")));
                                },
                            }
                        }
                    },

                    model.video.widget(),
                }
            }
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let player = PLAYER_STATE.read_inner();
        let state = VIDEO_STATE.read_inner();

        CTX_STATE.subscribe(sender.input_sender(), |_| PlayerInput::UpdateVideo);
        PLAYER_STATE.subscribe(sender.input_sender(), |_| PlayerInput::UpdateVideo);
        VIDEO_STATE.subscribe(sender.input_sender(), |_| PlayerInput::UpdateView);

        let video = Video::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| match msg {
                VideoOutput::PauseChanged(paused) => PlayerInput::PauseChanged(paused),
                VideoOutput::TimeChanged(time, duration) => {
                    PlayerInput::TimeChanged(time, duration)
                }
                VideoOutput::Ended => PlayerInput::Ended,
                VideoOutput::Error => PlayerInput::Error,
            });

        let seekbar = gtk::Scale::default();
        let volume = gtk::ScaleButton::default();

        let text_tracks_menu = TracksMenu::builder().launch("language").forward(
            sender.input_sender(),
            |msg| match msg {
                TracksMenuOutput::TrackChanged(index) => PlayerInput::TextTrackChanged(index),
            },
        );

        let audio_tracks_menu =
            TracksMenu::builder()
                .launch("sound-wave")
                .forward(sender.input_sender(), |msg| match msg {
                    TracksMenuOutput::TrackChanged(index) => PlayerInput::AudioTrackChanged(index),
                });

        let model = Player {
            video,
            immersed: false,
            fullscreen: false,
            start_time: false,
            mouse_position: (0.0, 0.0),
            immersed_timeout: None,
            hovering_controls: false,
            seekbar: seekbar.to_owned(),
            volume: volume.to_owned(),
            text_tracks_menu,
            audio_tracks_menu,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn pre_view() {
        let player = PLAYER_STATE.read_inner();
        let state = VIDEO_STATE.read_inner();
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            PlayerInput::Load(stream) => {
                models::player::load(*stream);
            }
            PlayerInput::Unload => {
                models::player::unload();
                self.video.emit(VideoInput::Unload);
                self.start_time = false;
            }
            PlayerInput::UpdateVideo => {
                let ctx = CTX_STATE.read_inner();
                let player = PLAYER_STATE.read_inner();
                let video = VIDEO_STATE.read_inner();

                if !video.loaded {
                    if let Some(uri) = &player.uri {
                        self.video.emit(VideoInput::Load(uri.to_string()));
                    }
                }

                let index = (ctx.settings.subtitles_size / SUBTITLES_MIN_SIZE) - 1;
                if let Some(size) = SUBTITLES_FONT_SIZES.get(index as usize) {
                    self.video.emit(VideoInput::SubtitlesSize(*size));
                }
            }
            PlayerInput::UpdateView => {
                let video = VIDEO_STATE.read_inner();
                let player = PLAYER_STATE.read_inner();

                if video.ready && video.loaded && !self.start_time {
                    self.start_time = true;
                    self.video.emit(VideoInput::Seek(player.time));
                }

                self.text_tracks_menu
                    .emit(TracksMenuInput::Update(video.text_tracks.to_owned()));
                self.audio_tracks_menu
                    .emit(TracksMenuInput::Update(video.audio_tracks.to_owned()));
            }
            PlayerInput::MouseMove(position) => {
                if self.mouse_position != position {
                    self.mouse_position = position;
                    self.cancel_immersed_timeout();
                    self.immersed = false;

                    let state = VIDEO_STATE.read_inner();
                    if !state.paused && !self.hovering_controls {
                        let timeout_sender = sender.clone();
                        self.create_immersed_timeout(timeout_sender);
                    }
                }
            }
            PlayerInput::MouseEnterControls => {
                self.cancel_immersed_timeout();
                self.hovering_controls = true;
            }
            PlayerInput::MouseLeaveControls => {
                self.hovering_controls = false;
            }
            PlayerInput::Immersed => {
                let state = VIDEO_STATE.read_inner();
                if !state.paused {
                    self.immersed = true;
                }
            }
            PlayerInput::PlayPause => {
                let state = VIDEO_STATE.read_inner();
                if state.paused {
                    self.video.emit(VideoInput::Play);
                } else {
                    self.video.emit(VideoInput::Pause);
                }
            }
            PlayerInput::Seek => {
                let time = self.seekbar.value();
                let state = VIDEO_STATE.read_inner();

                self.video.emit(VideoInput::Seek(time));
                models::player::update_seek_time(time, state.duration);
            }
            PlayerInput::SeekPrev => {
                let state = VIDEO_STATE.read_inner();
                let time = state.time - 10000.0;

                self.video.emit(VideoInput::Seek(time));
                models::player::update_seek_time(time, state.duration);
            }
            PlayerInput::SeekNext => {
                let state = VIDEO_STATE.read_inner();
                let time = state.time + 10000.0;

                self.video.emit(VideoInput::Seek(time));
                models::player::update_seek_time(time, state.duration);
            }
            PlayerInput::Volume => {
                let volume = self.volume.value();
                self.video.emit(VideoInput::Volume(volume));
            }
            PlayerInput::TextTrackChanged(index) => {
                self.video.emit(VideoInput::TextTrack(index));
            }
            PlayerInput::AudioTrackChanged(index) => {
                self.video.emit(VideoInput::AudioTrack(index));
            }
            PlayerInput::Fullscreen => {
                if let Some(window) = relm4::main_application().active_window() {
                    self.fullscreen = !window.is_fullscreen();
                    window.set_fullscreened(self.fullscreen);
                }
            }
            PlayerInput::PauseChanged(paused) => {
                models::player::update_paused(paused);
            }
            PlayerInput::TimeChanged(time, duration) => {
                models::player::update_time(time, duration);
            }
            PlayerInput::Ended => {
                APP_BROKER.send(AppMsg::NavigateBack);
            }
            PlayerInput::Error => {
                let message = t!("error_player").to_string();
                APP_BROKER.send(AppMsg::Toast(message));
            }
        }
    }

    fn shutdown(&mut self, _widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
        self.cancel_immersed_timeout();
    }
}

impl Player {
    fn create_immersed_timeout(&mut self, sender: ComponentSender<Self>) {
        let task = tokio::spawn(async move {
            sleep(Duration::from_secs(1)).await;
            sender.input_sender().emit(PlayerInput::Immersed);
        });

        self.immersed_timeout = Some(task);
    }

    fn cancel_immersed_timeout(&mut self) {
        if let Some(task) = self.immersed_timeout.take() {
            task.abort();
        }
    }

    fn ms_to_clock(ms: f64) -> String {
        let total_seconds = (ms / 1000.0).round() as u64;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}
