use kalypsi::*;
use leptos::*;
use leptos_meta::provide_meta_context;

fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    mount_to_body(|cx| {
        provide_meta_context(cx);

        view! { cx, <App/> }
    })
}
