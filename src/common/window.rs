use relm4::{
    adw::{prelude::AnimationExt, Easing, PropertyAnimationTarget, TimedAnimation},
    gtk::{
        glib::{object::IsA, Object},
        prelude::GtkWindowExt,
        Widget, Window,
    },
};

pub trait WindowExt {
    fn animate_width<T: Into<f64>>(&self, value: T);
    fn animate_height<T: Into<f64>>(&self, value: T);
    fn animate_size<T: Into<f64>>(&self, width: T, height: T);
    fn resize_to_aspect_ratio(&self, ratio: f64);
}

impl WindowExt for Window {
    fn animate_width<T: Into<f64>>(&self, value: T) {
        let width = self.default_width() as f64;
        animate_property(self, "default-width", width, value.into());
    }

    fn animate_height<T: Into<f64>>(&self, value: T) {
        let height = self.default_height() as f64;
        animate_property(self, "default-height", height, value.into());
    }

    fn animate_size<T: Into<f64>>(&self, width: T, height: T) {
        self.animate_width(width);
        self.animate_height(height);
    }

    fn resize_to_aspect_ratio(&self, aspect_ratio: f64) {
        // Force layout to update
        if !self.is_maximized() {
            self.maximize();
            self.unmaximize();
        }

        let width = self.default_width() as f64;
        let height = self.default_height() as f64;
        let ratio = width / height;

        if ratio < aspect_ratio {
            let new_height = (width / aspect_ratio).round();
            self.animate_height(new_height);
        } else {
            let new_width = (height * aspect_ratio).round();
            self.animate_width(new_width);
        }
    }
}

fn animate_property<T: IsA<Object> + IsA<Widget>>(widget: &T, property: &str, from: f64, to: f64) {
    let target = PropertyAnimationTarget::new(widget, property);
    let animation = TimedAnimation::builder()
        .widget(widget)
        .target(&target)
        .value_from(from)
        .value_to(to)
        .duration(500)
        .easing(Easing::EaseOutExpo)
        .build();

    animation.play();
}
