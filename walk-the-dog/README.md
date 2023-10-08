## Loading assets

Expanding our game loop to handle loading assets is going 
to require adding a function to our trait, an async one to be precise. 
This will allow us to put all our asynchronous code 
that's currently wrapped in the `spawn_local` in lib 
and put it in a function that returns Result with Game in it.

NOTE:
    async **trait functions** haven't landed in stable Rust yet.
    > cargo add async-trait

```rust
// filename: src/engine.rs
use async_trait::async_trait;

...

#[async_trait(?Send)]
pub trait Game {
    async fn initialize(&self) -> Result<Box<dyn Game>>;
    fn update(&mut self);
    fn draw(&self, context: &Renderer);
}

...

impl GameLoop {
    pub async fn start(mut game: impl Game + 'static) -> Result<()> {
        let mut game = game.initialize().await?;
        let mut game_loop = GameLoop {
            last_frame: browser::now()?,
            accumulated_delta: 0.0,
        };

        let renderer = Renderer {
            context: browser::context()?,
        };
        
        ...
```


### Cleaner drawing

Currently, we're sending a raw CanvasRenderingContext2d to the draw loop, 
with all of its awkward functions such as 
`draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh` . 

This works but it's ugly, and much like we did with the browser module, 
we can use a wrapper to narrow the context's wide interface to a smaller one, 
tailored to our needs. 
We'll replace passing CanvasRenderingContext2d with our own Renderer object 
that has easier-to-use functions.

We'll start by creating a structure for our Renderer in engine :

```rust
// filename: src/engine.rs

pub struct Renderer {
    context: CanvasRenderingContext2d,
}

```

This is a simple wrapper containing the rendering context. 
For now, we'll just add the two


```rust
// filename: src/engine.rs

impl Renderer {
    pub fn clear(&self, rect: &Rect) {
        self.context.clear_rect(
            rect.x.into(),
            rect.y.into(),
            rect.width.into(),
            rect.height.into(),
        );
    }

    pub fn draw_image(&self, 
                        image: &HtmlImageElement, 
                        frame: &Rect, 
                        destination: &Rect) {
        self.context
         .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
            &image,
            frame.x.into(),
            frame.y.into(),
            frame.width.into(),
            frame.height.into(),
            destination.x.into(),
            destination.y.into(),
            destination.width.into(),
            destination.height.into(),
        )
        .expect("Drawing is throwing exceptions! Unrecoverable error.");
    }//-- draw_image
    

}//^-- impl Renderer
```

These two functions, clear and draw_image , both wrap context functions but
do so using fewer parameters. 
Instead of four parameters and clear_rect , we pass clear Rect. 
Instead of that incredibly long function name, we pass draw_image
HtmlImageElement and two Rect structures. 

Currently, we go ahead and use expect to panic! here if we can't draw.

Of course, both of these functions take Rect, 
but we don't have a Rect structure. 
Let's add that to the engine

```rust
// filename: src/engine.rs

pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

```


Note: Since we replaced 

```rust
// filename: src/engine.rs

pub trait Game {
    fn update(&mut self);
    fn draw(&self, context: &CanvasRenderingContext2d);
}
```

with 

```rust
// filename: src/engine.rs

#[async_trait(?Send)]
pub trait Game {
    async fn initialize(&self) -> Result<Box<dyn Game>>;
    fn update(&mut self);
    fn draw(&self, context: &Renderer);
}
```

so 

```rust
// filename: src/engine.rs

            //game.draw(&browser::context().expect("Context should exist",));
            game.draw(&renderer);
```

Let's take our animation code out of lib and using the game loop.

### Integrating the game loop

It's great that we've written this game loop and all, but it's about time we actually use it.

Remember that we have our GameLoop structure, but it operates on a Game trait. 
So in order to use the loop, we need to implement that trait.
 
We'll implement it in another `module, game` , which we'll create in `game.rs` 
and then add to the library using the `mod game` instruction declaration in `lib.rs` .


```rust
// filename: lib.rs
mod game;


```

```rust
// filename: game.rs
use crate::engine::{Game, Renderer};
use anyhow::Result;
use async_trait::async_trait;

pub struct WalkTheDog;

#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        Ok(Box::new(WalkTheDog {}))
    }

    fn update(&mut self) {}
    fn draw(&self, renderer: &Renderer) {}
}

```

