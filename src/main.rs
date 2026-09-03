mod components;
mod logic;
mod model;
mod pages;

mod app;
mod helpers;

use app::*;
use leptos::logging;

fn main() {
    console_error_panic_hook::set_once();
    logging::log!("csr mode - mounting to body");
    leptos::mount::mount_to_body(App)
}
