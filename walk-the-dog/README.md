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

### Many different platforms

The current `Platform` struct assumes it's using the same three cells in the sprite sheet,
including calculating the bounding boxes. So, to allow many kinds of platforms to be used,
we'll need to pass in the cells we want to be rendered from the sheet, and we'll need to pass
in custom bounding boxes for each potential `Platform`. 

For example, imagine that you wanted to take the provided tileset ( tiles.json ) 
and arrange them into a little cliff:

![Look out below!](./readme_pix/cliff.png)

This would require passing the 11, 2, and 3 platform tiles. 
Those tiles aren't arranged horizontally or neatly, 
and the bounding boxes don't match our other platform. 

When we create this platform, we'll need to look up the tile dimensions in `tiles.json` 
and work out the bounding boxes from the provided dimensions manually. 

This means changing the way `Platform` works so that it's less specific.

Let's start by changing the Platform struct so that it can hold the bounding boxes 
and a list of the sprites, as shown here:


```rust
// src/game.rs

/*
pub struct Platform {
    //sheet: SpriteSheet,
    sheet: Rc<SpriteSheet>,
    position: Point,
}
*/

pub struct Platform {
    sheet: Rc<SpriteSheet>,
    bounding_boxes: Vec<Rect>,
    sprites: Vec<Cell>,
    position: Point,
}

```

While we're changing `Platform` to make it less specific, 
we're also going to introduce an optimization: 
Platform will hold the **sprite cells** instead of looking them up every
time they are drawn. 
There are two optimizations here because we are also storing the **bounding boxes** 
for `Platform` instead of calculating them every time they're created.

This change will break pretty much everything in the implementation of `Platform`, 
most notably the new `constructor`, which will need to take a list of sprite names 
and bounding
boxes and then convert the sprite names into cells, as shown here:


```rust
// src/game.rs

impl Platform {
    pub fn new( sheet: Rc<SpriteSheet>, position: Point,
                sprite_names: &[&str], bounding_boxes: &[Rect],
              ) -> Self {

        let sprites = sprite_names
                            .iter()
                            .filter_map(|sprite_name|
                            sheet.cell(sprite_name).cloned())
                            .collect();
        ...

```

This isn't the entire new method, just the beginning. 

We started by changing the signature so that it takes four parameters. 
`sheet` and `position` were already there but the new method now 
takes a list of sprite names as a reference to an array of string slices. 

You can take a Vec of String objects, but it's a lot nicer to use the reference to string slices 
because it's much easier to call it. 
Clippy will also object to the code taking a `Vec<String>`, which we will cover in Chapter 9, 
Testing, Debugging, and Performance.


The first thing we do in the constructor is to use an iterator to look up every `Cell` in
the `sprite sheet` via the `filter_map` call. 
We use `filter_map` instead of `map` because `sheet.cell` can return `None`, 
so we'll need to skip any invalid sprite names. 
`filter_map` combines `filter` and `map` to automatically reject any options that have a value of
`None` but map the inner value if it is present. 
The cloned method on Option will return an `Option<T>` for any `Option<&T>` by cloning the inner value. 
We use this to take ownership of the inner Cell . Let's continue with our constructor:


```rust
// src/game.rs

impl Platform {
    pub fn new( sheet: Rc<SpriteSheet>, position: Point,
                sprite_names: &[&str], bounding_boxes: &[Rect],
              ) -> Self {

        let sprites = sprite_names
            ...

        let bounding_boxes = bounding_boxes
                                .iter()
                                .map(|bounding_box| {
                                    Rect::new_from_x_y(
                                        bounding_box.x() + position.x,
                                        bounding_box.y() + position.y,
                                        bounding_box.width,
                                        bounding_box.height,
                                    )
                                })
                                .collect();

        Platform {
            sheet,
            position,
            sprites,
            bounding_boxes,
        }
    }//^-- fn new
...
}// impl Platform
```


We continue by taking the passed-in bounding boxes, which are of the &[Rect]
type, and converting them into a Vec<Rect> to be owned by the Platform struct.
However, instead of just calling collect or to_owned , we take each Rect and adjust
its position by the actual position of Platform . So, bounding_boxes will need
to be passed in relative to its image, where the image starts at (0,0) . Imagine that the
image you're drawing is positioned in the top-left corner. The bounding boxes are then
"drawn" around them, skipping any transparency that's relative to the top-left corner.
Then, everything is moved to the right spot in the game. That's the mental model I use to
prevent confusion when I'm specifying the bounding boxes later.



Tip::
    
    Having four parameters is a lot for a constructor, so you should probably
consider replacing this code with the Builder pattern. We did not do this
here because it would distract from the topic at hand, but it is a worthwhile
code improvement. For an example of this, take a look at 
[the unofficial Rust Design Patterns book here:](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html)



You'll also need to change the function for retrieving bounding_boxes , which gets
a lot smaller:
    

```rust
// src/game.rs

/*
    fn bounding_boxes(&self) -> Vec<Rect> {
        const X_OFFSET: i16 = 60; 
        const END_HEIGHT: i16 = 54; 
        let destination_box = self.destination_box();

        let bounding_box_one = Rect {
            position: Point {
                x: destination_box.x(),
                y: destination_box.y()},
            width: X_OFFSET,
            height: END_HEIGHT,
        };
        let bounding_box_two = Rect {
            position: Point { 
                x: destination_box.x() + X_OFFSET,
                y: destination_box.y()
            },
            width: destination_box.width - (X_OFFSET * 2),
            height: destination_box.height,
        };

        let bounding_box_three = Rect {
            position: Point {
                x: destination_box.x() + destination_box.width - X_OFFSET,
                y: destination_box.y()
            },
            width: X_OFFSET,
            height: END_HEIGHT,
        };

        vec![bounding_box_one, bounding_box_two, bounding_box_three]
    }//^-- fn bounding_boxes
*/


fn bounding_boxes(&self) -> &Vec<Rect> {
    &self.bounding_boxes
}
```

