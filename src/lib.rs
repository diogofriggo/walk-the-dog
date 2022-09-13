#[macro_use]
mod browser;

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Mutex;

use serde::Deserialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

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

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let document = browser::document().expect("No Document Found");
    let canvas = document
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();

    let context = browser::context().expect("Could not get browser context");

    browser::spawn_local(async move {
        let sheet: Sheet = browser::fetch_json("rhb.json")
            .await
            .expect("Could not fetch rhb.json")
            .into_serde()
            .expect("Could not convert rhb.json into a Sheet structure");

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

        // Closure::once(move || { is able to convert to Close<dyn FnMut()> but Closure::wrap( isn't
        // unlike Closure::once, Closure::wrap requires a Box, why?
        let mut frame = -1;
        let interval_callback = Closure::wrap(Box::new(move || {
            context.clear_rect(0.0, 0.0, 600.0, 600.0);
            frame = (frame + 1) % 8;
            let frame_name = format!("Run ({frame}).png");
            let sprite = sheet.frames.get(&frame_name).expect("Cell not found");
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
        }) as Box<dyn FnMut()>);

        let _ = browser::window()
            .unwrap()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                interval_callback.as_ref().unchecked_ref(),
                50,
            );

        interval_callback.forget();
    });

    Ok(())
}
