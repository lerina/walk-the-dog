# Collision Detection

## Creating a real scene

### Adding the background

Right now, our game can only render images from a sprite sheet, which we can use for
a background, but that's overkill for one image. Instead, we'll add a new struct
that draws a simple image from a .png file. Then, we'll add that to the draw and
initialize functions in WalkTheDog :

TODO:

1. Create an Image struct.
2. Update the Image
3. Load the image
4. Modify WalkTheDog enum to load 
5. Add a Walk struct
6. Initialize Game with the Walk struct
7. Draw background

#### Create an Image struct.

We can work bottom-up for these changes, adding code to the engine and then
integrating it into the game. Our Image struct will use a lot of the same code
that we wrote in Chapter 2, Drawing Sprites, but with a simpler setup because we
won't be using a sheet. 

All of this code should go into the `engine` module.

Start with a struct holding HtmlImageElement :

```rust 
// src/engine

pub struct Image {
    element: HtmlImageElement,
    position: Point,
}

impl Image {
    pub fn new(element: HtmlImageElement, position: Point) -> Self {
        Self { element, position }
    }
}
```

The `Image` struct holds the image element, presumably loaded via the `load_image` function, 
and its position in the scene. `Image` will also need a draw function, but there's no simple
way to draw the entire image as it is in `Renderer`. 
That will need a new method:

```rust 
// src/engine

impl Renderer {
    ...
    pub fn draw_entire_image(&self, image: &HtmlImageElement, position: &Point) {
        self.context
            .draw_image_with_html_image_element(image, 
                                                position.x.into(), 
                                                position.y.into())
            .expect("Drawing is throwing exceptions! Unrecoverable error.");
    }
}
```

This function is very similar to the draw_image function we wrote earlier, 
but it's using the simpler version of the JavaScript drawImage function 
that only takes an image and a position.

To use this method, you'll need to be aware of how large the image you're drawing is. 
If it's too big or too small, it will show up just as big or small as the source image.

#### Update the Image

Now that you've added a method to Renderer , go ahead and update the Image implementation 
to draw an image with it:

NOTE: Its Image not HtmlImageElement as in the book.

```rust 
// src/engine

impl Image{
    ...
    pub fn draw(&self, renderer: &Renderer) {
        renderer.draw_entire_image(&self.element,&self.position)
    }
}
```

Now that you can draw an image, let's load it.

#### Load the image

Load the background image `BG.png`. 
Where? In the `game` module, in the initialize function of WalkTheDog.

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
        ...
```

Our WalkTheDog enum only holds RedHatBoy , so we're going to have to restructure the code a little. 

While we could have the `WalkTheDog::Loaded` state hold a tuple of `RedHatBoy` and `Background`,
that's going to get real annoying, real fast.

#### Modify WalkTheDog enum

To do that, change enum from 

```rust 
// src/game.rs

pub enum WalkTheDog {
    Loading,
    Loaded(RedHatBoy),
}
```
to look like this:

```rust 
// src/game.rs

pub enum WalkTheDog {
    Loading,
    Loaded(Walk),
}
```

We'll have WalkTheDog represent our game, but I decided that RHB takes the dog
for "walks," so our level will be Walk . In a generic framework, I might call this a
"scene" or "level," but this is a specific game, so Walk should work.

#### Add a Walk struct 

The Walk struct will need to have the RHB and the background, so go ahead and
add that:
```rust
// src/game.rs

pub struct Walk {
    boy: RedHatBoy,
    background: Image,
}
```

Make sure you've imported Image from the engine module. 

```rust
// src/game.rs

use crate::engine::Image;
...

pub struct Walk {
    boy: RedHatBoy,
    background: Image,
}

pub enum WalkTheDog {
    Loading,
    Loaded(Walk),
}
...
```

Now, you can work your way down the game module and follow the compiler errors. 
In the initialize function for WalkTheDog, 
you should see an error for " expected struct `Walk`, found struct `RedHatBoy`".

#### Initialize Game with the Walk struct

Creating `Walk` with the `background` we already loaded and setting it in
`WalkTheDog::Loaded` that's returned. This will look as follows:

```rust
// src/game.rs

