## Loading assets

Expanding our game loop to handle loading assets is going 
to require adding a function to our trait, an async one to be precise. 
This will allow us to put all our asynchronous code 
that's currently wrapped in the spawn_local in lib 
and put it in a function that returns Result with Game in it.

NOTE:
    async trait functions haven't landed in stable Rust yet.
    > cargo add async-trait

```
// filename: src/engine.rs
use async_trait::async_trait;
```

```rust
#[async_trait(?Send)]
pub trait Game {
    async fn initialize(&self) -> Result<Box<dyn Game>>;
    fn update(&mut self);
    fn draw(&self, context: &Renderer);
}
```

```rust
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
Currently, we're sending a raw CanvasRenderingContext2d to the draw loop, with
all of its awkward functions such as 
draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh . 

This works but it's ugly, and much like we did with the browser module, we can use a wrapper to narrow the context's wide interface to a smaller one, tailored to our needs. 
We'll replace passing CanvasRenderingContext2d with our own Renderer object that has easier-to-use functions.
We'll start by creating a structure for our Renderer in engine :

```rust
pub struct Renderer {
    context: CanvasRenderingContext2d,
}

```

This is a simple wrapper containing the rendering context. 
For now, we'll just add the two


```rust
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
do so using fewer parameters. Instead of four parameters and clear_rect , we pass
clear Rect . Instead of that incredibly long function name, we pass draw_image
HtmlImageElement and two Rect structures. Currently, we go ahead and use expect
to panic! here if we can't draw.

Of course, both of these functions take Rect , but we don't have a Rect structure. Let's add that to the engine

```rust
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

```

Note: we replaced 
    game.draw(&browser::context().expect("Context should exist")); 
with 
    fn draw(&self, renderer: &Renderer);


Let's take our animation code out of lib and using the game loop.

### Integrating the game loop

It's great that we've written this game loop and all, but it's about time we actually use it.

Remember that we have our GameLoop structure, but it operates on a Game trait. 
So in order to use the loop, we need to implement that trait. 
We'll implement it in another module, game , which we'll create in game.rs and then add to the library using the mod game instruction declaration in lib.rs .


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




