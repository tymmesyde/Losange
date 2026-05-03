mod imp;
mod properties;

use libmpv2::{Format, SetData};
use relm4::adw::subclass::prelude::ObjectSubclassIsExt;
use relm4::gtk::{
    self,
    glib::{self, closure_local, object::ObjectExt, Variant},
};
use tracing::error;

use properties::{BOOL_PROPERTIES, FLOAT_PROPERTIES, INTEGER_PROPERTIES, STRING_PROPERTIES};

glib::wrapper! {
    pub struct MpvPlayer(ObjectSubclass<imp::MpvPlayer>)
        @extends gtk::GLArea, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for MpvPlayer {
    fn default() -> Self {
        glib::Object::builder()
            .property("hexpand", true)
            .property("vexpand", true)
            .build()
    }
}

impl MpvPlayer {
    pub fn connect_property_change<T: Fn(&str, Variant) + 'static>(&self, callback: T) {
        self.connect_closure(
            "property-changed",
            false,
            closure_local!(move |_: MpvPlayer, name: &str, value: Variant| {
                callback(name, value);
            }),
        );
    }

    pub fn connect_playback_ended<T: Fn() + 'static>(&self, callback: T) {
        self.connect_closure(
            "playback-ended",
            false,
            closure_local!(move |_: MpvPlayer| {
                callback();
            }),
        );
    }

    pub fn connect_playback_error<T: Fn() + 'static>(&self, callback: T) {
        self.connect_closure(
            "playback-error",
            false,
            closure_local!(move |_: MpvPlayer| {
                callback();
            }),
        );
    }

    pub fn send_command(&self, name: &str, args: &[&str]) {
        self.imp().send_command(name, args);
    }

    pub fn observe_property(&self, name: &str) {
        let widget = self.imp();

        match name {
            name if FLOAT_PROPERTIES.contains(&name) => {
                widget.observe_property(name, Format::Double);
            }
            name if INTEGER_PROPERTIES.contains(&name) => {
                widget.observe_property(name, Format::Int64);
            }
            name if BOOL_PROPERTIES.contains(&name) => {
                widget.observe_property(name, Format::Flag);
            }
            name if STRING_PROPERTIES.contains(&name) => {
                widget.observe_property(name, Format::String);
            }
            _ => error!("Failed to observe property {name}: Unsupported"),
        };
    }

    pub fn set_property<T: SetData>(&self, name: &str, value: T) {
        self.imp().set_property(name, value);
    }
}
