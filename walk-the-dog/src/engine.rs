//use crate::browser;
use crate::browser::{self, LoopClosure};//self is to have browser too.
use anyhow::{anyhow, Result};
use futures::channel::oneshot::channel;
//use std::borrow::Borrow;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Mutex;
//use wasm_bindgen::closure::Closure;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{HtmlImageElement, CanvasRenderingContext2d};

/*
We are still dependent on wasm_bindgen for the Closure and JSValue types, as well
as the unchecked_ref function, but we've reduced the amount of direct platform
dependencies. Our only JS dependency is on HtmlImageElement .
Now, take a look at the very beginning of the function and you'll see
the new_image call can use the ? operator to early return in the event of an error,
with a standard Rust error type.

This is why we mapped those errors in the browser functions.

Moving past the first two lines of the method, the rest of the function
is largely the same as before, replacing any direct calls to wasm-bindgen functions
with their corresponding calls in browser .

We've changed the channel to send anyhow::Result and used anyhow! in error_callback .
This then allows us to end the function with a call to complete_rx.await??
and Ok(image) . Those two ?? are not a misprint;
complete_rx.await returns Result<Result<(), anyhow::Error>,
Canceled> .
Since anyhow::Error and Canceled both conform to std::error::Error ,
we can handle those errors with ? each time.
*/
pub async fn load_image(source: &str) -> Result<HtmlImageElement> {
    let image = browser::new_image()?;
    let (complete_tx, complete_rx) = channel::<Result<()>>();
    let success_tx = Rc::new(Mutex::new(Some(complete_tx)));
    let error_tx = Rc::clone(&success_tx);

    let success_callback = browser::closure_once(move || {
        if let Some(success_tx) = success_tx.lock().ok().and_then(|mut opt| opt.take()) {
            success_tx.send(Ok(()));
        }
    });

    let error_callback: Closure<dyn FnMut(JsValue)> = browser::closure_once(move |err| {
        if let Some(error_tx) = error_tx.lock().ok().and_then(|mut opt| opt.take()) {
            error_tx.send(Err(anyhow!("Error Loading Image:{:#?}", err)));
        }
    });

    image.set_onload(Some(success_callback.as_ref().unchecked_ref()));
    image.set_onerror(Some(error_callback.as_ref().unchecked_ref()));
    image.set_src(source);
    complete_rx.await??;

    Ok(image)
}

pub trait Game {
    fn update(&mut self);
    fn draw(&self, context: &CanvasRenderingContext2d);
}

const FRAME_SIZE: f32 = 1.0 / 60.0 * 1000.0;

pub struct GameLoop {
    last_frame: f64,
    accumulated_delta: f32,
}

type SharedLoopClosure = Rc<RefCell<Option<LoopClosure>>>;

impl GameLoop {
    pub async fn start(mut game: impl Game + 'static) -> Result<()> {
        let mut game_loop = GameLoop {
            last_frame: browser::now()?,
            accumulated_delta: 0.0,
        };

        let f: SharedLoopClosure = Rc::new(RefCell::new(None));

        let g = f.clone();

        *g.borrow_mut() = Some(browser::create_raf_closure(move |perf: f64| {

            game_loop.accumulated_delta += (perf - game_loop.last_frame) as f32;
            while game_loop.accumulated_delta > FRAME_SIZE {
                game.update();
                game_loop.accumulated_delta -= FRAME_SIZE;
            }
            game_loop.last_frame = perf;
            game.draw(&browser::context().expect("Context should exist",));
            
            browser::request_animation_frame(f.borrow().as_ref().unwrap());
        }));

        browser::request_animation_frame(
            g.borrow()
                .as_ref()
                .ok_or_else(|| anyhow!("GameLoop: Loop is None"))?,
        )?;

        Ok(())
    }//^-- fn start

}//-- GameLoop
