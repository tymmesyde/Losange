pub const APP_ID: &str = "xyz.timtimtim.Losange";
pub const APP_NAME: &str = "Losange";
pub const POSTER_RATIO: f64 = 0.664;
pub const ITEM_MAX_SIZE: i32 = 225;
pub const ITEM_MIN_SIZE: i32 = (ITEM_MAX_SIZE as f64 * POSTER_RATIO) as i32;
pub const CATALOG_ICON_SIZE: i32 = 24;
pub const DETAILS_LOGO_SIZE: (i32, i32) = (80, 160);
pub const SERVER_UPDATER_ENDPOINT: &str = "https://www.strem.io/updater/server/check";
pub const SERVER_DOWNLOAD_ENDPOINT: &str = "https://dl.strem.io/server/vVERSION/desktop/server.js";
pub const COMMUNITY_MANIFESTS: &[&str] = &[
    "https://v3-cinemeta.strem.io/manifest.json",
    "https://stremio-addons.com/manifest.json",
];
pub const SUBTITLES_MIN_SIZE: u8 = 25;
pub const SUBTITLES_MAX_SIZE: u8 = 175;
