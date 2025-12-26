use std::{cell::RefCell, env, os::raw::c_void, rc::Rc};

use gtk::{gdk::GLContext, glib, prelude::*};
use itertools::Itertools;
use libc::{setlocale, LC_NUMERIC};
use libmpv2::{
    events::{Event, PropertyData},
    render::{OpenGLInitParams, RenderContext, RenderParam, RenderParamApiType},
    Format, Mpv,
};
use relm4::{gtk, ComponentParts, ComponentSender, RelmWidgetExt, SharedState, SimpleComponent};
use serde::{Deserialize, Serialize};
use tracing::error;

fn get_proc_address(_context: &GLContext, name: &str) -> *mut c_void {
    epoxy::get_proc_addr(name) as _
}

const SECOND: f64 = 1000.0;

#[derive(Debug, Deserialize)]
struct Track {
    id: i64,
    r#type: String,
    lang: Option<String>,
    selected: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct MediaTrack {
    pub id: i64,
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
    Update,
    Load((String, f64)),
    Unload,
    Play,
    Pause,
    Seek(f64),
    Volume(f64),
    TextTrack(i64),
    AudioTrack(i64),
    SubtitlesSize(f64),
    SubtitlesPosition(f64),
}

#[derive(Debug)]
pub enum VideoOutput {
    PauseChanged(bool),
    TimeChanged(f64, f64),
    TracksChanged,
    Ended,
    Error,
}

pub struct Video {
    mpv: Rc<RefCell<Mpv>>,
    render_context: Rc<RefCell<Option<RenderContext>>>,
    events_source: Option<glib::SourceId>,
    gl_area: gtk::GLArea,
}

#[relm4::component(pub)]
impl SimpleComponent for Video {
    type Init = ();
    type Input = VideoInput;
    type Output = VideoOutput;