The initialize function might look a little strange 
because we're taking self and just throwing it away in favor 
of a new WalkTheDog structure – thrown on the heap, no less! 

We're doing that for some changes that you'll see in the next chapter, 
so just bear with me for now.

Now, let's take the code that draws from lib.rs 
and move it into draw , updating it along the way:

Make sure that you add the #[async_trait(?Send)] annotation, which allows
you to implement a trait with the async functions.

```rust
// filename: game.rs

#[async_trait(?Send)]
impl Game for WalkTheDog {

    ...

    fn draw(&self, renderer: &Renderer) {
        let frame_name = format!("Run ({}).png", self.frame + 1);
        let sprite = self.sheet.frames.get(&frame_name).expect("Cell not found");

        renderer.clear(Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 600.0,
                        height: 600.0,
        });

        renderer.draw_image(&self.image,
            Rect {  x: sprite.frame.x.into(),
                    y: sprite.frame.y.into(),
                    width: sprite.frame.w.into(),
                    height: sprite.frame.h.into(),
            },
            Rect {  x: 300.0,
                    y: 300.0,
                    width: sprite.frame.w.into(),
                    height: sprite.frame.h.into(),
            },
        );


    }//^-- draw()
```

This only contains slight changes to the code in lib.rs , although it definitely won't compile. 
Calls to context are replaced with calls to renderer, and we've used the new Rect structure.


We'll need to add that to the game module. <SHOULD BE engine.rs>

```rust
// filename: src/game.rs

#[derive(Deserialize)]
struct SheetRect {
    x: i16,
    y: i16,
    w: i16,
    h: i16,
}

#[derive(Deserialize)]
struct Cell {
    frame: SheetRect, //was named Rect in lib.rs
}

#[derive(Deserialize)]
pub struct Sheet {
    frames: HashMap<String, Cell>,
}

pub struct WalkTheDog {
    image: HtmlImageElement,
    sheet: Sheet,
    frame: u8,
}
```

Here, we've moved the structures from lib.rs that serialize the JSON from our
sprite sheet and added fields for frame , HtmlImageElement , and Sheet to the
WalkTheDog struct. 

Pay close attention to the fact that we've taken Rect from lib and renamed it SheetRect . 
This is the specific rectangle from our sprite sheet. 

In game , we also have a Rect structure. 
This is the rectangle that we'll use as a game domain object.

This rename is confusing right now but is done to differentiate the two rectangles 
and is helpful as we go forward.

---

The WalkTheDog structure has the fields needed to make draw compile, but it may make
you wonder about initialize . Specifically, if we're going to move our loading code to
initialize , does the WalkTheDog struct really always have HtmlImageElement
and Sheet ? No, it does not. We'll need to convert those fields to Option types and make
the draw function account for them:

```rust
// filename: src/game.rs


pub struct WalkTheDog {
    image: Option<HtmlImageElement>,
    sheet: Option<Sheet>,
    frame: u8,
}
```

We can use the `as_ref()` function to borrow image and sheet , and then use the
`and_then` for the sprite and `map` Option functions for `renderer.draw_image` to cleanly get the frame and then draw it:

```rust
// filename: src/game.rs
#[async_trait(?Send)]
impl Game for WalkTheDog {

...

    fn draw(&self, renderer: &Renderer) {
        let frame_name = format!("Run ({}).png", self.frame + 1);
        //let sprite = self.sheet.frames.get(&frame_name).expect("Cell not found");

        let sprite = self.sheet.as_ref()
                               .and_then(|sheet| sheet.frames.get(&frame_name))
                               .expect("Cell not found");

        ...

        self.image.as_ref().map(|image| {
            renderer.draw_image(&self.image,
                Rect {  x: sprite.frame.x.into(),
                        y: sprite.frame.y.into(),
                        width: sprite.frame.w.into(),
                        height: sprite.frame.h.into(),
                },
                Rect {  x: 300.0,
                        y: 300.0,
                        width: sprite.frame.w.into(),
                        height: sprite.frame.h.into(),
                },
            );
        });



```


