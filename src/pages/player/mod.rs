mod tracks_menu;
mod video;

use std::time::Duration;

use adw::prelude::*;
use gtk::glib;
use relm4::{
    actions::{ActionGroupName, ActionName, RelmAction, RelmActionGroup},
    adw, css,
    gtk::{self, gio},
    Component, ComponentController, ComponentParts, ComponentSender, Controller, JoinHandle,
    RelmWidgetExt, SimpleComponent,
};
use rust_i18n::t;
use stremio_core_losange::{
    models::{self, ctx::CTX_STATE, player::PLAYER_STATE, server::SERVER_STATE},
    stremio_core::types::streams::{AudioTrack, SubtitleTrack},
    types::stream::Stream,
};
use tokio::time::sleep;
use tracks_menu::{TracksMenu, TracksMenuInput, TracksMenuOutput};
use video::{Video, VideoInput, VideoOutput, VIDEO_STATE};

use crate::{
    app::AppMsg,
    common::window::WindowExt,
    components::spinner::Spinner,
    constants::{APP_ID, VOLUME_DEFAULT, VOLUME_MAX, VOLUME_STEP},
    APP_BROKER,
};

relm4::new_action_group!(pub(super) PlayerActionGroup, "player");
relm4::new_stateless_action!(pub(super) PlayPauseAction, PlayerActionGroup, "play_pause");
relm4::new_stateless_action!(pub(super) SeekPrevAction, PlayerActionGroup, "seek_prev");
relm4::new_stateless_action!(pub(super) SeekNextAction, PlayerActionGroup, "seek_next");
relm4::new_stateless_action!(pub(super) VolumeUp, PlayerActionGroup, "volume_up");
relm4::new_stateless_action!(pub(super) VolumeDown, PlayerActionGroup, "volume_down");
relm4::new_stateless_action!(pub(super) ToggleFullscreen, PlayerActionGroup, "toggle_fullscreen");
relm4::new_stateless_action!(pub(super) Exit, PlayerActionGroup, "exit");

const SHORTCUTS: &[(&str, &str, &str)] = &[
    ("space", PlayerActionGroup::NAME, PlayPauseAction::NAME),
    ("Left", PlayerActionGroup::NAME, SeekPrevAction::NAME),
    ("Right", PlayerActionGroup::NAME, SeekNextAction::NAME),
    ("Up", PlayerActionGroup::NAME, VolumeUp::NAME),
    ("Down", PlayerActionGroup::NAME, VolumeDown::NAME),
    ("F", PlayerActionGroup::NAME, ToggleFullscreen::NAME),
    ("Escape", PlayerActionGroup::NAME, Exit::NAME),
];

#[derive(Debug)]
pub enum PlayerInput {
    Load(Box<Stream>),
    Unload,
    UpdateVideo,
    MouseMove((f64, f64)),
    MouseEnterControls,
    MouseLeaveControls,
    Immersed,
    PlayPause,
    PlayNext,
    Seek,
    SeekPrev,
    SeekNext,
    Volume(f64),
    TextTrackChanged(i64),
    AudioTrackChanged(i64),
    Fullscreen,
    Exit,
    PauseChanged(bool),
    TimeChanged(f64, f64),
    TracksChanged,
    SizeChanged((i64, i64)),
    Ended,
    Error,
}

pub struct Player {
    settings: gio::Settings,
    video: Controller<Video>,
    immersed: bool,
    fullscreen: bool,
    mouse_position: (f64, f64),
    immersed_timeout: Option<JoinHandle<()>>,
    hovering_controls: bool,
    seekbar: gtk::Scale,
    volume: gtk::ScaleButton,
    text_tracks_menu: Controller<TracksMenu>,
    audio_tracks_menu: Controller<TracksMenu>,
    statistics_task: Option<JoinHandle<()>>,
    default_window_size: Option<(i32, i32)>,
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

