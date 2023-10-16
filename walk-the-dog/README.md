## Jumping onto a platform

Now that RHB crashes into a stone, we'll need to find a way to go over it. Play the game
and try jumping the rock; you'll that notice it's really difficult. The timing has to be just
right.

First, we're going to put a platform above the stone that RHB can jump on to avoid the rock. 

In addition to putting a platform on screen with a new sprite sheet and giving it a bounding box, 
we'll have to handle a new type of collision.

Specifically, we'll need to handle collisions coming from above the platform so that we can
land on it.

### Adding a platform

We'll start by adding the platform from a new sprite sheet. This sprite sheet actually
contains the elements that will make up our map in the upcoming chapters, but we'll use it
for just one platform for now. The sprite sheet looks like this:

![platforms](./readme_pix/platforms.png)

The image is divided up into squares that aren't outlined but are visible in the way the
shapes are arranged, called `tiles`. Those squares are the sprites that we'll be mixing and
matching to make various obstacles for RHB to jump over and slide under. The tiles are
also jammed together nice and tight, so we won't have to concern ourselves with any
offsets. 
For the time being, we'll only need the platform at the lower-right corner, which
will float over the stone:

![platforms](./readme_pix/platforms2.png)

This one is conveniently set up with the sprites in order, so it will be easy to access in the
sprite sheet. You can see those dotted lines now marking the three sprites. 
Let's get it into our game. In the sprite_sheets directory of the assets, you'll find two files, 
`tiles.json` and `tiles.png`. This is the sheet for the tiles, which we'll need to load at startup.
So that we have something to load it into, we'll start by creating a Platform struct in the
`game` module:

```rust
// src/game.rs

struct Platform {
    sheet: Sheet,
    image: HtmlImageElement,
    position: Point,
}

impl Platform {
    fn new(sheet: Sheet, image: HtmlImageElement, position: Point) -> Self {
        Platform {
            sheet,
            image,
            position,
        }
    }
}

```

So far, this just loads up the expected data. At this point, you may note that sheet
and image are paired together repeatedly, which means they are good candidates for
refactoring into a new structure, such as SpriteSheet . We won't do that now because
we don't want to be premature and refactor to a bad abstraction, but we'll keep an eye out
for the duplication if it shows up again.

The platform is going to need two things. It's going to need to be drawn, and it's going to
need a bounding box so that we can land on it. To draw the box, we'll need to draw the
three tiles that make that platform on the bottom together. Looking at tiles.json , it's
hard to tell which platforms we want because the frame names are all just numbers such as
14.png , so just take my word for it that the tiles are 13.png , 14.png , and 15.png.

Let's dive into the draw function for Platform now, which has a little trick in it, as
seen here:

```rust
// src/game.rs

impl Platform {
...
    fn draw(&self, renderer: &Renderer) {
        let platform = self
                        .sheet
                        .frames
                        .get("13.png")
                        .expect("13.png does not exist");
        
        renderer.draw_image(&self.image,
                                &Rect {
                                    x: platform.frame.x.into(),
                                    y: platform.frame.y.into(),
                                    width: (platform.frame.w * 3).into(),
                                    height: platform.frame.h.into(),
                                },
                                &Rect {
                                    x: self.position.x.into(),
                                    y: self.position.y.into(),
                                    width: (platform.frame.w * 3).into(),
                                    height: platform.frame.h.into(),
                                },
                            );
    }

```

The cheat is that we know that the three tiles happen to be next to each other in the
sheet, so instead of getting all three sprites out of the sheet, we'll just get three times the
width of the first sprite. That will happen to include the other two tiles. Don't forget that
the second Rect is the destination and, as such, should use the position field. 

That second rectangle also corresponds to the bounding box of the platform, 
so let's create the platform's bounding box function and use it there instead. 
These changes are shown here:

```rust
// src/game.rs

impl Platform {
    ...

    fn bounding_box(&self) ->Rect {
        let platform = self
                        .sheet
                        .frames
                        .get("13.png")
                        .expect("13.png does not exist");
        
        Rect {
            x: self.position.x.into(),
            y: self.position.y.into(),
            width: (platform.frame.w * 3).into(),
            height: platform.frame.h.into(),
        }
    }//^-- bounding_box

    fn draw(&self, renderer: &Renderer) {
        ...
```

And use `bounding_box` in `draw` to replace the second `&Rect`