Let's prepare to draw by copying our loading code from `lib.rs` to the initialize function 
in the game loop. 
Don't do any cutting and pasting yet; we'll go ahead and clean up lib.rs at the end. 

Initialize should now look like this:

```rust
// filename: src/game.rs

/* 
#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        Ok(Box::new(WalkTheDog {}))
    }
*/

#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        //Ok(Box::new(WalkTheDog {}))
        let sheet: Sheet = browser::fetch_json("../resources/pix/rhb.json").await?.into_serde()?;
        let image = Some(engine::load_image("../resources/pix/rhb.png").await?);

        Ok(Box::new(WalkTheDog { image, sheet, frame: self.frame, }))
    }
```


Now that we have initialize and draw , we can write update . 

The version we wrote in lib.rs used 
`set_interval_with_callback_and_timeout_and_arguments_0` to animate our Red Hat Boy, 
but that's not going to work anymore.

Instead, the update function will need to keep track of the number of frames that
have passed and advance when it's appropriate. In the original code, we called the set_
interval callback every 50 milliseconds. In this new code, update will be called
every 1/60th of a second, or 16.7 milliseconds. So, in order to approximately match the
animation, we'll want to update the current sprite frame every three updates; otherwise,
our little RHB will run very, very fast.

If you look at the rhb.json file, you can see that there are 8 frames in the Run
animation. If we want to advance a sprite frame every 3 updates, that means it will take
24 updates to complete the animation. At that point, we'll want to return to the beginning
and play it again. So, we'll need to calculate the sprite frame from the frame count, which
is updated in the update function:

```rust
// filename: src/game.rs

#[async_trait(?Send)]
impl Game for WalkTheDog {
    ...

    fn update(&mut self) {
        if self.frame < 23 {
            self.frame += 1;
        } else {
            self.frame = 0;
        }
    }

    ...
```

This won't work with our current draw code because it uses frame to look up the sprite
to render. It will crash when it looks for Run (9).png , which doesn't exist. 
We'll update the draw function to get the sprite index from frame :

```rust
// filename: src/game.rs

    fn draw(&self, renderer: &Renderer) {
        let current_sprite = (self.frame / 3) + 1;
        let frame_name = format!("Run ({}).png", current_sprite);
    ...
```

The current_sprite variable will cycle from one to eight, and then loop back again.

We now have a game loop that can render to the canvas and
a game that renders our running RHB; we just need to integrate it. We'll add a plain
constructor to the WalkTheDog struct, right under the struct definition in game

```rust
// filename: src/game.rs
...

pub struct WalkTheDog {
    image: Option<HtmlImageElement>,
    sheet: Option<Sheet>,
    frame: u8,
}

impl WalkTheDog {
    pub fn new() -> Self {
        WalkTheDog {
            image: None,
            sheet: None,
            frame: 0,
            position: Point { x: 0, y: 0 },
        }
    }
}

#[async_trait(?Send)]
impl Game for WalkTheDog {

    ...
```

And now for the moment you've been waiting for – the new main function integrating all
these changes:

```rust
// filename: src/lib.rs

#[macro_use]
mod browser;
mod engine;
mod game;

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
```

### Possible error in the book

 sheet in WalkTheDog is now Option<Sheet>

```rust
// filename: src/game.rs

#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        let sheet: Sheet = browser::fetch_json("../resources/pix/rhb.json").await?.into_serde()?;
        let image = Some(engine::load_image("../resources/pix/rhb.png").await?);

        let sheet = Some(sheet); // <--- WalkTheDog now take an Option
        Ok(Box::new(WalkTheDog { image, sheet, frame: self.frame, }))
    }
```

Since draw_image expects a plain &HtmlImageElement 
```rust
// filename src/engine.rs

    pub fn draw_image(&self, 
                        image: &HtmlImageElement, <---- 
                        frame: &Rect, 
                        destination: &Rect) {
    ...
```
we must unwrap it to pass it along but we cant move  it 
so we use `as_ref()` to borrow it and then unwrap.

```rust
#[async_trait(?Send)]
impl Game for WalkTheDog {
    ...
    fn draw(&self, renderer: &Renderer) {
        ...

        self.image.as_ref().map(|image| {
            renderer.draw_image(&self.image.as_ref().unwrap(),
    ...

```

