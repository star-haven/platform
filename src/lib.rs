pub mod app;
pub mod auth;
pub mod prelude;
pub mod shell;
pub mod create;
pub mod browse;

#[cfg(feature = "hydrate")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(feature = "hydrate")]
#[wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;

    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}

#[cfg(feature = "ssr")]
pub fn static_assets_dir() -> std::path::PathBuf {
    let proj_dirs = directories::ProjectDirs::from("dev", "Star Haven", "Star Haven Platform").unwrap();
    proj_dirs.data_dir().to_path_buf()
}