```rust
// src/game.rs

impl Platform {
    ...

    fn draw(&self, renderer: &Renderer) {
        ...
        renderer.draw_image(&self.image,
                        &Rect {
                            x: platform.frame.x.into(),
                            y: platform.frame.y.into(),
                            width: (platform.frame.w * 3).into(),
                            height: platform.frame.h.into(),
                        },  
                        &self.bounding_box(),
        );
    }//^-- draw


```


This code has the same troubles as other code where we search for the frame on every
draw and we're doing it twice. We're also constructing `Rect` on every `bounding_box`
call, which we explicitly avoided earlier. 
Why the change? Because we'll be changing how we construct this shortly, 
so it's not worth worrying about saving an extra cycle or two here.

#### drawing the platform

Now that we've made a platform that could theoretically be drawn, let's actually draw it.
First, we'll add it to the Walk struct, as shown here:

```rust
// src/game.rs

struct Walk {
    boy: RedHatBoy,
    background: Image,
    stone: Image,
    platform: Platform,
}

```

Of course, that won't compile because when we create `Walk` , we don't have a platform.
We need to update the initialize function in `WalkTheDog` to fetch the `tiles.json` 
and create a new `Platform` with it and `tiles.png`. And include the new `Platform` 
as shown here:


```rust
// src/game.rs

impl Game for WalkTheDog {
    async fn initialize(&mut self) -> Result<Box<dyn Game>> {
        match self {
            WalkTheDog::Loading => {
                        ...
                        let stone = engine::load_image("../resources/pix/Stone.png").await?;
                        let platform_sheet = browser::fetch_json("../resources/pix/tiles.json").await?;
                        let platform = Platform::new( platform_sheet.into_serde::<Sheet>()?,
                                                      engine::load_image("../resources/pix/tiles.png").await?,
                                                      Point { x: 200, y: 400 },
                                       );

                let walk = Walk {   boy: rhb, 
                                    background: Image::new(background, Point {x:0, y:0}),
                                    stone: Image::new(stone, Point { x: 150, y: 546 }),
                                    platform: platform,
                                };

```

Finally, we create Walk with platform. 
Drawing the platform is a one-line change, adding it to the draw function of WalkTheDog, 
as shown here:

```rust
// src/game.rs

#[async_trait(?Send)]
impl Game for WalkTheDog {
    ...
    fn draw(&self, renderer: &Renderer) {
        ...
        if let WalkTheDog::Loaded(walk) = self {
            walk.background.draw(renderer);
            walk.boy.draw(renderer);
            walk.boy.draw_rect(renderer);
            walk.stone.draw(renderer);
            walk.stone.draw_rect(renderer);
            
            walk.platform.draw(renderer);
        }
    }
}
```

Lets add a `draw_rect` for platform since we already have `bounding_box`

```rust
// src/game.rs

impl Platform {
    ...
    fn bounding_box(&self) ->Rect {
        ...

    fn draw(&self, renderer: &Renderer) {
        ...
        walk.stone.draw_rect(renderer);            
        walk.platform.draw(renderer);
        walk.platform.draw_rect(renderer);
    }

    fn draw_rect(&self, renderer: &Renderer){
        renderer.draw_rect(&self.bounding_box());
    }
```

yeah!

![An escape](./readme_pix/an_escape.png)


But while the platform has a bounding box, you aren't using it yet, so we'll need to
add that collision to the `update` function of `WalkTheDog`. 

When colliding with the platform, you'll want to **transition** from `Jumping` back to `Running`. 
This transition is already written – we do it when we land on the floor – so you'll just need 
to add a check and an event that can perform the transition.

We'll also need to make sure that RHB stays on the platform. 

Currently, gravity would just pull him right through it, regardless of whether or not 
there's a collision or the player is in the Running state. 

That solution is a little more complex. 

A naive solution, and I know because I wrote it, is to stop applying gravity 
when the player is on the platform. This works until it doesn't, causing a Wile E. Coyote effect 
when RHB runs off the platform and stays in the air. 
Presumably, if he could look down, he would hold up a sign and then crash to the ground.

Instead, what we do is continue to apply gravity on every frame and check whether RHB 
is still landing on the platform. 
If he is, then we adjust him right back onto the top of it. 
This effectively means that RHB "lands" repeatedly until he reaches the end of the platform,
when he falls off. 
Fortunately, this isn't visible to the user, since we calculate RHB's new position on every update, 
and this results in him moving to the right until he falls off the edge, as he should.

Let's start by adding the check to the update function so that RHB can land on a platform:




