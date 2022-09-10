use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Mutex;

use rand::prelude::*;
use serde::Deserialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
// use web_sys::console;

#[derive(Deserialize)]
struct Sheet {
    frames: HashMap<String, Cell>,
}

#[derive(Deserialize)]
struct Cell {
    frame: Rect,
}

#[derive(Deserialize)]
struct Rect {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

// // When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// // allocator.
// //
// // If you don't want to use `wee_alloc`, you can safely delete this.
// #[cfg(feature = "wee_alloc")]
// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // // This provides better error messages in debug mode.
    // // It's disabled in release mode so it doesn't bloat up the file size.
    // #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    // Your code goes here!
    // console::log_1(&JsValue::from_str("Hello world!"));

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    wasm_bindgen_futures::spawn_local(async move {
        let (success_tx, success_rx) = futures::channel::oneshot::channel::<Result<(), JsValue>>();
        let success_tx = Rc::new(Mutex::new(Some(success_tx)));
        let error_tx = Rc::clone(&success_tx);
        let image = web_sys::HtmlImageElement::new().unwrap();

        let callback = Closure::once(move || {
            if let Some(success_tx) = success_tx.lock().ok().and_then(|mut opt| opt.take()) {
                let _ = success_tx.send(Ok(()));
            }
        });

        let error_callback = Closure::once(move |err| {
            if let Some(error_tx) = error_tx.lock().ok().and_then(|mut opt| opt.take()) {
                let _ = error_tx.send(Err(err));
            }
        });

        image.set_onload(Some(callback.as_ref().unchecked_ref()));
        image.set_onerror(Some(error_callback.as_ref().unchecked_ref()));
        image.set_src("Idle (1).png");

        let _ = success_rx.await.unwrap();
        let _ = context.draw_image_with_html_image_element(&image, 0.0, 0.0);

        let side = 600.0;
        let x = side / 2.0;
        let y = 0.0;

        sierpinski(&context, x, y, side, "rgb(0, 255, 0)", 8);

        let json = fetch_json("rhb.json")
            .await
            .expect("Could not fetch rhb.json");

        let sheet: Sheet = json
            .into_serde()
            .expect("Could not convert rhb.json into a Sheet structure");

        // COPY PASTE BEGINS

        let (success_tx, success_rx) = futures::channel::oneshot::channel::<Result<(), JsValue>>();
        let success_tx = Rc::new(Mutex::new(Some(success_tx)));
        let error_tx = Rc::clone(&success_tx);
        let image = web_sys::HtmlImageElement::new().unwrap();

        let callback = Closure::once(move || {
            if let Some(success_tx) = success_tx.lock().ok().and_then(|mut opt| opt.take()) {
                let _ = success_tx.send(Ok(()));
            }
        });

        let error_callback = Closure::once(move |err| {
            if let Some(error_tx) = error_tx.lock().ok().and_then(|mut opt| opt.take()) {
                let _ = error_tx.send(Err(err));
            }
        });

        image.set_onload(Some(callback.as_ref().unchecked_ref()));
        image.set_onerror(Some(error_callback.as_ref().unchecked_ref()));
        image.set_src("rhb.png");

        let _ = success_rx.await.unwrap();

        // COPY PASTE ENDS

        let sprite = sheet.frames.get("Run (1).png").expect("Cell not found");
        let _ = context
            .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &image,
                sprite.frame.x.into(),
                sprite.frame.y.into(),
                sprite.frame.w.into(),
                sprite.frame.h.into(),
                200.0,
                300.0,
                sprite.frame.w.into(),
                sprite.frame.h.into(),
            );
    });

    Ok(())
}

async fn fetch_json(json_path: &str) -> Result<JsValue, JsValue> {
    let window = web_sys::window().unwrap();
    // window.fetch_with_str(json_path) returns a JS Promise
    // which we then convert to a Rust Future so that we can await on it on the next line
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(json_path)).await?;
    // JS's dynamic typing into Rust's static typing
    let resp: web_sys::Response = resp_value.dyn_into()?;
    wasm_bindgen_futures::JsFuture::from(resp.json()?).await
}

fn sierpinski(
    context: &web_sys::CanvasRenderingContext2d,
    x_top: f64,
    y_top: f64,
    side: f64,
    color: &str,
    depth: usize,
) {
    web_sys::console::log_1(&JsValue::from_str(&format!(
        "Drawing triangle at {x_top} {y_top}"
    )));

    context.move_to(x_top, y_top);
    context.begin_path();
    context.line_to(x_top - side / 2.0, y_top + side);
    context.line_to(x_top + side / 2.0, y_top + side);
    context.line_to(x_top, y_top);
    context.close_path();
    context.stroke();
    context.set_fill_style(&color.into());
    context.fill();

    if depth > 0 {
        let mut rng = thread_rng();

        let color = (
            rng.gen_range(0..255),
            rng.gen_range(0..255),
            rng.gen_range(0..255),
        );

        let color = &format!("rgb({}, {}, {})", color.0, color.1, color.2);

        let side = side / 2.0;
        sierpinski(context, x_top, y_top, side, color, depth - 1);
        sierpinski(
            context,
            x_top - side / 2.0,
            y_top + side,
            side,
            color,
            depth - 1,
        );
        sierpinski(
            context,
            x_top + side / 2.0,
            y_top + side,
            side,
            color,
            depth - 1,
        );
    }
}
