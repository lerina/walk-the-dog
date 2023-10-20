## Creating an Endless Runner

Red Hat Boy (RHB) can run, jump on a platform, and even crash into a rock and fall
over. But once he starts running to his right, he just goes off the screen and is never seen
again.

We'll make our game truly endless by generating
new scenes as RHB runs that contain new obstacles and challenges. They will even contain
randomness, and it all starts with RHB staying in one place!

We will cover the following topics:

• Scrolling the background
• Refactoring for endless running
• Creating a dynamic level

By the end of this chapter, you'll have a functioning endless runner and be able to create
obstacles for RHB to hop over and slide under.


### Scrolling the background

Repeating background technique. We'll use the same background element we're using now and move it to the left as RHB
runs to the right. Immediately after, it will become a copy of the same background,
making two Image elements with the same source image. Once the first image moves
completely off screen, we'll move it so that it's to the right of the second image. These two
backgrounds will loop, creating the illusion of the background moving forever:

![Sliding the canvas over the background](./readme_pix/scrolling_bg.png)

This technique relies on three things. 

The first is that the background has to be seamless so that there is no visible seam between the two images. Fortunately, our background was built for this and it will work fine. 

The second is that the canvas window needs to be smaller than the background so that the entire background 
is never shown on screen.
If we do this, then the first background can go entirely off screen to the left and then be
moved to the right of the second background, all without any noticeable gaps or tearing.
This is because this all happens outside the window's boundaries. 
I like to think of it as being offstage in a play, then scrambling over to the right-hand side 
behind the curtain.

Finally, we must use another illusion and freeze the main character in place. 
Instead of moving the character from left to right on the screen, the objects will move right 
to left, almost as if on a treadmill.
 
Visually, this will look the same as if the character were running, 
and it has the advantage of fixing a bug where if the player keeps running right,
their x position eventually overflows (becomes bigger than the i16 we are using to hold it)
and the game crashes. 

We'll have to adjust our brains by changing the x velocity from what we expect, 
but once you get used to it, you'll find that it works quite easily. 
Let's get started with our scrolling background.


### Fixing RHB in x

We can scroll the background as much as we want, but if we continue to simultaneously
move `RHB` to the right at the same time, the effect will be having him run at double speed.
Instead, we want `RHB` to run in place while the rocks and platforms move toward him as
if they were on a conveyor belt. At the end of this section, we will see `RHB` run to the right
into an empty white void as everything passes past him as if he were running past the end
of the world.

Let's start in the `game::red_hat_boy_states` module 
and not update `x` in the update method of RedHatBoyContext :


```rust
// src/game.rs
impl RedHatBoyContext {
    ...
    fn update(mut self, frame_count: u8) -> Self {
        ...
        // rhb stays in-place.  
        // self.position.x += self.velocity.x
        self.position.y += self.velocity.y;

```

With this change, `RHB` will run in place, with nothing moving around him. We are
keeping velocity as is because that value is going to be used by the rest of the code base. 

For ease of use, we'll add a few methods. 
First, let's add an accessor to the `RedHatBoy` implementation, as shown here:


```rust
// src/game.rs

impl RedHatBoy {
    ...
    fn walking_speed(&self) -> i16 {
        self.state_machine.context().velocity.x
    }
```

This function works similar to several of our other accessors for RedHatBoy, 
making it easier to get at the context values. 

Next, let's add a new implementation – Walk for the Walk struct:


```rust
// src/game.rs

pub struct Walk {
    ...
}

impl Walk {
    fn velocity(&self) -> i16 {
        -self.boy.walking_speed()
    }
}

```

The `Walk` implementation is only available when the `WalkTheDog` enum is in the
`Loaded` state and it flips walking_speed of `boy`. While `boy` is moving to the
right, this means everything else is moving to the left. 
Now, in the update function of WalkTheDog, we can use that value 
to move everything else to the left. Right after updating walk.boy, 
we can update the stone and platform positions so that they match the following code:


```rust
// src/game.rs

impl Game for WalkTheDog {
    ...
    fn update(&mut self, keystate: &KeyState) {
        if let WalkTheDog::Loaded(walk) = self {
            ...

            walk.boy.update();
    
            walk.platform.position.x += walk.velocity();
            walk.stone.move_horizontally(walk.velocity());

            for bounding_box in &walk.platform.bounding_boxes() {
                ...
```

We get a compiler error because stone doesn't have a `move_horizontally` function. 
`Stone` is of the `Image` type and can be found in the `engine` module, 
while position on `Image` is private. 

We'll keep things that way, and instead add `move_horizontally` to the `Image` implementation, 
as shown here:

```rust
// src/engine.rs

impl Image {
    ...
    pub fn move_horizontally(&mut self, distance: i16) {
        self.bounding_box.x += distance as f32;
        self.position.x += distance;
    }
}
```

Two things may bother you about this code. 

The first is that we are directly manipulating position on Platform but used a method on Image. 
This inconsistency is a smell that tells us that something isn't right with our code – in this case, 
stone and platform have two different interfaces to modify their positions, 
even though the code has been duplicated. 
For now, we'll leave this as is, but it's a hint regarding changes we may want to make later. 

The other is that we're updating the `bounding_box` and `position` values with the same thing. 
That's a refactoring we'll leave for the next section (putting a position on Rect Point).


Now, you should see RHB running in place as the rock and platform move beneath him.


We can start moving the background by matching the `stone` and `platform` movement
in the update function of `WalkTheDogupdate`. 

This change will look as follows:


```rust
// src/game.rs

    fn update(&mut self, keystate: &KeyState) {
        if let WalkTheDog::Loaded(walk) = self {
            ...
            walk.platform.position.x += walk.velocity();
            walk.stone.move_horizontally(walk.velocity());
            walk.background.move_horizontally(walk.velocity());


            for bounding_box in &walk.platform.bounding_boxes() {
               
            ...
```

This small change will mean that RHB can now walk off the edge of the world:

However, we don't want this, so let's learn how to use two tiling backgrounds to simulate
an infinite one.

### An infinite background

To get an infinite background, we'll need two background images instead of one.
We'll start by storing background as an array instead of just one Image in Walk , as
shown here:

```rust
// src/game.rs

struct Walk {
    boy: RedHatBoy,
    //background: Image,
    backgrounds: [Image; 2],
    stone: Image,
    platform: Platform,
}
```

This will cause several compiler errors because `backgrounds` doesn't exist; 
even if it did, the code expects it to be an Image array. 
Fortunately, the errors largely make sense and we can figure out what needs to be done. 
Moving once again to initialize in the Game implementation, 
let's set up an array of backgrounds instead of just one when initializing `Walk`, 

as shown here:


```rust
// src/game.rs

async fn initialize(&self) -> Result<Box<dyn Game>> {
        match self {
            WalkTheDog::Loading => {
                ...
                
                let background_width = background.width() as i16;
                let backgrounds = [ Image::new( background.clone(), Point { x: 0, y: 0 }),
                                    Image::new( background, Point { x: background_width, y: 0,},),
                                  ];

                let walk = Walk {   boy: rhb, 
                                    backgrounds: backgrounds,
                                    stone: Image::new(stone, Point { x: 150, y: 546 }),
                                    platform: platform,
                                };
                Ok(Box::new(WalkTheDog::Loaded(walk)))
                    ...
```

There's a little more going on here compared to our previous changes, so let's go through
this code in more detail. 

The first thing we do is get the `width` property of `background`.
This is the temporary variable that we created when we loaded `HtmlImageElement`,
not the background property that's attached to `Walk` that we have been using. 
We have done this to prevent a `borrow-after-move` error during the initialization of Walk. 

Then. we made `Walk` take an array of `Image` objects, making sure to clone the background
property the first time we create it. 

Finally, we made sure to position the second `Image` at `background_width` so that 
it will be lined up to the right of the first background, off screen.

However, we still aren't done with compiler errors. This is because the background is
being updated and drawn. We'll make the simplest changes we can so that we can start
compiling and running again. 
First, replace the `move_horizontally` code we just wrote in the `update` function with the following code, 
which `loops` through all the backgrounds and moves them:



```rust
// src/game.rs

fn update(&mut self, keystate: &KeyState) {
    if let WalkTheDog::Loaded(walk) = self {
        ...
        walk.platform.position.x += walk.velocity();
        walk.stone.move_horizontally(walk.velocity());
        
        // walk.background.move_horizontally(walk.velocity());
        let velocity = walk.velocity();
        walk.backgrounds.iter_mut().for_each(|background| {
            background.move_horizontally(velocity);
        });

```

Make sure you use `iter_mut` so that background is mutable. 
Note that you'll need to bind `walk.velocity()` to a temporary variable; 
otherwise, you'll get a compiler error saying cannot borrow '*walk' as immutable because it is also
borrowed as mutable . 

Now, you can update the draw function to draw all the backgrounds:


```rust
// src/game.rs


#[async_trait(?Send)]
impl Game for WalkTheDog {
    ...

    fn draw(&self, renderer: &Renderer) {
        if let WalkTheDog::Loaded(walk) = self {
            //walk.background.draw(renderer);
            walk.backgrounds.iter().for_each(|background| {
                background.draw(renderer);
                });
            ...
            walk.platform.draw_rect(renderer);
        }//^-- if let
    }
```

Here, we are looping through backgrounds again and drawing them, relying on the
canvas to only show the backgrounds that are on screen. If you play the game while
running this code, you'll see that `RHB` runs farther but doesn't run infinitely. 

This is because we aren't cycling the backgrounds. 

If you run the game for long enough, you'll see that the game also crashes with a buffer overflow error, 
but we'll fix that in the next section. 

First, we need to get the backgrounds cycling. 
We can do that by replacing the loop in the update function with code 
that explicitly destructures the array, as shown here:


```rust
// src/game.rs

fn update(&mut self, keystate: &KeyState) {
    if let WalkTheDog::Loaded(walk) = self {
        ...
        walk.platform.position.x += walk.velocity();
        walk.stone.move_horizontally(walk.velocity());

        let velocity = walk.velocity();

        // walk.backgrounds.iter_mut().for_each(|background| {
        //        background.move_horizontally(velocity);
        // });

        let [first_background, second_background] = &mut walk.backgrounds;
        first_background.move_horizontally(velocity);
        second_background.move_horizontally(velocity);

        if first_background.right() < 0 {
            first_background.set_x(
            second_background.right());
        }
        if second_background.right() < 0 {
            second_background.set_x(
            first_background.right());
        }


```

Here, we start by replacing the for loop with 
`let [first_background, second_background] = &mut walk.backgrounds;` 
to get access to both backgrounds.
Then, we move them both to the left, the same as we did in the loop, 
and we check whether the right-hand side of the image is negative. 
This means that the image is off screen, so we can go ahead 
and move it to the right-hand side of the other background.

If you type this in, it won't compile because `set_x` and `right` don't exist 
on the Image struct. 

Open the `engine` module again so that we can add those to `Image` , as follows:


```rust
// src/engine.rs

impl Image {
    ...
    /*
    pub fn move_horizontally(&mut self, distance: i16) {
        self.bounding_box.x += distance as f32;
        self.position.x += distance;
    }

    */

    pub fn move_horizontally(&mut self, distance: i16) {
        self.set_x(self.position.x + distance);
    }
    pub fn set_x(&mut self, x: i16) {
        self.bounding_box.x = x as f32;
        self.position.x = x;
    }
    pub fn right(&self) -> i16 {
        (self.bounding_box.x + self.bounding_box.width) as i16
    }

}
```

Here, we added a `set_x` function that updates `position` and `bounding_box`, 
just like we did previously, and we had `move_horizontally` call it to avoid duplication.

We also added a `right` function that calculates the right-hand side of `bounding_box`
based on the current position. With that, `RHB` now runs to the right, forever! 

Well, until the buffer overflows and it crashes. 
Fortunately, we'll take care of that in the next section.


----------------------


```rust
// src/game.rs


```





