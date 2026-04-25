use relm4::{
    adw::{prelude::AnimationExt, Easing, PropertyAnimationTarget, TimedAnimation},
    gtk::{
        glib::{object::IsA, Object},
        prelude::GtkWindowExt,
        Widget, Window,
    },
};

pub trait WindowExt {
    fn dimensions(&self) -> (f64, f64);
    fn animate_width(&self, value: f64);
    fn animate_height(&self, value: f64);
}

impl WindowExt for Window {
    fn dimensions(&self) -> (f64, f64) {
        let width = self.default_width() as f64;
        let height = self.default_height() as f64;
        (width, height)
    }

    fn animate_width(&self, value: f64) {
        let width = self.default_width() as f64;
        animate_property(self, "default-width", width, value);
    }

    fn animate_height(&self, value: f64) {
        let height = self.default_height() as f64;
        animate_property(self, "default-height", height, value);
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