    view! {
        gtk::Box {
            #[local_ref]
            gl_area -> gtk::GLArea {
                set_expand: true,

                connect_realize[sender] => move |gl_area| {
                    gl_area.make_current();

                    if gl_area.error().is_some() {
                        return;
                    }

                    if let Some(context) = gl_area.context() {
                        let mut mpv = mpv.borrow_mut();
                        let mpv_handle = unsafe { mpv.ctx.as_mut() };

                        let render_context_result = RenderContext::new(
                            mpv_handle,
                            vec![
                                RenderParam::ApiType(RenderParamApiType::OpenGl),
                                RenderParam::InitParams(OpenGLInitParams {
                                    get_proc_address,
                                    ctx: context,
                                }),
                                RenderParam::BlockForTargetTime(false),
                            ],
                        );

                        match render_context_result {
                            Ok(mut render_context) => {
                                let sender = sender.clone();
                                render_context.set_update_callback(move || {
                                    sender.input_sender().emit(VideoInput::Update);
                                });

                                *render_context_realize.borrow_mut() = Some(render_context);
                            }
                            Err(e) => {
                                error!("Failed to create render context: {}", e);
                            }
                        }
                    }
                },

                connect_unrealize[sender] => move |_| {
                    if let Some(render_context) = render_context_unrealize.borrow_mut().take() {
                        drop(render_context);
                    }

                    sender.input_sender().emit(VideoInput::Unload);
                },

                connect_render => move |gl_area, _| {
                    let mut fbo = 0;
                    unsafe {
                        epoxy::GetIntegerv(epoxy::FRAMEBUFFER_BINDING, &mut fbo);
                    }

                    let width = gl_area.width();
                    let height = gl_area.height();
                    let scale_factor = gl_area.native()
                        .and_then(|native| native.surface())
                        .map(|surface| surface.scale_factor())
                        .unwrap_or(1);

                    if let Some(ref render_context) = *render_context_render.borrow() {
                        if let Err(e) = render_context.render::<GLContext>(fbo, width * scale_factor, height * scale_factor, true) {
                            error!("Failed to render frame: {}", e);
                            return glib::Propagation::Stop;
                        }
                    }

                    glib::Propagation::Proceed
                },
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mpv = Self::create_mpv().expect("Failed to initialize player");
        let gl_area = gtk::GLArea::default();

        let model = Video {
            mpv: Rc::new(RefCell::new(mpv)),
            render_context: Rc::new(RefCell::new(None)),
            events_source: None,
            gl_area,
        };

        let gl_area = &model.gl_area;
        let mpv = model.mpv.clone();
        let render_context_realize = model.render_context.clone();
        let render_context_unrealize = model.render_context.clone();
        let render_context_render = model.render_context.clone();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            VideoInput::Update => {
                self.gl_area.queue_render();
            }
            VideoInput::Load((uri, start_time)) => {
                let mut state = VIDEO_STATE.write();
                state.loaded = true;
                state.buffering = true;
                drop(state);

                self.start(&uri, start_time, sender);
            }
            VideoInput::Unload => {
                let mut state = VIDEO_STATE.write();
                state.loaded = false;

                self.stop();
            }
            VideoInput::Play => {
                self.play();
            }
            VideoInput::Pause => {
                self.pause();
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
            VideoInput::SubtitlesSize(size) => {
                self.set_subtitles_size(size);
            }
            VideoInput::SubtitlesPosition(position) => {
                self.set_subtitles_position(position);
            }
        }
    }

    fn shutdown(&mut self, _widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
        self.stop();
    }
}

impl Video {
    fn create_mpv() -> anyhow::Result<Mpv> {
        // Required for libmpv to work alongside gtk
        unsafe {
            setlocale(LC_NUMERIC, c"C".as_ptr());
        }

        let log = env::var("RUST_LOG");
        let msg_level = match log {
            Ok(scope) => &format!("all={}", scope.as_str()),
            _ => "all=no",
        };

        let mpv = Mpv::with_initializer(|init| {
            init.set_property("vo", "libmpv")?;
            init.set_property("video-timing-offset", "0")?;
            init.set_property("terminal", "yes")?;
            init.set_property("msg-level", msg_level)?;
            Ok(())
        })
        .expect("Failed to create mpv");

        mpv.disable_deprecated_events().ok();

        mpv.observe_property("pause", Format::Flag, 0).ok();
        mpv.observe_property("time-pos", Format::Double, 0).ok();
        mpv.observe_property("duration", Format::Double, 0).ok();
        mpv.observe_property("volume", Format::Double, 0).ok();
        mpv.observe_property("cache-buffering-state", Format::Int64, 0)
            .ok();
        mpv.observe_property("track-list", Format::String, 0).ok();

        Ok(mpv)
    }

    fn watch_events(&mut self, sender: ComponentSender<Self>) {
        let mpv = self.mpv.clone();

        self.events_source = Some(glib::idle_add_local(move || {
            if let Some(Ok(event)) = mpv.borrow_mut().wait_event(0.0) {
                match event {
                    Event::PropertyChange { name, change, .. } => {
                        let mut state = VIDEO_STATE.write();

                        match change {
                            PropertyData::Flag(value) if name == "pause" => {
                                state.paused = value;
                                sender.output_sender().emit(VideoOutput::PauseChanged(true));
                            }
                            PropertyData::Double(value) => match name {
                                "time-pos" => {
                                    state.time = value * SECOND;
                                    sender
                                        .output_sender()
                                        .emit(VideoOutput::TimeChanged(state.time, state.duration));
                                }
                                "duration" => state.duration = value * SECOND,
                                "volume" => state.volume = value,
                                _ => {}
                            },
                            PropertyData::Int64(value) if name == "cache-buffering-state" => {
                                state.buffering = value < 100;
                            }
                            PropertyData::Str(value) if name == "track-list" => {
                                if let Ok(list) = serde_json::from_str::<Vec<Track>>(value) {
                                    let (text, audio) = Self::create_media_tracks(list);
                                    state.text_tracks = text;
                                    state.audio_tracks = audio;
                                    sender.output_sender().emit(VideoOutput::TracksChanged);
                                }
                            }
                            _ => {}
                        }
                    }
                    Event::EndFile(reason) => {
                        if reason == 4 {
                            sender.output_sender().emit(VideoOutput::Error);
                        }

                        if reason == 0 {
                            sender.output_sender().emit(VideoOutput::Ended);
                        }
                    }
                    _ => {}
                }
            }

            glib::ControlFlow::Continue
        }));
    }

    fn create_media_tracks(list: Vec<Track>) -> (Vec<MediaTrack>, Vec<MediaTrack>) {
        let media_tracks = |r#type: &str| {
            list.iter()
                .filter(|track| track.r#type == r#type)
                .map(|track| {
                    let label = track.lang.as_ref().and_then(|lang| {
                        rust_iso639::from_code_1(lang)
                            .or(rust_iso639::from_code_2t(lang))
                            .or(rust_iso639::from_code_2b(lang))
                            .map(|code| code.name.to_string())
                    });

                    MediaTrack {
                        id: track.id,
                        label,
                        lang: track.lang.to_owned().unwrap_or("und".to_owned()),
                        active: track.selected,
                    }
                })
                .collect_vec()
        };

        (media_tracks("sub"), media_tracks("audio"))
    }

    fn start(&mut self, uri: &str, start_time: f64, sender: ComponentSender<Self>) {
        self.watch_events(sender);

        let start = &format!("start=+{}", start_time / SECOND);
        if let Err(e) = self
            .mpv
            .borrow()
            .command("loadfile", &[uri, "replace", "-1", start])
        {
            error!("Failed to start: {e}");
        }
    }

    fn stop(&mut self) {
        if let Some(source) = self.events_source.take() {
            source.remove();
        }

        if let Err(e) = self.mpv.borrow().command("stop", &[]) {
            error!("Failed to stop: {e}");
        }
    }

    fn play(&self) {
        if let Err(e) = self.mpv.borrow().set_property("pause", false) {
            error!("Failed to play: {e}");
        }
    }

    fn pause(&self) {
        if let Err(e) = self.mpv.borrow().set_property("pause", true) {
            error!("Failed to pause: {e}");
        }
    }

    fn set_time(&self, time: f64) {
        let value = time / SECOND;
        if let Err(e) = self.mpv.borrow().set_property("time-pos", value) {
            error!("Failed to set time: {e}");
        }
    }

    fn set_volume(&self, volume: f64) {
        if let Err(e) = self.mpv.borrow().set_property("volume", volume) {
            error!("Failed to set volume: {e}");
        }
    }

    fn set_text_track(&self, id: i64) {
        if let Err(e) = self.mpv.borrow().set_property("sid", id) {
            error!("Failed to set text track: {e}");
        }
    }

    fn set_audio_track(&self, id: i64) {
        if let Err(e) = self.mpv.borrow().set_property("aid", id) {
            error!("Failed to set audio track: {e}");
        }
    }

    fn set_subtitles_size(&self, size: f64) {
        if let Err(e) = self.mpv.borrow().set_property("sub-scale", size) {
            error!("Failed to set subtitles size: {e}");
        }
    }

    fn set_subtitles_position(&self, position: f64) {
        if let Err(e) = self.mpv.borrow().set_property("sub-pos", position) {
            error!("Failed to set subtitles position: {e}");
        }
    }
}