impl Game for WalkTheDog {
    async fn initialize(&mut self) -> Result<Box<dyn Game>> {
        ...
        Ok(Box::new( WalkTheDog::Loaded( 
                            Walk { boy: rhb,
                                   background: Image::new( background, Point {x: 0, y: 0 }),
                            })
        ))
    }
...
}
```

This will create Walk with a boy and background positioned at the upper-left corner, 
but you should still have several compiler errors in the update method of `WalkTheDog` 
because those all assume that `WalkTheDog::Loaded` contains `RedHatBoy`. 

Each of those can be changed in the exact same way. The first looks like this:

```rust
// src/game.rs

impl Game for WalkTheDog {
...
fn update(&mut self, keystate: &KeyState) {
    //if let WalkTheDog::Loaded(rhb) = self {
    if let WalkTheDog::Loaded(walk) = self {
        if keystate.is_pressed("ArrowRight") {
            //walk.run_right();
            walk.boy.run_right();
        }
...
```


The if let WalkTheDog::Loaded line is unchanged, except now the variable
name is walk instead of rhb . Then, we call run_right on boy but via the
walk structure. You could argue that we should add methods to Walk instead
of delegating to boy , but we'll hold off on that for now. After all, walk.run_
right() doesn't really make sense. After fixing all the similar compiler errors in
update , you can also fix a similar error in draw , like so:

```rust
// src/game.rs

impl Game for WalkTheDog {
    ...
    fn draw(&self, renderer: &Renderer) {
        renderer.clear(&Rect {
            x: 0.0,
            y: 0.0,
            width: 600.0,
            height: 600.0,
        });

        if let WalkTheDog::Loaded(walk) = self {
            walk.boy.draw(renderer);
        }
    }
    ...
```

#### Draw background

Next, go ahead and draw the background for our game. Drawing the background
is a matter of using our new draw function, so let's add that right before the walk.
boy.draw function call, as shown here:

```rust
// src/game.rs

impl Game for WalkTheDog {
    ...
    fn draw(&self, renderer: &Renderer) {
    ...
        if let WalkTheDog::Loaded(walk) = self {
            walk.background.draw(renderer);
            walk.boy.draw(renderer);
        }
...
```

After doing that, you should see RHB standing in front of the background.

## Adding an obstacle

Our new Image object means we won't need much code to add an obstacle

Put Stone.png in your pix resources folder, then can add it to
`Walk` in the same way you added `Background`, like so:

```rust
// src/game.rs

struct Walk {
    boy: RedHatBoy,
    background: Image,
    stone: Image,
}
```

That will start causing compiler errors again because Walk is created without
a stone. 

In initialize , go ahead and load the stone, just as you loaded the background,
as shown here:

```rust
// src/game.rs

impl Game for WalkTheDog {
    async fn initialize(&mut self) -> Result<Box<dyn Game>> {
        ...
        let background = engine::load_image("../resources/pix/BG.png").await?;
        let stone = engine::load_image("../resources/pix/Stone.png").await?;
        ...
```

Then, you need to take the stone that we just loaded and add it to Walk . We'll make
sure the stone is on the ground by taking the FLOOR value ( 600 ) and subtracting
the height of the stone image, which happens to be 54 pixels. If we position the
stone at a y position of 546 , it should be sitting right on the ground. Here's the
update for creating Walk :       


```rust
// src/game.rs

impl Game for WalkTheDog { 
    async fn initialize(&mut self) -> Result<Box<dyn Game>> {
        ...
           let walk = Walk {   boy: rhb, 
                               background: Image::new(background, Point {x:0, y:0}),
                               stone: Image::new(stone, Point { x: 150, y: 546 })
                            };
            Ok(Box::new(WalkTheDog::Loaded(walk)))
        

```

The stone is 150 pixels to the right, so it will be in front of RHB. 
Finally, draw the stone using the `draw` method. 
That addition is as follows:


```rust
// src/game.rs

impl Game for WalkTheDog {
    ...
    fn draw(&self, renderer: &Renderer) {
    if let WalkTheDog::Loaded(walk) = self {
        ...
        if let WalkTheDog::Loaded(walk) = self {
            walk.background.draw(renderer);
            walk.boy.draw(renderer);
            walk.stone.draw(renderer);
        }

