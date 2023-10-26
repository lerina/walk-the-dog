#[macro_use]
mod browser;
mod engine;
mod game;
mod segments;

use engine::GameLoop;
use game::WalkTheDog;
use wasm_bindgen::prelude::*;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    browser::spawn_local(async move {
        let game = WalkTheDog::new();

        GameLoop::start(game)
            .await
            .expect("Could not start game loop");
    });

    Ok(())
}

/*
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
    //let window = web_sys::window().unwrap();
    let window = browser::window().unwrap();

    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(json_path)).await?;

    let resp: web_sys::Response = resp_value.dyn_into()?;
    wasm_bindgen_futures::JsFuture::from(resp.json()?).await
}
*/
