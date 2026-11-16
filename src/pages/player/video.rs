use itertools::Itertools;
use relm4::{gtk, ComponentParts, ComponentSender, SharedState, SimpleComponent};
use serde::{Deserialize, Serialize};

use crate::{common::language::Language, pages::player::mpv::MpvPlayer};

const SECOND: f64 = 1000.0;

#[derive(Debug, Deserialize)]
struct Track {
    id: i64,
    r#type: String,
    title: Option<String>,
    lang: Option<String>,
    selected: bool,
    #[serde(rename = "external-filename")]
    external_filename: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MediaTrack {
    pub id: i64,
    /// Language code used to group tracks
    pub lang: String,
    /// Display name of the language
    pub language: String,
    pub title: Option<String>,
    pub label: String,
    pub active: bool,
    /// Source of an external subtitle file
    pub url: Option<String>,
}

impl MediaTrack {
    /// A track that is only listed and not yet loaded into mpv
    pub fn pending(id: i64, lang: &str, title: &str, url: &str) -> Self {
        let (code, language) = Video::language_info(Some(lang));

        Self {
            id,
            lang: code,
            label: format!("{language} - {title}"),
            language,
            title: Some(title.to_owned()),
            active: false,
            url: Some(url.to_owned()),
        }
    }
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
    pub width: i64,
    pub height: i64,
}

pub static VIDEO_STATE: SharedState<VideoState> = SharedState::new();

#[derive(Debug)]
pub enum VideoInput {
    Load((String, f64)),
    Unload,
    Play,
    Pause,
    Seek(f64),
    Volume(f64),
    AddSubtitle((String, String, String)),
    TextTrack(i64),
    AudioTrack(i64),
    SubtitlesSize(f64),
    SubtitlesPosition(f64),
    SubtitlesColor(String),
    SubtitlesOutlineColor(String),
}

#[derive(Debug)]
pub enum VideoOutput {
    PauseChanged(bool),
    TimeChanged(f64, f64),
    TracksChanged,
    SizeChanged((i64, i64)),
    Ended,
    Error,
}

pub struct Video {
    mpv: MpvPlayer,
}

#[relm4::component(pub)]
impl SimpleComponent for Video {
    type Init = ();
    type Input = VideoInput;
    type Output = VideoOutput;

