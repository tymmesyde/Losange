use std::time::Duration;

use gst::prelude::*;
use gtk::{gdk, glib};
use itertools::Itertools;
use relm4::{gtk, ComponentParts, ComponentSender, RelmWidgetExt, SharedState, SimpleComponent};
use serde::Serialize;
use tracing::error;

const MS_IN_NS: i64 = 1000000;

pub enum MediaTrackType {
    Text,
    Audio,
}

#[derive(Debug, Clone, Serialize)]
pub struct MediaTrack {
    pub id: i32,
    pub lang: String,
    pub label: Option<String>,
    pub active: bool,
}

#[derive(Default, Debug)]
pub struct VideoState {
    pub loaded: bool,
    pub paused: bool,
    pub volume: f64,
    pub duration: f64,
    pub time: f64,
    pub buffering: bool,
    pub text_tracks: Vec<MediaTrack>,
    pub audio_tracks: Vec<MediaTrack>,
}

pub static VIDEO_STATE: SharedState<VideoState> = SharedState::new();

#[derive(Debug)]
pub enum VideoInput {
    StateChanged,
    PropertyChanged(String),
    TimeChanged,
    Buffering(i32),
    Load(String),
    Unload,
    Play,
    Pause,
    Seek(f64),
    Volume(f64),
    TextTrack(i32),
    AudioTrack(i32),
}

#[derive(Debug)]
pub enum VideoOutput {
    PauseChanged(bool),
    TimeChanged(f64, f64),
    Ended,
    Error,
}

pub struct Video {
    paintable: gdk::Paintable,
    playbin: gst::Element,
    pipeline: gst::Pipeline,
    bus_watch: Option<gst::bus::BusWatchGuard>,
    timeout_id: Option<glib::SourceId>,
}

#[relm4::component(pub)]
impl SimpleComponent for Video {
    type Init = ();
    type Input = VideoInput;
    type Output = VideoOutput;

    view! {
        gtk::Picture {
            set_expand: true,
            set_paintable: Some(&model.paintable),
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let (paintable, playbin, pipeline) = Self::init().expect("Failed to initialize player");

        let model = Video {
            paintable,
            playbin,
            pipeline,
            bus_watch: None,
            timeout_id: None,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            VideoInput::StateChanged => {
                let mut state = VIDEO_STATE.write();
                state.paused = self.get_paused();
                state.duration = self.get_duration();
                state.text_tracks = self.get_media_tracks(MediaTrackType::Text);
                state.audio_tracks = self.get_media_tracks(MediaTrackType::Audio);
            }
            VideoInput::TimeChanged => {
                if !self.get_paused() {
                    let mut state = VIDEO_STATE.write();
                    state.time = self.get_time();
                    sender
                        .output_sender()
                        .emit(VideoOutput::TimeChanged(state.time, state.duration));
                }
            }
            VideoInput::PropertyChanged(name) => {
                let mut state = VIDEO_STATE.write();
                match name.as_str() {
                    "volume" => {
                        state.volume = self.get_volume();
                    }
                    "current-text" => {
                        state.text_tracks = self.get_media_tracks(MediaTrackType::Text);
                    }
                    "current-audio" => {
                        state.audio_tracks = self.get_media_tracks(MediaTrackType::Audio);
                    }
                    _ => {}
                }
            }
            VideoInput::Buffering(percent) => {
                let mut state = VIDEO_STATE.write();
                state.buffering = percent < 100;
            }
            VideoInput::Load(uri) => {
                let mut state = VIDEO_STATE.write();
                state.loaded = true;
                state.buffering = true;

                self.start(&uri, sender);
            }
            VideoInput::Unload => {
                let mut state = VIDEO_STATE.write();
                state.loaded = false;

                self.stop();
            }
            VideoInput::Play => {
                self.play();
                sender
                    .output_sender()
                    .emit(VideoOutput::PauseChanged(false));
            }
            VideoInput::Pause => {
                self.pause();
                sender.output_sender().emit(VideoOutput::PauseChanged(true));
            }
            VideoInput::Seek(time) => {
                self.set_time(time);
            }
            VideoInput::Volume(volume) => {
                self.set_volume(volume);
            }
            VideoInput::TextTrack(id) => {
                self.set_text_track(id);
            }
            VideoInput::AudioTrack(id) => {
                self.set_audio_track(id);
            }
        }
    }

    fn shutdown(&mut self, _widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
        self.stop();
    }
}

impl Video {
    fn init() -> Result<(gdk::Paintable, gst::Element, gst::Pipeline), glib::BoolError> {
        let gtksink = gst::ElementFactory::make("gtk4paintablesink").build()?;

        let paintable = gtksink.property::<gtk::gdk::Paintable>("paintable");

        let sink = match paintable.property::<Option<gdk::GLContext>>("gl-context") {
            Some(_) => gst::ElementFactory::make("glsinkbin")
                .property("sink", &gtksink)
                .build()?,
            None => {
                let sink = gst::Bin::default();
                let convert = gst::ElementFactory::make("videoconvert").build()?;

                sink.add(&convert)?;
                sink.add(&gtksink)?;
                convert.link(&gtksink)?;

                sink.add_pad(&gst::GhostPad::with_target(
                    &convert.static_pad("sink").unwrap(),
                )?)?;

                sink.upcast()
            }
        };

        let playbin = gst::ElementFactory::make("playbin").build()?;
        playbin.set_property("video-sink", sink);
        playbin.add_property_notify_watch(None, false);
        playbin.set_property("subtitle-font-desc", "Cantarell");

        let pipeline = gst::Pipeline::new();
        pipeline.add(&playbin)?;

        Ok((paintable, playbin, pipeline))
    }