### Adding keyboard input

We'll start listening to keyboard events and use them to control our RHB.
That means adding keyboard input to the game loop and passing that into the update function.

In a normal program, you would listen for keys to get pressed – in other words, 
pushed down and then released – and then do something such as update the screen 
when the button is released. 

This doesn't fit in with a game because typical players want the action to happen 
as soon as a key is pushed down and want it to continue for as long as it's held. 

Think of moving around the screen with the arrow keys. 
You expect motion to start the second you hit the arrow key, not after you release it. 
In addition, traditional programming doesn't account for things like pressing "up" 
and "right" at the same time. 
If we process those as two separate actions, we'll move right, then up, then right, 
and then up, like we're moving up the stairs. 

What we'll do is listen to every keyup and keydown event, 
and bundle that all up into a keystate that stores every currently pressed key. 
Then we'll pass that state to the update function so that the game can figure out 
just what to do with all the currently pressed keys.


To get keyboard events, we have to listen for the keydown and keyup events on canvas .
Let's start with a new function in `engine.rs` , `prepare_input()` :

```rust
fn prepare_input() {
    let onkeydown = browser::closure_wrap( 
                        Box::new(move |keycode: web_sys::KeyboardEvent| {})
                        as Box<dyn FnMut(web_sys::KeyboardEvent)>);
    
    let onkeyup = browser::closure_wrap(Box::new( 
                        move |keycode: web_sys::KeyboardEvent| {})
                        as Box<dyn FnMut(web_sys::KeyboardEvent)>);

    browser::canvas().unwrap()
                     .set_onkeydown(Some(onkeydown.as_ref().unchecked_ref()));

    browser::canvas().unwrap()
                     .set_onkeyup(Some(onkeyup.as_ref().unchecked_ref()));


    onkeydown.forget();
    onkeyup.forget();
}

```

We're setting up Closure objects in the same way we did for `load_image` 
and `request_animation_frame`.
We have to make sure we call forget on both of the Closure instances so that they
aren't deallocated immediately after being set up because nothing in the Rust application
is holding onto them. 
You'll also need in `Cargo.toml` to add the `KeyboardEvent` feature to `web-sys` to include it.

Tip::

        Unlike most things in Rust, if you don't add a forget call, you won't get a
        compile-time error. You'll get a panic almost immediately and not always with
        a helpful error message. If you think you've set up callbacks into JavaScript
        and you're getting panics, ask yourself whether anything is holding on to
        that callback in your program. If nothing is, you've probably forgotten to add
        forget .


We're listening to the input, so now we need to keep track of all of it. 

It's tempting to start trying to condense the events into keystate in this function, 
but that's troublesome because this function only handles one keyup or keydown at a time 
and doesn't know anything about all the other keys. 

If you wanted to keep track of an ArrowUp and ArrowRight being pressed at the same time, 
you couldn't do it here. 

What we will do is set up the listeners once before the game loop starts, 
such as with initialize , and then process all the new key events on every update 
updating our keystate . 

This will mean sharing state from these closures with the closure 
we passed to `request_animation_frame`. 

It's time to add a channel.


We'll create an unbounded channel, which is a channel that will grow forever if you let it, 
here in prepare_input and then return its receiver. 
We'll pass transmitters to both onkeyup and onkeydown , and send the KeyboardEvent 
to each of those.




```rust
fn prepare_input() -> Result<UnboundedReceiver<KeyPress>> {
    let (keydown_sender, keyevent_receiver) = unbounded();
    let keydown_sender = Rc::new(RefCell::new(keydown_sender));
    let keyup_sender = Rc::clone(&keydown_sender);

    /* //Before
    let onkeydown = browser::closure_wrap( 
                        Box::new(move |keycode: web_sys::KeyboardEvent| {})
                        as Box<dyn FnMut(web_sys::KeyboardEvent)>);
     ...
    */
    let onkeydown = browser::closure_wrap(
                        Box::new(move |keycode: web_sys::KeyboardEvent| {
                            keydown_sender.borrow_mut()
                                          .start_send(KeyPress::KeyDown(keycode));
                        }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

    let onkeyup = browser::closure_wrap(
                        Box::new(move |keycode: web_sys::KeyboardEvent| {
                            keyup_sender.borrow_mut()
                                        .start_send(KeyPress::KeyUp(keycode));
                        }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

    /* //Before
    browser::canvas().unwrap()
                     .set_onkeydown(Some(onkeydown.as_ref().unchecked_ref()));

    browser::canvas().unwrap()
                     .set_onkeyup(Some(onkeyup.as_ref().unchecked_ref()));


    onkeydown.forget();
    onkeyup.forget();
    */
    browser::window()?.set_onkeydown(Some(onkeydown.as_ref().unchecked_ref()));
    browser::window()?.set_onkeyup(Some(onkeyup.as_ref().unchecked_ref()));

    onkeydown.forget();
    onkeyup.forget();
}
```

