/*
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Mutex;
*/
use std::{collections::HashMap, rc::Rc, sync::Mutex};
use rand::prelude::*;
use serde::Deserialize;
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

    // spawn_local : Runs a Rust Future on the current thread
    // https://github.com/rustwasm/wasm-bindgen/issues/1126
    wasm_bindgen_futures::spawn_local(async move {
//        let (success_tx, success_rx) = futures::channel::oneshot::channel::<()>();
        let (success_tx, success_rx) = futures::channel::oneshot::channel::<Result<(), JsValue>>();
        let success_tx = Rc::new(Mutex::new(Some(success_tx)));
        let error_tx = Rc::clone(&success_tx);


        let image = web_sys::HtmlImageElement::new().unwrap();
       /* let callback = Closure::once(move || {
            success_tx.send(());
            web_sys::console::log_1(&JsValue::from_str("loaded"));
        });
        */
        let callback = Closure::once(move || {
            if let Some(success_tx) = success_tx.lock().ok()
                .and_then(|mut opt| opt.take()) {
                success_tx.send(Ok(()));
            }
            web_sys::console::log_1(&JsValue::from_str("loaded"));
        });

        let error_callback = Closure::once(move |err| {
            if let Some(error_tx) = error_tx.lock().ok()
                .and_then(|mut opt| opt.take()) {
                    error_tx.send(Err(err));
            }
        });
        
        image.set_onload(Some(callback.as_ref().unchecked_ref()));
        image.set_onerror(Some(error_callback.as_ref().unchecked_ref()));

        image.set_src("../resources/pix/Idle (1).png");
        success_rx.await;

        context.draw_image_with_html_image_element(&image, 0.0, 0.0);

        sierpinski(
            &context,
            [(300.0, 0.0), (0.0, 600.0), (600.0, 600.0)],
            (0, 255, 0),
            5,
        );

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
            if let Some(success_tx) = success_tx.lock().ok()
                .and_then(|mut opt| opt.take()) {
                success_tx.send(Ok(()));
                web_sys::console::log_1(&JsValue::from_str("sprite"));
            }
        });

        let error_callback = Closure::once(move |err| {
            if let Some(error_tx) = error_tx.lock().ok()
                .and_then(|mut opt| opt.take()) {
                    error_tx.send(Err(err));
            }
        });
        
        image.set_onload(Some(callback.as_ref().unchecked_ref()));
        image.set_onerror(Some(error_callback.as_ref().unchecked_ref()));

        image.set_src("../resources/pix/rhb.png");
        success_rx.await;
    }); //^-- wasm_bindgen_futures::spawn_local()

    Ok(())
}

async fn fetch_json(json_path: &str) -> Result<JsValue, JsValue> {
    let window = web_sys::window().unwrap();
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(json_path)).await?;
    let resp: web_sys::Response = resp_value.dyn_into()?;
    wasm_bindgen_futures::JsFuture::from(resp.json()?).await
}

//---------------------
fn sierpinski(
    context: &web_sys::CanvasRenderingContext2d,
    points: [(f64, f64); 3],
    color: (u8, u8, u8),
    depth: u8,
) {
    draw_triangle(&context, points, color);

    let depth = depth - 1;
    let [top, left, right] = points;

    if depth > 0 {
        let mut rng = thread_rng();

        let next_color = (
            rng.gen_range(0..255),
            rng.gen_range(0..255),
            rng.gen_range(0..255),
        );

        let left_middle = midpoint(top, left);
        let right_middle = midpoint(top, right);
        let bottom_middle = midpoint(left, right);

        sierpinski(
            &context,
            [top, left_middle, right_middle],
            next_color,
            depth,
        );
        sierpinski(
            &context,
            [left_middle, left, bottom_middle],
            next_color,
            depth,
        );
        sierpinski(
            &context,
            [right_middle, bottom_middle, right],
            next_color,
            depth,
        );
    } //^-- if depth
}

fn midpoint(point_1: (f64, f64), point_2: (f64, f64)) -> (f64, f64) {
    ((point_1.0 + point_2.0) / 2.0, (point_1.1 + point_2.1) / 2.0)
}

fn draw_triangle(
    context: &web_sys::CanvasRenderingContext2d,
    points: [(f64, f64); 3],
    color: (u8, u8, u8),
) {
    let [top, left, right] = points;

    // Convert Rust String to JsValue.
    let color_str = format!("rgb({}, {}, {})", color.0, color.1, color.2);
    //lets spy on it
    console::log_1(&JsValue::from_str(&color_str));
    // and use it
    context.set_fill_style(&wasm_bindgen::JsValue::from_str(&color_str));

    context.move_to(top.0, top.1);
    context.begin_path();
    context.line_to(left.0, left.1);
    context.line_to(right.0, right.1);
    context.line_to(top.0, top.1);
    context.close_path();
    context.stroke();
    context.fill();
}