    fn register_bus_watch(&mut self, sender: ComponentSender<Self>) {
        if let Some(bus) = self.pipeline.bus() {
            let bus_watch_handler = move |_: &gst::Bus, msg: &gst::Message| {
                match msg.view() {
                    gst::MessageView::Buffering(buffering) => sender
                        .input_sender()
                        .emit(VideoInput::Buffering(buffering.percent())),
                    gst::MessageView::StateChanged(..) => {
                        sender.input_sender().emit(VideoInput::StateChanged)
                    }
                    gst::MessageView::PropertyNotify(message) => {
                        let (_, name, _) = message.get();
                        sender
                            .input_sender()
                            .emit(VideoInput::PropertyChanged(name.to_string()))
                    }
                    gst::MessageView::Eos(..) => sender.output_sender().emit(VideoOutput::Ended),
                    gst::MessageView::Error(e) => {
                        error!("{e}");
                        sender.output_sender().emit(VideoOutput::Error)
                    }
                    _ => {}
                };

                glib::ControlFlow::Continue
            };

            if let Ok(bus_watch) = bus.add_watch_local(bus_watch_handler) {
                self.bus_watch = Some(bus_watch);
            }
        }
    }

    fn register_time_timeout(&mut self, sender: ComponentSender<Self>) {
        self.timeout_id = Some(glib::timeout_add_local(
            Duration::from_millis(500),
            move || {
                sender.input_sender().emit(VideoInput::TimeChanged);

                glib::ControlFlow::Continue
            },
        ))
    }

    fn start(&mut self, uri: &str, sender: ComponentSender<Self>) {
        self.playbin.set_property("uri", uri);
        self.play();

        let bus_watch_sender = sender.clone();
        self.register_bus_watch(bus_watch_sender);

        let time_timeout = sender.clone();
        self.register_time_timeout(time_timeout);
    }

    fn stop(&mut self) {
        if let Err(e) = self.pipeline.set_state(gst::State::Null) {
            error!("Failed to set the pipeline to `Null` state: {e}");
        }

        drop(self.bus_watch.take());

        if let Some(timeout_id) = self.timeout_id.take() {
            timeout_id.remove();
        }
    }

    fn play(&self) {
        if let Err(e) = self.pipeline.set_state(gst::State::Playing) {
            error!("Failed to set the pipeline to `Playing` state: {e}");
        }
    }

    fn pause(&self) {
        if let Err(e) = self.pipeline.set_state(gst::State::Paused) {
            error!("Failed to set the pipeline to `Paused` state: {e}");
        }
    }

    fn set_time(&self, time: f64) {
        if let Err(e) = self.playbin.seek_simple(
            gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
            gst::ClockTime::from_mseconds(time as u64),
        ) {
            error!("Failed to seek: {e}");
        }
    }

    fn set_volume(&self, volume: f64) {
        let value = volume / 100.0;
        self.playbin.set_property("volume", value);
    }

    fn set_text_track(&self, id: i32) {
        self.playbin.set_property("current-text", id);
    }

    fn set_audio_track(&self, id: i32) {
        self.playbin.set_property("current-audio", id);
    }

    fn get_paused(&self) -> bool {
        !matches!(self.playbin.current_state(), gst::State::Playing)
    }

    fn get_duration(&self) -> f64 {
        let duration_value = self.playbin.query_duration_generic(gst::Format::Time);
        match duration_value {
            Some(duration) => (duration.value() / MS_IN_NS) as f64,
            None => 0.0,
        }
    }

    fn get_time(&self) -> f64 {
        let position_value = self.playbin.query_position_generic(gst::Format::Time);
        match position_value {
            Some(position) => (position.value() / MS_IN_NS) as f64,
            None => 0.0,
        }
    }

    fn get_volume(&self) -> f64 {
        (self.playbin.property::<f64>("volume") * 100.0).round()
    }

    fn get_media_tracks(&self, track_type: MediaTrackType) -> Vec<MediaTrack> {
        let (count_property, current_property) = match track_type {
            MediaTrackType::Text => ("n-text", "current-text"),
            MediaTrackType::Audio => ("n-audio", "current-audio"),
        };

        let tracks_count = self.playbin.property::<i32>(count_property);
        let active_track = self.playbin.property::<i32>(current_property);

        (0..tracks_count)
            .map(|track_id| {
                let lang = self.get_track_lang(&track_type, track_id);
                let label = rust_iso639::from_code_1(&lang).map(|lang| lang.name.to_owned());

                MediaTrack {
                    id: track_id,
                    lang,
                    label,
                    active: active_track == track_id,
                }
            })
            .collect_vec()
    }

    fn get_track_lang(&self, track_type: &MediaTrackType, track_id: i32) -> String {
        let signal = match track_type {
            MediaTrackType::Text => "get-text-tags",
            MediaTrackType::Audio => "get-audio-tags",
        };

        self.playbin
            .emit_by_name::<Option<gst::TagList>>(signal, &[&track_id])
            .and_then(|tags| tags.get::<gst::tags::LanguageCode>())
            .map_or("und".to_owned(), |code| code.get().to_owned())
    }
}