The function now returns `Result<UnboundedReceiver<KeyPress>>`.
`UnboundedReceiver` and `unbounded` are both in the `futures::channel::mspc`
module and are declared in a use declaration at the top of the file. 

```rust
use futures::channel::mspc::{unbounded, UnboundedReceiver};
```

We create the unbounded channel on the first line with the unbounded function 
and then create reference counted versions of both `keydown_sender` and `keyup_sender`, 
so that we can move each of them into their respective closures 
while sending both events to the same receiver. 
Note that the unbounded channel uses `start_send` instead of `send`.
Finally, we return `keyevent_receiver` as `Result`.

KeyPress

It turns out you can't tell what kind of `KeyboardEvent` happened simply 
by inspecting it. In order to keep track of whether the event was keyup or keydown, 
we wrap those events in an enumerated type that we'll define in `engine.rs`:

```rust
enum KeyPress {
    KeyUp(web_sys::KeyboardEvent),
    KeyDown(web_sys::KeyboardEvent),
}
```

This enum approach means we won't have to manage two channels. 
Now that we have a function that will listen for and put all our key events into a channel, 
we need to write a second function that grabs all those events off the channel 
and reduces them into KeyState . 
We can do that like so, still in the `engine` module:

```rust
fn process_input(state: &mut KeyState, keyevent_receiver: &mut UnboundedReceiver<KeyPress>) {
    loop {
        match keyevent_receiver.try_next() {
            Ok(None) => break,
            Err(_err) => break,
            Ok(Some(evt)) => match evt {
                KeyPress::KeyUp(evt) => state.set_released(&evt.code()),
                KeyPress::KeyDown(evt) => state.set_pressed(&evt.code(), evt), 
            },//^-- match evt                   
        };//^-- match 
    }//^-- loop
}//^-- fn process_input
```

This function `process_input` takes `KeyState` and `Receiver` 
and updates state by taking every entry off of the receiver until its empty. 

Theoretically, this appears to create the possibility for an infinite loop in the event 
that the receiver is constantly filled, but I was unable to do that by normal means (pressing the keyboard like a madman), and if somebody decides to write a script that fills this channel 
and break their own game, more power to them.

`KeyState` has to be passed as `mut` so that we update the current one 
and do not start from a brand-new state on each update. 

We've written this function pretending that `KeyState` already exists, 
but we need to create it as well, again in the `engine`

```rust
// filename: src/engine.rs

pub struct KeyState {
    pressed_keys: HashMap<String, web_sys::KeyboardEvent>,
}

impl KeyState {
    fn new() -> Self {
        KeyState {
            pressed_keys: HashMap::new(),
        }
    }

    pub fn is_pressed(&self, code: &str) -> bool {
        self.pressed_keys.contains_key(code)
    }

    fn set_pressed(&mut self, code: &str, event: web_sys::KeyboardEvent) {
        self.pressed_keys.insert(code.into(), event);
    }

    fn set_released(&mut self, code: &str) {
        self.pressed_keys.remove(code.into());
    }

}//^-- impl KeyState
```

The `KeyState` struct is just a wrapper around `HashMap` , storing a lookup of
`KeyboardEvent.code` to its `KeyboardEvent` . 
If the code isn't present, then the key isn't pressed. 
The code is the actual representation of a physical key on the keyboard.


We've created the libraries and structures we need for keyboard input, 
so now we can integrate it into our GameLoop . 

We'll call prepare_input in the start function before we start looping:

```rust
// filename: src/engine.rs

/* //Before
impl GameLoop {
    pub async fn start(mut game: impl Game + 'static) -> Result<()> {
        let mut game = game.initialize().await?;
    ...
*/
impl GameLoop {
    pub async fn start(mut game: impl Game + 'static) -> Result<()> {
        let mut keyevent_receiver = prepare_input()?;
        let mut game = game.initialize().await?;
    ...
```

Then, we'll move `keyevent_receiver` into the `request_animation_frame`
closure and process the input on every update

```rust
// filename: src/engine.rs

/* //Before
impl GameLoop {
    pub async fn start(mut game: impl Game + 'static) -> Result<()> {
    ...
    *g.borrow_mut() = Some(browser::create_raf_closure(move |perf: f64| {

            game_loop.accumulated_delta += (perf - game_loop.last_frame) as f32;
            while game_loop.accumulated_delta > FRAME_SIZE {
                ...
        
*/

    let mut keystate = KeyState::new();

    *g.borrow_mut() = Some(browser::create_raf_closure(move |perf: f64| {
        process_input(&mut keystate, &mut keyevent_receiver);

        game_loop.accumulated_delta += (perf - game_loop.last_frame) as f32;
        while game_loop.accumulated_delta > FRAME_SIZE {
```

You can see that we initialized an empty `KeyState` right before 
the `request_animation_frame` closure, so that we can start with an empty one. 

Each frame will now call our `process_input` function and <u>generate a new KeyState</u> . 
That's all the changes we have to do to our game loop to keep track of KeyState . 

The only thing that's remaining is to pass it to our `Game` object so that it can be used. 
Some game implementations will store this as a global, 
but we'll just pass it to the `Game` trait. 

We'll update the trait's update function to accept KeyState :

```rust
// filename: src/engine.rs

/* Before
#[async_trait(?Send)]
pub trait Game {
    async fn initialize(&self) -> Result<Box<dyn Game>>;
    fn update(&mut self);
    fn draw(&self, context: &Renderer);
}
*/

#[async_trait(?Send)]
pub trait Game {
    async fn initialize(&self) -> Result<Box<dyn Game>>;
    fn update(&mut self, keystate: &KeyState);
    fn draw(&self, context: &Renderer);
}
```

Now, we can pass KeyState to the update function on every loop:

```rust
impl GameLoop {
    pub async fn start(mut game: impl Game + 'static) -> Result<()> {
    ...
        /* Before
            while game_loop.accumulated_delta > FRAME_SIZE {
                game.update();
                game_loop.accumulated_delta -= FRAME_SIZE;
            }
        */

            while game_loop.accumulated_delta > frame_size {
                game.update(&keystate);
                game_loop.accumulated_delta -= frame_size;
            }
...
```
Finally, to keep our game compiling, we will need to update the `WalkTheDog::update`
signature, over in the `game` module, to match and bring KeyState in scope:

```rust
// filename: src/game.rs
use crate::engine::KeyState;

#[async_trait(?Send)]
impl Game for WalkTheDog {
    ...
    fn update(&mut self, keystate: &KeyState) {
        ...
```

> We've got a GameLoop that processes keyboard input  and passes that state to our Game .


### Moving Red Hat Boy

We'll create a Point structure in engine that will hold an x and a y position for RHB. 
On every update call, we'll also calculate a velocity for him, based on which keys are pressed. 
Every direction will be the same size, so if ArrowLeft and ArrowRight are pressed at the same time, he'll stop moving. 

After we calculate his velocity, we'll update his position with that number. 
That should be enough to allow us to move him around the screen. 
Let's start by adding position to the WalkTheDog game struct:

```rust
// filename:game.rs
use crate::engine::{ ...,
                     KeyState,
                     Point
                    };

pub struct WalkTheDog {
    image: Option<HtmlImageElement>,
    sheet: Option<Sheet>,
    frame: u8,
    position: Point,
}
```

We put Point in engine:

```rust
// filename: src/engine.rs

#[derive(Clone, Copy)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

```
Note that we're using integers here so that we don't have to deal with floating point
math when it's not necessary. While the canvas functions all take f64 values, that's
only because there is only one number type in JavaScript , and canvas is faster 
if you use integer coordinates. 

