use std::collections::BTreeMap;

use relm4::{
    css,
    gtk::{self, prelude::*},
    ComponentParts, ComponentSender, SimpleComponent,
};
use rust_i18n::t;

use super::video::MediaTrack;

const OFF_ROW: &str = "off";
const LANGUAGES_PAGE: &str = "languages";
const VARIANTS_PAGE: &str = "variants";

pub struct TracksMenuInit {
    pub icon: &'static str,
    /// Show a language list first, then the tracks of the chosen language
    pub grouped: bool,
}

#[derive(Debug)]
pub enum TracksMenuInput {
    Update(Vec<MediaTrack>),
    LanguageClicked(String),
    Back,
}

#[derive(Debug)]
pub enum TracksMenuOutput {
    TrackChanged(i64),
}

pub struct TracksMenu {
    icon: &'static str,
    grouped: bool,
    tracks: Vec<MediaTrack>,
    selected_language: Option<String>,
    stack: gtk::Stack,
    languages_list: gtk::ListBox,
    variants_list: gtk::ListBox,
    back_label: gtk::Label,
}

#[relm4::component(pub)]
impl SimpleComponent for TracksMenu {
    type Init = TracksMenuInit;
    type Input = TracksMenuInput;
    type Output = TracksMenuOutput;

    view! {
        gtk::MenuButton {
            add_css_class: relm4::css::classes::OSD,
            set_size_request: (45, 45),
            set_icon_name: model.icon,

            #[watch]
            set_visible: !model.tracks.is_empty(),

            #[wrap(Some)]
            set_popover = &gtk::Popover {
                connect_closed[sender] => move |_| {
                    sender.input_sender().emit(TracksMenuInput::Back);
                },

                #[local_ref]
                stack -> gtk::Stack {
                    set_transition_type: gtk::StackTransitionType::SlideLeftRight,
                    set_interpolate_size: true,
                    set_vhomogeneous: false,
                    set_hhomogeneous: false,
                }
            }
        },
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let stack = gtk::Stack::new();

        let languages_list = Self::create_list();
        let variants_list = Self::create_list();

        languages_list.connect_row_activated({
            let sender = sender.input_sender().clone();
            move |_, row| {
                sender.emit(TracksMenuInput::LanguageClicked(
                    row.widget_name().to_string(),
                ));
            }
        });

        let back_label = gtk::Label::builder()
            .halign(gtk::Align::Start)
            .css_classes([css::classes::HEADING])
            .build();

        let back_content = gtk::Box::builder().spacing(8).build();
        back_content.append(&gtk::Image::from_icon_name("go-previous-symbolic"));
        back_content.append(&back_label);

        let back_button = gtk::Button::builder()
            .css_classes(["flat"])
            .child(&back_content)
            .build();

        back_button.connect_clicked({
            let sender = sender.input_sender().clone();
            move |_| sender.emit(TracksMenuInput::Back)
        });

        let languages_page = Self::create_scrolled(&languages_list);
        let variants_page = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .build();
        variants_page.append(&back_button);
        variants_page.append(&Self::create_scrolled(&variants_list));

        stack.add_named(&languages_page, Some(LANGUAGES_PAGE));
        stack.add_named(&variants_page, Some(VARIANTS_PAGE));
        stack.set_visible_child_name(if init.grouped {
            LANGUAGES_PAGE
        } else {
            VARIANTS_PAGE
        });

        back_button.set_visible(init.grouped);

        let model = TracksMenu {
            icon: init.icon,
            grouped: init.grouped,
            tracks: vec![],
            selected_language: None,
            stack: stack.clone(),
            languages_list,
            variants_list,
            back_label,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            TracksMenuInput::Update(tracks) => {
                self.tracks = tracks;

                if self.grouped {
                    self.rebuild_languages();

                    // Keep the opened language in sync with the new tracks
                    match self.selected_language.clone() {
                        Some(lang) if self.has_language(&lang) => self.rebuild_variants(&sender),
                        _ => {
                            self.selected_language = None;
                            self.stack.set_visible_child_name(LANGUAGES_PAGE);
                        }
                    }
                } else {
                    self.rebuild_variants(&sender);
                }
            }
            TracksMenuInput::LanguageClicked(code) => {
                if code == OFF_ROW {
                    if let Some(track) = self.tracks.iter().find(|track| track.id == -1) {
                        sender
                            .output_sender()
                            .emit(TracksMenuOutput::TrackChanged(track.id));
                    }
                } else {
                    self.selected_language = Some(code);
                    self.rebuild_variants(&sender);
                    self.stack.set_visible_child_name(VARIANTS_PAGE);
                }
            }
            TracksMenuInput::Back => {
                if self.grouped {
                    self.stack.set_visible_child_name(LANGUAGES_PAGE);
                    self.selected_language = None;
                }
            }
        }
    }
}

