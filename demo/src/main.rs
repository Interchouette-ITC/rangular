use leptos::mount::mount_to_body;

mod app;
mod components;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(app::App);
}
