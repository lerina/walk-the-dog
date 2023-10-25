## Refactoring for endless running

### Sharing a sprite sheet

Each `Platform` has a reference to an `Image` and a `Sheet` that we've casually been
referring to as "the sprite sheet." When we start generating more `Platform` objects,
we'll want to share a reference to the sheet. So, the time has come to add
a `SpriteSheet` struct to our engine to enable that. 

Let's open the `engine` module and add that new concept.

#### Creating a sprite sheet

We will start by creating a struct that holds both `HtmlImageElement` and `Sheet` 
in the `engine` module:


```rust
// src/engine.rs

pub struct SpriteSheet {
    sheet: Sheet,
    image: HtmlImageElement,
}

```

Now, let's create an implementation that will wrap the common behaviors of the sheet that
we're using in Platform :

```rust
// src/engine.rs

impl SpriteSheet {
    pub fn new(sheet: Sheet, image: HtmlImageElement) ->
        Self { SpriteSheet { sheet, image } }
    }

    pub fn cell(&self, name: &str) -> Option<&Cell> {
        self.sheet.frames.get(name)
    }
    pub fn draw(&self, renderer: &Renderer, source: &Rect, destination: &Rect) {
        renderer.draw_image(&self.image, source, destination);
    }
}

```


I initially considered having draw take the name of the cell property we were drawing,
but right now, our Platform draws more than one cell at a time, and we want to keep
that functionality. 

Let's replace `HtmlImageElement` and `Sheet` in `Platform` with the `SpriteSheet` field, 
as shown here:


```rust
// src/game.rs

pub struct Platform {
    sheet: SpriteSheet,
    position: Point,
}

```

Don't forget to import SpriteSheet from the engine module. 


```rust
// src/game.rs

use crate::{
    browser,
    engine::{self, Cell, Game, Image, KeyState, Point, Rect, Renderer, Sheet, SpriteSheet,},
};


```


Now, you can follow the compiler to simplify `Platform` by removing references 
to `Sheet` and `HtmlImageElement` and just using `SpriteSheet`. 
In particular, you'll need to change the `new` function so that it takes one `SpriteSheet` instead 
of the two parameters.

The following code shows how this can be initialized in the initialize method of WalkTheDog :


```rust
// src/game.rs

impl Game for WalkTheDog {
    async fn initialize(&mut self) -> Result<Box<dyn Game>> {
        match self {
            WalkTheDog::Loading => {
                ...
                /*                
                let platform = Platform::new( platform_sheet.into_serde::<Sheet>()?,
                                              engine::load_image("../resources/pix/tiles.png").await?,
                                              Point { x: 200, y: 400 },
                               );
                */

                let platform = Platform::new(  
                                   SpriteSheet::new(platform_sheet.into_serde::<Sheet>()?,
                                                    engine::load_image("../resources/pix/tiles.png").await?, 
                                                  ),
                                    Point { x: 200, y: 400 },
                               );
```

The rest of Platform can be modified to fit the new interface. 
Note how you no longer need to say `frames` and can just call `sheet.cell`. 

The `draw` method will now delegate to `self.sheet.draw` and pass it the `renderer` instead of an `Image`. 
This structure is small and wouldn't be worth the effort if we didn't want to share the same `SpriteSheet`
across multiple `Platform` objects. 

But we do want to share one `SpriteSheet`, instead of duplicating that memory everywhere. 
Due to this, we need to make it possible to share it.


#### Sharing a sprite sheet

To share SpriteSheet across more than one Platform , we'll need to store it
somewhere that all of the platforms can point to, and designate something to be the owner
of SpriteSheet . We could give SpriteSheet a static lifetime, and make it global,
but that would mean making it an Option since it's not available until initialize
is used. Instead, we'll store a reference-counted version of SpriteSheet in the Walk
structure. This is a tradeoff since we'll be using reference counting instead of ownership
to track when we should delete SpriteSheet , but in exchange, we'll only be duplicating
the pointer in memory instead of an entire SpriteSheet .
Let's add obstacle_sheet to the Walk struct, as shown here:


```rust
// src/game.rs

struct Walk {
    obstacle_sheet: Rc<SpriteSheet>,
    ...
}
```

You'll need to make sure you add use std::rc::Rc to the top of the game
module. We'll also need to make sure that Platform can take a reference-counted
SpriteSheet instead of taking ownership of SpriteSheet , as shown here:

```rust
// src/game.rs

pub struct Platform {
    sheet: Rc<SpriteSheet>,
    ...
}

```
Don't forget to bring `Rc` it into scope:

```rust
// src/game.rs

use std::rc::Rc;
```

Change the corresponding builder

```rust
// src/game.rs

impl Platform {
    pub fn new(sheet: Rc<SpriteSheet>, position: Point) -> Self {
        Platform { sheet, position }
    }
    ...

```

Here, we're replacing `SpriteSheet` with `Rc<SpriteSheet>`. 

This leaves us with one last modification we need to make – we must initialize 
the `Walk` struct and set up `obstacle_sheet` and the `platform`, as shown here:

Here is the old `initialize`:

```rust
// src/game.rs

#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        match self {
            WalkTheDog::Loading => {
                let sheet = browser::fetch_json("../resources/pix/rhb.json").await?.into_serde()?;
                let rhb = RedHatBoy::new(sheet, engine::load_image("../resources/pix/rhb.png").await?);
                let background = engine::load_image("../resources/pix/BG.png").await?;
                let stone = engine::load_image("../resources/pix/Stone.png").await?;
                let platform_sheet = browser::fetch_json("../resources/pix/tiles.json").await?;

                let platform = Platform::new(  
                                   SpriteSheet::new(platform_sheet.into_serde::<Sheet>()?,
                                                    engine::load_image("../resources/pix/tiles.png").await?, 
                                                  ),
                                    Point { x: 200, y: 400 },
                               );

                let background_width = background.width() as i16;
                let backgrounds = [ Image::new( background.clone(), Point { x: 0, y: 0 }),
                                    Image::new( background, Point { x: background_width, y: 0,},),
                                  ];


                let obstacles = vec![ Box::new(Barrier::new(
                                                 Image::new( stone, Point { x: 150, y: 546 }))),
                                      Box::new(platform),
                                            ];

                let walk = Walk {   boy: rhb, 
                                    backgrounds: backgrounds,
                                    //stone: Image::new(stone, Point { x: 150, y: 546 }),
                                    //platform: Box::new(platform), //platform: platform,
                                    obstacles: obstacles, 
                                };

                Ok(Box::new(WalkTheDog::Loaded(walk)))
            },
            WalkTheDog::Loaded(_) => Err(anyhow!("Error: Game is already initialized")),
        }
    }//^-- async fn initialize

```

And here is our new `initialize`:


```rust
// src/game.rs

#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        match self {
            WalkTheDog::Loading => {
                ...
                // change of name
                // let platform_sheet = browser::fetch_json("../resources/pix/tiles.json").await?;
                let tiles = browser::fetch_json("../resources/pix/tiles.json").await?;

                let sprite_sheet = Rc::new(SpriteSheet::new(
                                    tiles.into_serde::<Sheet>()?,
                                    engine::load_image("tiles.png").await?,
                                   ));
                
                /*
                let platform = Platform::new(  
                                   SpriteSheet::new(platform_sheet.into_serde::<Sheet>()?,
                                                    engine::load_image("../resources/pix/tiles.png").await?, 
                                                  ),
                                    Point { x: 200, y: 400 },
                               );        
                */
                let platform = Platform::new(
                                    sprite_sheet.clone(),
                                    Point {
                                        x: FIRST_PLATFORM,
                                        y: LOW_PLATFORM,
                                    },
                                );
                ...
                
                /*
                let walk = Walk {   boy: rhb, 
                                    backgrounds: backgrounds,
                                    obstacles: obstacles,
                                };                
                */
                let walk = Walk {   boy: rhb, 
                                    backgrounds: backgrounds,
                                    obstacles: obstacles,
                                    obstacle_sheet: sprite_sheet, 
                                };

                Ok(Box::new(WalkTheDog::Loaded(walk)))
            }
            ...
```

Two sections change in initialize. 

First, after we call `fetch_json` to get `tiles.json`, 
we use that to create a **reference-counted** SpriteSheet named `sprite_sheet`
with `Rc::new`. 
Note that we've replaced `let platform_sheet` with `let tiles` because 
that's a better name – it's loading `tiles.json` after all. 

Then, when we create `platform` with `Platform::new`, 
we pass it a clone of the created `sprite _sheet`.

Previously, this was done inline, but we're going to need `sprite_sheet` again in a minute.

Then, when we're creating the `Walk` struct, we need to pass that created sheet to the
`obstacle_sheet` field. 
This doesn't need to be cloned because `Walk` is the **ultimate owner** of `sprite_sheet`, 
so `sprite_sheet` can be moved into it. 

This will increment the reference counter and will not clone the entire `SpriteSheet`. 
We will need to clone `obstacle_sheet` every time we create a `Platform` to ensure 
the references are counted correctly, but don't worry about this – the compiler 
will force us to do this.

With that, we're now ready to reevaluate how our `Platform` object works. 

Currently, it can only create one Platform, but there's no reason it can't create many things 
the player can stand on. 

We'll want that as we generate levels. We'll do that next.


------------


```rust
// src/game.rs



```

