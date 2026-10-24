//! Craft-fonts faces served beside the wasm (`fonts/<sha16>/…`, copied by
//! `cargo xtask web-fonts`) instead of embedded: [`load_startup_fonts`] fetches the `startup`
//! ones before the app starts, [`spawn_background_fonts`] the rest once it runs. The browser checks
//! each file against its SHA-256 (Subresource Integrity). The Korean fallback is required before
//! startup; other failed faces are logged and skipped. With `WEB_FONTS` empty (built without
//! craft-fonts) both do nothing.

use std::cell::Cell;
use std::rc::Rc;

use vectorcraft_text::{FontDb, WEB_FONTS, WebFont};
use wasm_bindgen::JsCast as _;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::JsFuture;

/// Per startup face: the page shows its loading text meanwhile.
const STARTUP_TIMEOUT_MS: i32 = 20_000;
/// Per background face.
const BACKGROUND_TIMEOUT_MS: i32 = 120_000;
/// Background fetches at once, leaving the connection to the app's own requests.
const BACKGROUND_PARALLEL: usize = 4;

thread_local! {
    /// The background fetch runs once per page, though a graphics restart creates the app again.
    static BACKGROUND_STARTED: Cell<bool> = const { Cell::new(false) };
}

/// The full face whose Hangul coverage the release build verifies.
fn is_korean_fallback(f: &WebFont) -> bool {
    f.family == "Noto Sans CJK SC" && f.style == "Regular"
}

/// Fetch and register the `startup` faces (one after the other). Returns how many loaded, or an
/// error if the required Korean fallback is absent or fails. Empty optional local builds still run.
pub async fn load_startup_fonts() -> Result<usize, &'static str> {
    if WEB_FONTS.is_empty() {
        return Ok(0);
    }
    if !WEB_FONTS.iter().any(|f| f.startup && is_korean_fallback(f)) {
        return Err("required startup font Noto Sans CJK SC Regular is absent");
    }
    let mut loaded = 0;
    for f in WEB_FONTS.iter().filter(|f| f.startup) {
        if load(f, STARTUP_TIMEOUT_MS).await {
            loaded += 1;
        } else if is_korean_fallback(f) {
            return Err("required startup font Noto Sans CJK SC Regular could not be loaded and registered");
        }
    }
    Ok(loaded)
}

/// Fetch and register the `background` faces, [`BACKGROUND_PARALLEL`] at a time, asking `ctx` for
/// a frame after each so the font menus and the canvas pick it up. Only the first call does
/// anything.
pub fn spawn_background_fonts(ctx: egui::Context) {
    if BACKGROUND_STARTED.with(|s| s.replace(true)) {
        return;
    }
    let queue: Rc<Vec<&'static WebFont>> = Rc::new(WEB_FONTS.iter().filter(|f| !f.startup).collect());
    let next = Rc::new(Cell::new(0usize));
    for _ in 0..BACKGROUND_PARALLEL.min(queue.len()) {
        let (queue, next, ctx) = (queue.clone(), next.clone(), ctx.clone());
        wasm_bindgen_futures::spawn_local(async move {
            loop {
                let i = next.get();
                next.set(i + 1);
                let Some(f) = queue.get(i) else { break };
                if load(f, BACKGROUND_TIMEOUT_MS).await {
                    ctx.request_repaint();
                }
            }
        });
    }
}

/// Fetch one face and add it to the font database. False (and a console warning) on failure.
async fn load(f: &WebFont, timeout_ms: i32) -> bool {
    let bytes = match fetch(f.url, f.integrity, timeout_ms).await {
        Ok(bytes) => bytes,
        Err(e) => {
            log::warn!("web font {} {}: {e}", f.family, f.style);
            return false;
        }
    };
    if FontDb::global().add_font(bytes) == 0 {
        log::warn!("web font {} {}: not a font, or already loaded", f.family, f.style);
        return false;
    }
    true
}

/// GET `url` (relative to the page) with Subresource Integrity `integrity`, aborted after
/// `timeout_ms`.
async fn fetch(url: &str, integrity: &str, timeout_ms: i32) -> Result<Vec<u8>, String> {
    let window = web_sys::window().ok_or("no window")?;
    let abort = web_sys::AbortController::new().map_err(js)?;
    let init = web_sys::RequestInit::new();
    init.set_method("GET");
    init.set_integrity(integrity);
    init.set_signal(Some(&abort.signal()));
    let on_timeout = {
        let abort = abort.clone();
        Closure::<dyn FnMut()>::new(move || abort.abort())
    };
    let timer = window.set_timeout_with_callback_and_timeout_and_arguments_0(on_timeout.as_ref().unchecked_ref(), timeout_ms).map_err(js)?;
    let result = async {
        let response: web_sys::Response = JsFuture::from(window.fetch_with_str_and_init(url, &init)).await.map_err(js)?.dyn_into().map_err(js)?;
        if !response.ok() {
            return Err(format!("HTTP {} for {url}", response.status()));
        }
        let buffer = JsFuture::from(response.array_buffer().map_err(js)?).await.map_err(js)?;
        Ok(js_sys::Uint8Array::new(&buffer).to_vec())
    }
    .await;
    window.clear_timeout_with_handle(timer);
    // Kept alive until the timer can no longer fire.
    drop(on_timeout);
    result
}

fn js(e: wasm_bindgen::JsValue) -> String {
    format!("{e:?}")
}
