//! Writes the native SSR HTML of the shared counter view for the browser
//! hydration test: `cargo run --example render_counter -- <output.html>`.

fn main() {
    let output = std::env::args_os()
        .nth(1)
        .expect("usage: render_counter <output.html>");
    let html = rangular_document_consumer_fixture::render_counter();
    std::fs::write(&output, html).expect("write server-rendered counter HTML");
}
