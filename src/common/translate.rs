use rust_i18n::t;

pub fn genre(genre: &Option<String>) -> String {
    genre.as_ref().map_or(t!("default").to_string(), |value| {
        t!(value.as_str().to_lowercase()).to_string()
    })
}