Well, that was a lot easier! 
Make sure you return a **reference** to Vec and not a Vec instance. 

We don't need to make any more calculations here; `Platform` is being passed its bounding boxes. 
The rest of the implementation for `Platform` won't be so easy, 
as we'll need to modify `move_horizontally` and `draw` to account for these changes. 

The change that needs to be made to `move_horizontally` is shown here:


```rust
// src/game.rs

impl Obstacle for Platform {
    ...
/*
    fn move_horizontally(&mut self, x: i16) {
        self.position.x += x;
    }
*/

    fn move_horizontally(&mut self, x: i16) {
        self.position.x += x;
        self.bounding_boxes.iter_mut()
                           .for_each(|bounding_box| {
                                bounding_box.set_x(bounding_box.position.x + x);
                            });
    }

```


The original code only moved `position` because `bounding_boxes` was calculated on demand. 

Now that `bounding_boxes` is stored on `Platform`, 
this needs to be adjusted every time we move `Platform`. 
Otherwise, you'll have images for `Platform` in one place, 
the bounding boxes in another, and very strange bugs. 

Ask me how I know.

Finally, let's update the `draw` function for the new structure. 
Whereas the original implementation assumed that it was three cells wide 
and looked up each cell on each draw, the new implementation will loop 
through every cell and draw it individually. 
It will also need to account for the width of each cell. 
So, if the cells are 50 pixels wide, then the first cell will be positioned at 0, 
the second at 50, and so on:

```rust
// src/game.rs

impl Obstacle for Platform {
    ...
    fn draw(&self, renderer: &Renderer) {
        let mut x = 0;
        self.sprites.iter().for_each(|sprite| {
            self.sheet.draw(
                renderer,
                &Rect::new_from_x_y(
                    sprite.frame.x,
                    sprite.frame.y,
                    sprite.frame.w,
                    sprite.frame.h,
                ),
                // Just use position and the standard
                widths in the tileset
                &Rect::new_from_x_y(
                    self.position.x + x,
                    self.position.y,
                    sprite.frame.w,
                    sprite.frame.h,
                ),
            );
            x += sprite.frame.w;
        });
    }//^-- fn draw
```

This isn't my favorite code in the world, but it gets the job done. 

It starts by creating a local, temporary `x` that will calculate the offset from `position` for each `Cell`. 
Then, it loops through the **sprites**, drawing each one but adjusting them for both `position` and `x`. 
Note how, in the destination `Rect`, we advance the `x` position with `self.position.x + x`. 
This ensures each `cell` is drawn to the right of the previous one.

Finally, we calculate the next `x` position based on the `width` of `cell`. 
This implementation of draw does not use the `destination_box` method, 
which means nobody uses it, and you can safely delete it.


```rust
// src/game.rs

impl Platform {
    ...

/* 
//No longer used using mutiple platform
    fn destination_box(&self) ->Rect {
        let platform = self
                        .sheet
                        .frames
                        .get("13.png")
                        .expect("13.png does not exist");
        
        Rect {
            position: Point{ x: self.position.x.into(), y: self.position.y.into()},                       
            width: (platform.frame.w * 3).into(),
            height: platform.frame.h.into(),
        }
    }//^-- destination_box
*/
...

```
 

Note::
    

    This code assumes that width is variable but height is constant and that
    the sprites move from left to right. Here, a two-level platform would need to be
    constructed with two platforms.
    

Platform should now work with any list of sprites that we can construct it with. 

Now, all we need to do is initialize `Platform` properly in `WalkTheDog::initialize`, 
as shown here:


```rust
// src/game.rs

impl Game for WalkTheDog {
    async fn initialize(&mut self) -> Result<Box<dyn Game>> {
        match self {
            WalkTheDog::Loading => {
                ...
                let platform = Platform::new(
                                sprite_sheet.clone(),
                                Point {
                                    x: FIRST_PLATFORM,
                                    y: LOW_PLATFORM,
                                },
                                &["13.png", "14.png", "15.png"],
                                &[
                                    Rect::new_from_x_y(0, 0, 60, 54),
                                    Rect::new_from_x_y(60, 0, 384 - (60 * 2), 93),
                                    Rect::new_from_x_y(384 - 60, 0, 60, 54),
                                ],
                );
                ...
```

With that, Platform has been created with two more parameters – the list of tiles and
the list of bounding boxes – making up the platform we've had all along. Notice that we
can now pass in a simple array of strings for the names of the sprites. 
This is because we accept the `&[&str]` type as a parameter instead of a `Vec<String>`. 

You may be wondering where I got the three bounding box rectangles from. 
After all, previously, we were calculating them in the `bounding_boxes` method, using offsets. 
I simply looked in `tiles.json` and did the math, factoring in the offsets we used earlier. 
These are the same measurements as the bounding boxes were when we calculated them. 

You may also be wondering why these don't use constants, especially after I extolled the virtues of using
constants in Chapter 5, Collision Detection. 
That's because we're going to create those in the next section.
At this point, you should be back to where you started – with RHB waiting to jump over
a rock. 
Now, we are ready to create a stream of dynamic segments. At the end of the next
section, you'll have the constructs you will need for an endless runner.


We need  `FIRST_PLATFORM`  and `LOW_PLATFORM`

```rust
// src/game.rs

use std::rc::Rc;
...

const HEIGHT: i16 = 600;

const FIRST_PLATFORM: i16 = 200;
const LOW_PLATFORM: i16 = 400;
```
for now.


---------

```rust
// src/game.rs



```





