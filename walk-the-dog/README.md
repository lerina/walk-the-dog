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


We'll need to add that to the game module.

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
