    view! {
        gtk::Box {
            #[local_ref]
            mpv -> MpvPlayer {
                connect_property_change[sender] => move |name, value| {
                    let mut state = VIDEO_STATE.write();

                    match name {
                        "pause" if let Some(value) = value.get::<bool>() => {
                            state.paused = value;
                            sender
                                .output_sender()
                                .emit(VideoOutput::PauseChanged(value));
                        }
                        "seeking" if let Some(value) = value.get::<bool>() => {
                            state.buffering = value;
                        }
                        "time-pos" if let Some(value) = value.get::<f64>() => {
                            state.time = value * SECOND;
                            sender
                                .output_sender()
                                .emit(VideoOutput::TimeChanged(state.time, state.duration));
                        }
                        "duration" if let Some(value) = value.get::<f64>() => {
                            state.duration = value * SECOND;
                        }
                        "volume" if let Some(value) = value.get::<f64>() => {
                            state.volume = value;
                        }
                        "cache-buffering-state" if let Some(value) = value.get::<i64>() => {
                            state.buffering = value < 100;
                        }
                        "width" if let Some(value) = value.get::<i64>() => {
                            state.width = value;
                            sender
                                .output_sender()
                                .emit(VideoOutput::SizeChanged((value, state.height)))
                        }
                        "height" if let Some(value) = value.get::<i64>() => {
                            state.height = value;
                            sender
                                .output_sender()
                                .emit(VideoOutput::SizeChanged((state.width, value)))
                        }
                        "track-list" if let Some(value) = value.get::<String>() => {
                            if let Ok(list) = serde_json::from_str::<Vec<Track>>(&value) {
                                let (text, audio) = Self::create_media_tracks(list);
                                state.text_tracks = text;
                                state.audio_tracks = audio;
                                sender.output_sender().emit(VideoOutput::TracksChanged);
                            }
                        },
                        _ => {}
                    }
                },

                connect_playback_ended[sender] => move || {
                    sender.output_sender().emit(VideoOutput::Ended);
                },

                connect_playback_error[sender] => move || {
                    sender.output_sender().emit(VideoOutput::Error);
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mpv = MpvPlayer::default();

        mpv.observe_property("pause");
        mpv.observe_property("seeking");
        mpv.observe_property("time-pos");
        mpv.observe_property("duration");
        mpv.observe_property("volume");
        mpv.observe_property("cache-buffering-state");
        mpv.observe_property("width");
        mpv.observe_property("height");
        mpv.observe_property("track-list");

        let model = Video { mpv };

        let mpv = &model.mpv;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            VideoInput::Load((uri, start_time)) => {
                let mut state = VIDEO_STATE.write();
                state.loaded = true;
                state.buffering = true;
                drop(state);

                let start = &format!("start=+{}", start_time / SECOND);
                self.mpv
                    .send_command("loadfile", &[&uri, "replace", "-1", start]);
            }
            VideoInput::Unload => {
                let mut state = VIDEO_STATE.write();
                state.loaded = false;
                state.height = 0;
                state.width = 0;
                drop(state);

                self.mpv.send_command("stop", &[]);
            }
            VideoInput::Play => {
                self.mpv.set_property("pause", false);
            }
            VideoInput::Pause => {
                self.mpv.set_property("pause", true);
            }
            VideoInput::Seek(time) => {
                self.mpv.set_property("time-pos", time / SECOND);
            }
            VideoInput::Volume(volume) => {
                self.mpv.set_property("volume", volume);
            }
            VideoInput::AddSubtitle((url, lang, title)) => {
                // Loaded only when picked, since mpv downloads the file. `select` makes it
                // the active track once loaded. Async so the UI thread doesn't wait on the download.
                self.mpv
                    .send_command_async("sub-add", &[&url, "select", &title, &lang]);
            }
            VideoInput::TextTrack(id) => {
                if id == -1 {
                    self.mpv.set_property("sid", "no");
                } else {
                    self.mpv.set_property("sid", id);
                }
            }
            VideoInput::AudioTrack(id) => {
                self.mpv.set_property("aid", id);
            }
            VideoInput::SubtitlesSize(size) => {
                self.mpv.set_property("sub-scale", size);
            }
            VideoInput::SubtitlesPosition(position) => {
                self.mpv.set_property("sub-pos", position);
            }
            VideoInput::SubtitlesColor(color) => {
                self.mpv.set_property("sub-color", color);
            }
            VideoInput::SubtitlesOutlineColor(color) => {
                self.mpv.set_property("sub-border-color", color);
            }
        }
    }
}

impl Video {
    fn language_info(lang: Option<&str>) -> (String, String) {
        let language = lang.and_then(|lang| Language::try_from(lang).ok());

        match language {
            Some(language) => (language.code.to_owned(), language.name.to_owned()),
            None => ("und".to_owned(), "Unknown".to_owned()),
        }
    }

    fn create_media_tracks(list: Vec<Track>) -> (Vec<MediaTrack>, Vec<MediaTrack>) {
        let media_tracks = |r#type: &str| {
            list.iter()
                .filter(|track| track.r#type == r#type)
                .map(|track| {
                    let (code, language) = Self::language_info(track.lang.as_deref());
                    let known = code != "und";

                    let label = match (known, &track.title) {
                        (true, Some(title)) => format!("{} - {}", language, title),
                        (true, None) => language.to_owned(),
                        (false, Some(title)) => title.to_owned(),
                        _ => "Unknown".to_owned(),
                    };

                    MediaTrack {
                        id: track.id,
                        lang: code,
                        language,
                        title: track.title.to_owned(),
                        label,
                        active: track.selected,
                        url: track.external_filename.to_owned(),
                    }
                })
                .sorted_by(|a, b| Ord::cmp(&a.lang, &b.lang))
                .collect_vec()
        };

        let mut text_tracks = media_tracks("sub");
        let text_track_disabled = text_tracks.iter().all(|track| !track.active);

        text_tracks.insert(
            0,
            MediaTrack {
                id: -1,
                lang: "und".to_owned(),
                language: String::new(),
                title: None,
                label: "disabled".to_owned(),
                active: text_track_disabled,
                url: None,
            },
        );

        (text_tracks, media_tracks("audio"))
    }
}