impl TracksMenu {
    fn create_list() -> gtk::ListBox {
        gtk::ListBox::builder()
            .css_classes([css::classes::OSD, "tracks-list"])
            .width_request(200)
            .selection_mode(gtk::SelectionMode::None)
            .build()
    }

    fn create_scrolled(list: &gtk::ListBox) -> gtk::ScrolledWindow {
        gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .propagate_natural_height(true)
            .max_content_height(300)
            .min_content_width(200)
            .child(list)
            .build()
    }

    fn is_language_track(track: &MediaTrack) -> bool {
        track.id != -1
    }

    fn has_language(&self, lang: &str) -> bool {
        self.tracks
            .iter()
            .any(|track| Self::is_language_track(track) && track.lang == lang)
    }

    fn rebuild_languages(&self) {
        self.languages_list.remove_all();

        if let Some(off) = self.tracks.iter().find(|track| track.id == -1) {
            self.languages_list.append(&Self::language_row(
                OFF_ROW,
                &t!(&off.label),
                None,
                off.active,
            ));
        }

        // (display name, count, active) by language code, sorted by display name
        let mut languages: BTreeMap<&str, (&str, usize, bool)> = BTreeMap::new();
        for track in self.tracks.iter().filter(|t| Self::is_language_track(t)) {
            let entry =
                languages
                    .entry(track.lang.as_str())
                    .or_insert((track.language.as_str(), 0, false));
            entry.1 += 1;
            entry.2 |= track.active;
        }

        let mut languages = languages.into_iter().collect::<Vec<_>>();
        languages.sort_by(|a, b| a.1 .0.cmp(b.1 .0));

        for (code, (name, count, active)) in languages {
            self.languages_list
                .append(&Self::language_row(code, name, Some(count), active));
        }
    }

    fn language_row(code: &str, name: &str, count: Option<usize>, active: bool) -> gtk::ListBoxRow {
        let content = gtk::Box::builder()
            .spacing(8)
            .margin_start(12)
            .margin_end(12)
            .height_request(40)
            .build();

        let label = gtk::Label::builder()
            .label(name)
            .halign(gtk::Align::Start)
            .hexpand(true)
            .build();
        content.append(&label);

        if let Some(count) = count {
            content.append(
                &gtk::Label::builder()
                    .label(count.to_string())
                    .css_classes([css::classes::DIM_LABEL])
                    .build(),
            );
            content.append(&gtk::Image::from_icon_name("go-next-symbolic"));
        } else if active {
            content.append(&gtk::Image::from_icon_name("object-select-symbolic"));
        }

        if count.is_some() && active {
            label.add_css_class(css::classes::ACCENT);
        }

        gtk::ListBoxRow::builder()
            .name(code)
            .activatable(true)
            .child(&content)
            .build()
    }

    fn rebuild_variants(&self, sender: &ComponentSender<Self>) {
        self.variants_list.remove_all();

        let group = gtk::CheckButton::new();

        let tracks: Vec<&MediaTrack> = if self.grouped {
            let Some(lang) = &self.selected_language else {
                return;
            };

            if let Some(track) = self
                .tracks
                .iter()
                .find(|t| Self::is_language_track(t) && &t.lang == lang)
            {
                self.back_label.set_label(&track.language);
            }

            self.tracks
                .iter()
                .filter(|track| Self::is_language_track(track) && &track.lang == lang)
                .collect()
        } else {
            self.tracks.iter().collect()
        };

        // Several variants share a title (e.g. the same addon), so number those
        let mut totals: BTreeMap<String, usize> = BTreeMap::new();
        for track in &tracks {
            *totals.entry(self.variant_label(track)).or_default() += 1;
        }

        let mut seen: BTreeMap<String, usize> = BTreeMap::new();
        for track in tracks {
            let mut label = self.variant_label(track);

            if totals.get(&label).copied().unwrap_or(0) > 1 {
                let number = seen.entry(label.to_owned()).or_default();
                *number += 1;
                label = format!("{label} ({number})");
            }

            let button = gtk::CheckButton::builder()
                .label(&label)
                .height_request(35)
                .margin_start(12)
                .margin_end(12)
                .group(&group)
                .active(track.active)
                .build();

            button.connect_toggled({
                let id = track.id;
                let sender = sender.output_sender().clone();
                move |button| {
                    if button.is_active() {
                        sender.emit(TracksMenuOutput::TrackChanged(id));
                    }
                }
            });

            self.variants_list.append(&button);
        }
    }

    fn variant_label(&self, track: &MediaTrack) -> String {
        if !self.grouped {
            return t!(&track.label).to_string();
        }

        match &track.title {
            Some(title) if !title.is_empty() => title.to_owned(),
            _ if track.url.is_some() => "External".to_owned(),
            _ => "Embedded".to_owned(),
        }
    }
}