            gtk::WindowHandle {
                add_css_class: relm4::css::classes::OSD,
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
                                add_css_class: css::classes::HEADING,

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

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 18,

                            #[template]
                            Spinner {},

                            adw::Clamp {
                                set_maximum_size: 100,

                                #[watch]
                                set_visible: player.torrent_info.is_some(),

                                match server.torrent_progress {
                                    Some(progress) => gtk::ProgressBar {
                                        #[watch]
                                        set_fraction: progress / 100.0,
                                    },
                                    None => gtk::Box {
                                        set_visible: false,
                                    },
                                },
                            }
                        }
                    },

                    add_overlay = &gtk::Revealer {
                        set_valign: gtk::Align::End,
                        set_transition_type: gtk::RevealerTransitionType::Crossfade,

                        #[watch]
                        set_reveal_child: state.paused || !model.immersed,

                        gtk::Box {
                            add_css_class: relm4::css::classes::TOOLBAR,
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
                                set_size_request: (45, 45),

                                #[watch]
                                set_icon_name: match state.paused {
                                    true => "media-playback-start-symbolic",
                                    false => "media-playback-pause-symbolic",
                                },

                                connect_clicked => PlayerInput::PlayPause,
                            },

                            gtk::Button {
                                set_size_request: (45, 45),
                                set_icon_name: "skip-forward-large",

                                #[watch]
                                set_visible: player.next_stream.is_some(),

                                connect_clicked => PlayerInput::PlayNext,
                            },

                            #[local_ref]
                            volume -> gtk::ScaleButton {
                                set_size_request: (45, 45),

                                set_icons: &[
                                    "audio-volume-muted",
                                    "audio-volume-high",
                                    "audio-volume-low",
                                    "audio-volume-medium",
                                ],

                                set_adjustment: &gtk::Adjustment::new(
                                    VOLUME_DEFAULT,
                                    0.0,
                                    VOLUME_MAX,
                                    VOLUME_STEP,
                                    VOLUME_STEP,
                                    0.0,
                                ),

                                #[watch]
                                #[block_signal(volume_handler)]
                                set_value: state.volume,

                                connect_value_changed[sender] => move |_, _| {
                                    sender.input(PlayerInput::Volume(0.0));
                                } @volume_handler,
                            },

                            gtk::Label {
                                add_css_class: css::classes::HEADING,
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
                                add_css_class: css::classes::HEADING,
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
                                set_size_request: (45, 45),
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
        let server = SERVER_STATE.read_inner();
        let state = VIDEO_STATE.read_inner();

        CTX_STATE.subscribe(sender.input_sender(), |_| PlayerInput::UpdateVideo);
        PLAYER_STATE.subscribe(sender.input_sender(), |_| PlayerInput::UpdateVideo);

        let settings = gio::Settings::new(APP_ID);

        let video = Video::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| match msg {
                VideoOutput::PauseChanged(paused) => PlayerInput::PauseChanged(paused),
                VideoOutput::TimeChanged(time, duration) => {
                    PlayerInput::TimeChanged(time, duration)
                }
                VideoOutput::TracksChanged => PlayerInput::TracksChanged,
                VideoOutput::SizeChanged(size) => PlayerInput::SizeChanged(size),
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
            settings,
            video,
            immersed: false,
            fullscreen: false,
            mouse_position: (0.0, 0.0),
            immersed_timeout: None,
            hovering_controls: false,
            seekbar: seekbar.to_owned(),
            volume: volume.to_owned(),
            text_tracks_menu,
            audio_tracks_menu,
            statistics_task: None,
            default_window_size: None,
        };

        let play_pause_action = {
            let sender = sender.input_sender().clone();
            RelmAction::<PlayPauseAction>::new_stateless(move |_| {
                sender.emit(PlayerInput::PlayPause);
            })
        };

        let seek_prev_action = {
            let sender = sender.input_sender().clone();
            RelmAction::<SeekPrevAction>::new_stateless(move |_| {
                sender.emit(PlayerInput::SeekPrev);
            })
        };

        let seek_next_action = {
            let sender = sender.input_sender().clone();
            RelmAction::<SeekNextAction>::new_stateless(move |_| {
                sender.emit(PlayerInput::SeekNext);
            })
        };

        let volume_up_action = {
            let sender = sender.input_sender().clone();
            RelmAction::<VolumeUp>::new_stateless(move |_| {
                sender.emit(PlayerInput::Volume(VOLUME_STEP));
            })
        };

        let volume_down_action = {
            let sender = sender.input_sender().clone();
            RelmAction::<VolumeDown>::new_stateless(move |_| {
                sender.emit(PlayerInput::Volume(-VOLUME_STEP));
            })
        };

        let toggle_fullscreen_action = {
            let sender = sender.input_sender().clone();
            RelmAction::<ToggleFullscreen>::new_stateless(move |_| {
                sender.emit(PlayerInput::Fullscreen);
            })
        };

        let exit_action = {
            let sender = sender.input_sender().clone();
            RelmAction::<Exit>::new_stateless(move |_| {
                sender.emit(PlayerInput::Exit);
            })
        };

        let mut actions = RelmActionGroup::<PlayerActionGroup>::new();
        actions.add_action(play_pause_action);
        actions.add_action(seek_prev_action);
        actions.add_action(seek_next_action);
        actions.add_action(volume_up_action);
        actions.add_action(volume_down_action);
        actions.add_action(toggle_fullscreen_action);
        actions.add_action(exit_action);

        if let Some(window) = relm4::main_application().active_window() {
            actions.register_for_widget(window);
        }

        let shortcut_controller = gtk::ShortcutController::new();

        for (key, group, action) in SHORTCUTS {
            shortcut_controller.add_shortcut({
                let named_action = gtk::NamedAction::new(&format!("{group}.{action}"));
                let trigger = gtk::ShortcutTrigger::parse_string(key).unwrap();

                gtk::Shortcut::builder()
                    .action(&named_action)
                    .trigger(&trigger)
                    .build()
            });
        }

        root.add_controller(shortcut_controller);

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn pre_view() {
        let player = PLAYER_STATE.read_inner();
        let server = SERVER_STATE.read_inner();
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

                if self.settings.boolean("player-resize-window") {
                    if let Some((width, height)) = self.default_window_size.take() {
                        if let Some(window) = relm4::main_application().active_window() {
                            window.animate_size(width, height);
                        }
                    }
                }
            }
            PlayerInput::UpdateVideo => {
                let ctx = CTX_STATE.read_inner();
                let player = PLAYER_STATE.read_inner();
                let video = VIDEO_STATE.read_inner();

                self.cancel_statistics_task();

                if let Some((info_hash, file_idx)) = &player.torrent_info {
                    let info_hash = info_hash.clone();
                    let file_idx = *file_idx;

                    self.create_statistics_task(info_hash, file_idx);
                }

                if !video.loaded {
                    if let Some(uri) = &player.uri {
                        self.video
                            .emit(VideoInput::Load((uri.to_string(), player.time)));
                    }
                }

                let size = ctx.settings.subtitles_size as f64 / 100.0;
                self.video.emit(VideoInput::SubtitlesSize(size));

                let position = 100.0 - ctx.settings.subtitles_offset as f64;
                self.video.emit(VideoInput::SubtitlesPosition(position));

                let color = ctx.settings.subtitles_text_color.to_owned();
                self.video.emit(VideoInput::SubtitlesColor(color));

                let outline_color = ctx.settings.subtitles_outline_color.to_owned();
                self.video
                    .emit(VideoInput::SubtitlesOutlineColor(outline_color));

                APP_BROKER.sender().emit(AppMsg::MediaMetadata((
                    player.title.to_owned(),
                    player.image.to_owned(),
                )));
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
            PlayerInput::PlayNext => {
                let player = PLAYER_STATE.read_inner();

                if let Some(stream) = &player.next_stream {
                    sender.input_sender().emit(PlayerInput::Unload);
                    sender
                        .input_sender()
                        .emit(PlayerInput::Load(Box::new(stream.to_owned())));
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
            PlayerInput::Volume(amount) => {
                let volume = (self.volume.value() + amount).clamp(0.0, VOLUME_MAX);

                if amount != 0.0 {
                    let message = format!("{} {}%", &t!("volume"), volume);
                    APP_BROKER.send(AppMsg::Toast((message, 1)));
                }

                self.video.emit(VideoInput::Volume(volume));
            }
            PlayerInput::TextTrackChanged(id) => {
                self.video.emit(VideoInput::TextTrack(id));

                models::player::update_stream_state(|mut settings| {
                    settings.subtitle_track = Some(SubtitleTrack {
                        id: id.to_string(),
                        embedded: true,
                        language: None,
                    });
                    settings
                });
            }
            PlayerInput::AudioTrackChanged(id) => {
                self.video.emit(VideoInput::AudioTrack(id));

                models::player::update_stream_state(|mut settings| {
                    settings.audio_track = Some(AudioTrack {
                        id: id.to_string(),
                        language: None,
                    });
                    settings
                });
            }
            PlayerInput::Fullscreen => {
                if let Some(window) = relm4::main_application().active_window() {
                    self.fullscreen = !window.is_fullscreen();
                    window.set_fullscreened(self.fullscreen);
                }
            }
            PlayerInput::Exit => {
                if let Some(window) = relm4::main_application().active_window() {
                    if window.is_fullscreen() {
                        window.set_fullscreened(false);
                    } else {
                        APP_BROKER.send(AppMsg::NavigateBack);
                    }
                }
            }
            PlayerInput::PauseChanged(paused) => {
                models::player::update_paused(paused);
                APP_BROKER.sender().emit(AppMsg::MediaStatus(paused));
            }
            PlayerInput::TimeChanged(time, duration) => {
                models::player::update_time(time, duration);
            }
            PlayerInput::TracksChanged => {
                let player = PLAYER_STATE.read_inner();
                let video = VIDEO_STATE.read_inner();

                self.text_tracks_menu
                    .emit(TracksMenuInput::Update(video.text_tracks.to_owned()));
                self.audio_tracks_menu
                    .emit(TracksMenuInput::Update(video.audio_tracks.to_owned()));

                if let Some(state) = &player.stream_state {
                    if let Some(track) = &state.subtitle_track {
                        if let Ok(id) = track.id.parse::<i64>() {
                            self.video.emit(VideoInput::TextTrack(id));
                        }
                    }

                    if let Some(track) = &state.audio_track {
                        if let Ok(id) = track.id.parse::<i64>() {
                            self.video.emit(VideoInput::AudioTrack(id));
                        }
                    }
                }
            }
            PlayerInput::SizeChanged((video_width, video_height)) => {
                if self.settings.boolean("player-resize-window")
                    && video_width > 0
                    && video_height > 0
                {
                    if let Some(window) = relm4::main_application().active_window() {
                        self.default_window_size = Some(window.default_size());
                        window.resize_to_aspect_ratio(video_width as f64 / video_height as f64);
                    }
                }
            }
            PlayerInput::Ended => {
                let ctx = CTX_STATE.read_inner();
                let player = PLAYER_STATE.read_inner();

                match (&player.next_stream, ctx.settings.binge_watching) {
                    (Some(stream), true) => {
                        sender.input_sender().emit(PlayerInput::Unload);
                        sender
                            .input_sender()
                            .emit(PlayerInput::Load(Box::new(stream.to_owned())));
                    }
                    _ => APP_BROKER.send(AppMsg::NavigateBack),
                }
            }
            PlayerInput::Error => {
                let message = t!("error_player").to_string();
                APP_BROKER.send(AppMsg::Toast((message, 3)));
            }
        }
    }

    fn shutdown(&mut self, _widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
        self.cancel_immersed_timeout();
        self.cancel_statistics_task();
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

    fn create_statistics_task(&mut self, info_hash: String, file_idx: u16) {
        let task = tokio::spawn(async move {
            loop {
                models::server::update_statistics(&info_hash, file_idx);
                sleep(Duration::from_millis(800)).await;
            }
        });

        self.statistics_task = Some(task);
    }

    fn cancel_statistics_task(&mut self) {
        if let Some(task) = self.statistics_task.take() {
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
