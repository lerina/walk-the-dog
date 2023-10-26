## Creating a dynamic level

The initial screen we've been looking at for so long, with RHB jumping from a stone onto
a platform, is what we're going to call a "*segment*." 
It's not a technical term, just a concept we've made up for the sake of generating them. 
As RHB moves to the right (that is, when all the obstacles move to the left), 
we'll generate new segments to the right, which is just off screen. 
We'll create these as segments so that we can control what is generated and how they fit together. 

Think of it like this: if we generated obstacles at random, then our platforms would look messy 
and would arrange themselves in an unbeatable fashion, like so:

![A truly random level](./readme_pix/truly_random.png)


Instead, what we'll do is create a segment where the first one looks exactly like our one
platform and one rock, and have them string together via a "*timeline*" value that's stored
in `Walk`. 

This `timeline` will represent the right-hand side of the last segment in `x`. 
As that value gets closer to the edge of the screen, we'll generate another new `segment` 
and move the `timeline` back out. 

With this approach, `RHB` will be able to run for as long as we like, and we will have the freedom 
of a level designer. We will be able to create segments that are both easy and hard to navigate, 
though we'll need to make sure they all interlock and can be beaten. 
This is the fun part!

### Creating one segment

We'll start by taking the introductory screen and creating it as a `segment`. 
Let's do this by creating a new file called `segments.rs`, 
making sure to add `mod segments` to the `lib.rs` file. 
This module isn't created for the typical software design reasons; 
usually, it's because `game.rs` is getting pretty long 
and these segments are closer to being levels than they are true code.

```sh
$ touch src/segments.rs
```

```rust
// src/lib.rs

...
mod game;
mod segments;
...
use engine::GameLoop;
...
```

Each segment will be a function that returns a list of obstacles. Let's create a public
function in segments.rs that returns the same list that the game is initialized with:


```rust
// src/segments.rs

pub fn stone_and_platform( 
            stone: HtmlImageElement, 
            sprite_sheet: Rc<SpriteSheet>, 
            offset_x: i16, ) -> Vec<Box<dyn Obstacle>> {

        const INITIAL_STONE_OFFSET: i16 = 150;
        vec![
            Box::new(Barrier::new(Image::new(
                stone,
                Point {
                    x: offset_x + INITIAL_STONE_OFFSET,
                    y: STONE_ON_GROUND,
                },
            ))),
            Box::new(create_floating_platform(
                sprite_sheet,
                Point {
                    x: offset_x + FIRST_PLATFORM,
                    y: LOW_PLATFORM,
            },
            )),
        ]
}

```

Look, constants! 
We want the segments module to look as data-driven as possible, 
so we'll be using constants throughout this file. 

This section of code doesn't compile because the `create_floating_platform` function doesn't exist yet, 
but it does the same things that the corresponding code in the `initialize` method of `WalkTheDog` does. 
The only differences are that it uses the `create_floating_platform` function, which
doesn't exist, and some constants that also do not exist.

The function itself takes `HtmlImageElement` from `stone` and `Rc<SpriteSheet>`
to create `Barrier` and `Platform`, respectively, but it also takes an `offset_x`
value. 
That's because while the first `Barrier` and `Platform` may be at 150 and
200, respectively, in the future, we'll want those to be that many pixels away from the
`timeline`. 

It returns a vector of `obstacles`, which we can use in the `initialize` method of
`WalkTheDog` and anywhere else that we generate segments.


Information::
        
        You may have noticed that we used an Rc for SpriteSheet but just take
        ownership of HtmlImageElement , which may need to be cloned when it's
        called. Nice catch! You may wish to consider making HtmlImageElement
        an Rc as well. HtmlImageElement is small enough that it's probably
        fine if we clone it, but it may be worth investigating in Chapter 9,
        Testing, Debugging, and Performance.
        

Let's continue by creating the function that's missing – that is, 
`create_floating_platform`:


```rust
// src/segments.rs

fn create_floating_platform(sprite_sheet: Rc<SpriteSheet>, 
                            position: Point) -> Platform {
    Platform::new(
        sprite_sheet,
        position,
        &FLOATING_PLATFORM_SPRITES,
        &FLOATING_PLATFORM_BOUNDING_BOXES,
    )
}

```

This is a pretty small function in that it just delegates to the `Platform` constructor 
and passes along important information. 
As you can see, there are two new constants to go along with the others in `stone_and_platform`.


Tip::
        
    If you want to use `Rect::new_from_x_y` when you're declaring
    `FLOATING_PLATFORM_BOUNDING_BOXES`, you'll need to declare it 
    and `Rect::new` as `pub const fn`.


The rest of the segments module consists of constants and use statements. 
You can infer the values for all the constants from the code we used earlier, 
or just check out https://github.com/PacktPublishing/Game-Development-with-Rust-
and-WebAssembly/blob/chapter_6/src/segments.rs . 
By putting all the values in constants, the code looks increasingly data-driven, 
with functions just returning the data we want for every segment.

```rust
// src/segments.rs
use std::rc::Rc;
use web_sys::HtmlImageElement;


use crate::game::{Barrier, Obstacle, Platform};
use crate::engine::{Image, Point, Rect, SpriteSheet};

const LOW_PLATFORM: i16 = 420;
const HIGH_PLATFORM: i16 = 375;
const FIRST_PLATFORM: i16 = 370;
const STONE_ON_GROUND: i16 = 546;

const FLOATING_PLATFORM_SPRITES: [&str; 3] = ["13.png", "14.png", "15.png"];
const PLATFORM_WIDTH: i16 = 384;
const PLATFORM_HEIGHT: i16 = 93;
const PLATFORM_EDGE_WIDTH: i16 = 60;
const PLATFORM_EDGE_HEIGHT: i16 = 54;
const FLOATING_PLATFORM_BOUNDING_BOXES: [Rect; 3] = [
    Rect::new_from_x_y(0, 0, PLATFORM_EDGE_WIDTH, PLATFORM_EDGE_HEIGHT),
    Rect::new_from_x_y(
        PLATFORM_EDGE_WIDTH,
        0,
        PLATFORM_WIDTH - (PLATFORM_EDGE_WIDTH * 2),
        PLATFORM_HEIGHT,
    ),
    Rect::new_from_x_y(
        PLATFORM_WIDTH - PLATFORM_EDGE_WIDTH,
        0,
        PLATFORM_EDGE_WIDTH,
        PLATFORM_EDGE_HEIGHT,
    ),
];


```
    
Once you've filled in the constants and the use statements, 
you can use the new `stone_and_platform` function in the `initialize` method of `WalkTheDog`. 
Yeah, that one again. 
Let's replace the hardcoded list of obstacles with a call to this new function:


```rust
// src/game.rs


#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        match self {
            WalkTheDog::Loading => {
                ...
                /*
                let obstacles = vec![ Box::new(Barrier::new(
                                                 Image::new( stone, Point { x: 150, y: 546 }))),
                                      Box::new(platform),
                                            ];
                */
                let obstacles = stone_and_platform(stone, sprite_sheet.clone(), 0);
```


Make sure you bring in scope stone_and_platform from segments! 


```rust
// src/game.rs

use crate::{
    browser,
    engine::{self, Cell, Game, Image, KeyState, Point, Rect, Renderer, Sheet, SpriteSheet},
    segments::stone_and_platform,
};

```

Now that we've got a function to create the initial scene, 
we can add a timeline and start generating scenes again and again. 
Let's get started.

---------

```rust
// src/game.rs



```