You'll also need to update the `WalkTheDog::new` function in `game.rs` to fill in a default position. 
Let's use 0, 0 for now:

```rust
// filetype: src/game.rs

impl WalkTheDog {
    pub fn new() -> Self {
        WalkTheDog {
            image: None,
            sheet: None,
            frame: 0,
            position: Point {x: 0, y: 0},
        }
    }
}
```

The initialize function also needs to be updated to account for position. 
This is actually why we marked Point with Clone and Copy. 
It makes it possible to copy it into the new WalkTheDog initialize function:

```rust
// filename: src/game.rs
...
/* Before
        Ok(Box::new(WalkTheDog { image, sheet, frame: self.frame,}))
*/

#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        let sheet: Sheet = browser::fetch_json("../resources/pix/rhb.json").await?.into_serde()?;
        let image = Some(engine::load_image("../resources/pix/rhb.png").await?);
        let sheet = Some(sheet);

        Ok(Box::new(WalkTheDog { image, sheet, frame: self.frame, position: self.position, }))
    }
...
```


In order for position to have any meaning, we'll need to update the draw function so
that it's actually being used with:
```
x: self.position.x.into(),
y: self.position.y.into(),
```

```rust
// filename: src/game.rs

#[async_trait(?Send)]
impl Game for WalkTheDog {
    ...
    fn draw(&self, renderer: &Renderer) {
    ...
        self.image.as_ref().map(|image| {
            renderer.draw_image(&self.image.as_ref().unwrap(),
                &Rect {  x: sprite.frame.x.into(),
                ...
                },
                &Rect { x: self.position.x.into(), //x: 300.0,
                        y: self.position.y.into(), //y: 300.0,
                        width: sprite.frame.w.into(),
                        height: sprite.frame.h.into(),
                },
```


NOTE: The first Rect is the slice we are taking out of our sprite sheet. 
The second one is where we want to draw it. 
This should cause a noticeable change to the game, as RHB is now in the upper-left corner.

Finally, we're going to modify update to calculate a velocity based 
on which keys are pressed in KeyState. 

We'll add this before updating the current frame, as shown here:

```rust
// filename: src/game.rs

/*Before
#[async_trait(?Send)]
impl Game for WalkTheDog {
    ...

    fn update(&mut self, keystate: &KeyState) {
        if self.frame < 23 {
            self.frame += 1;
        } else {
            self.frame = 0;
        }
    }

*/
    fn update(&mut self, keystate: &KeyState) {
        let mut velocity = Point { x: 0, y: 0 };
        
        if keystate.is_pressed("ArrowDown") { velocity.y += 3; }
        if keystate.is_pressed("ArrowUp") { velocity.y -= 3; }
        if keystate.is_pressed("ArrowRight") { velocity.x += 3; }
        if keystate.is_pressed("ArrowLeft") { velocity.x -= 3; }
        if keystate.is_pressed("ArrowLeft") { velocity.x -= 3; }
        
        // Next, we adjust the position based on velocity
        self.position.x += velocity.x;
        self.position.y += velocity.y;
    }
```

The " ArrowDown " and " ArrowUp " strings and so on are all listed at
[Code values for keyboard events](https://developer.mozilla.org/en-US/docs/Web/API/UI_Events/Keyboard_event_code_values)
You can see here that if " ArrowDown " is pressed we increase y , and if
" ArrowUp " is pressed, we decrease it, and that's because the origin is in the upper-left-
hand corner, with y increasing as you go down, not up. Note also that we don't use if/
else here. We want to account for every pressed key and not short-circuit on the first key
that's pressed. Next, we adjust the position based on velocity:

```rust
// filename: src/game.rs
        ...
        if keystate.is_pressed("ArrowLeft") { velocity.x -= 3; }
        
        self.position.x += velocity.x;
        self.position.y += velocity.y;
    }
```


Head back to the webbrowser, and you can now use the arrow keys to move RHB around! If
he doesn't move, make sure you click in the canvas to give it focus. If he still doesn't move
and you're sure you've gotten everything right, put some log! messages in the start
function and make sure KeyState is being created, or in the update function to see if
you're actually getting a new KeyState .

