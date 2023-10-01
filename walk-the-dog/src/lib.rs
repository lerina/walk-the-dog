#[macro_use]
mod browser; // This makes log! available whenever the browser module is used

use rand::prelude::*;
use serde::Deserialize;
use std::{collections::HashMap, rc::Rc, sync::Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console;

#[derive(Deserialize)]
struct Sheet {
    frames: HashMap<String, Cell>,
}

#[derive(Deserialize)]
struct Rect {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

#[derive(Deserialize)]
struct Cell {
    frame: Rect,
}

#[wasm_bindgen(start, catch)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    console::log_1(&JsValue::from_str(
        "You can see this in the browsers console log",
    ));

    //let window = browser::window().expect("No Window Found");
    //let document = window.document().unwrap();
    let document = browser::document().expect("No Document Found");
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

    // spawn_local : Runs a Rust Future on the current thread
    // https://github.com/rustwasm/wasm-bindgen/issues/1126
    wasm_bindgen_futures::spawn_local(async move {
        //
        let json = fetch_json("../resources/pix/rhb.json")
            .await
            .expect("Could not fetch rhb.json");
        let sheet: Sheet = json
            .into_serde()
            .expect("Could not convert rhb.json into a Sheet structure");

        let (success_tx, success_rx) = futures::channel::oneshot::channel::<Result<(), JsValue>>();
        let success_tx = Rc::new(Mutex::new(Some(success_tx)));
        let error_tx = Rc::clone(&success_tx);

        let image = web_sys::HtmlImageElement::new().unwrap();

        let callback = Closure::once(move || {
            if let Some(success_tx) = success_tx.lock().ok().and_then(|mut opt| opt.take()) {
                success_tx.send(Ok(()));
                web_sys::console::log_1(&JsValue::from_str("sprite"));
            }
        });

        let error_callback = Closure::once(move |err| {
            if let Some(error_tx) = error_tx.lock().ok().and_then(|mut opt| opt.take()) {
                error_tx.send(Err(err));
            }
        });

        image.set_onload(Some(callback.as_ref().unchecked_ref()));
        image.set_onerror(Some(error_callback.as_ref().unchecked_ref()));

        image.set_src("../resources/pix/rhb.png");
        success_rx.await;

        // frame counter for the animation
        let mut frame = -1;

        // set up a callback every 50 milliseconds
        // with JavaScript's setInterval function,
        // which is called set_interval_with_callback .
        // First, we need to set up the callback itself
        // This closure will be called multiple times
        // so we use Closure::wrap not Closure::once
        // This also means we need to use Box with an explicit cast,
        // as Box<dyn FnMut()> , because the wrap function requires Box ,
        // and there isn't enough information for the compiler to infer the type.
        // So:
        // 1. Get interval callback, clear screen, draw sprite
        let interval_callback = Closure::wrap(Box::new(move || {
            // cycle the frame count between 0 and 7
            frame = (frame + 1) % 8;
            let frame_name = format!("Run ({}).png", frame + 1);
            let sprite = sheet.frames.get(&frame_name).expect("Cell not found");
            context.clear_rect(0.0, 0.0, 600.0, 600.0);

            context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &image,
                sprite.frame.x.into(),
                sprite.frame.y.into(),
                sprite.frame.w.into(),
                sprite.frame.h.into(),
                300.0,
                300.0,
                sprite.frame.w.into(),
                sprite.frame.h.into(),
            );
        }) as Box<dyn FnMut()>);

        // 2. schedule it to be called every 50 milliseconds
        /*
        window.set_interval_with_callback_and_timeout_and_arguments_0(
            interval_callback.as_ref().unchecked_ref(),
            50,
        );
        */
        // now using our broser module
        browser::window()
            .unwrap()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                interval_callback.as_ref().unchecked_ref(),
                50,
            );
        // Now we need to forget the closure that we passed into setInterval
        // so that Rust doesn't destroy it when we leave the scope of this future.
        interval_callback.forget();
    }); //^-- wasm_bindgen_futures::spawn_local()

    Ok(())
}

// NOTE: JsFuture is not a JavaScript future but a Rust future
// backed by a JavaScript promise
//
// window.fetch_with_str() corresponds to the window.fetch function in JavaScript
// That function returns Promise , which we immediately convert to a future
// via the `from` call
//
// we cast the returned resp_value into Response
// because the fetch call resolves to JsValue .
// we must convert from the dynamic typing of JavaScript
// to the static typing of Rust, and the dyn_into() function does that.
// resp.json() also returns a promise, so we wrap it in JsFuture
// as well and block on it with an await call.
async fn fetch_json(json_path: &str) -> Result<JsValue, JsValue> {
    let window = web_sys::window().unwrap();

    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(json_path)).await?;

    let resp: web_sys::Response = resp_value.dyn_into()?;
    wasm_bindgen_futures::JsFuture::from(resp.json()?).await
}
