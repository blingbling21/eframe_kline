mod app;
mod kline;

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::prelude::wasm_bindgen;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn main() {
    console_error_panic_hook::set_once();

    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "eframe_kline",
            web_options,
            Box::new(|cc| Box::new(app::App::new(cc))),
        )
        .await
        .expect("启动eframe失败");
    });
}
