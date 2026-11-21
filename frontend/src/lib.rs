/// 浏览器端水合入口
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);

    leptos::mount::hydrate_body(app::App);
}
