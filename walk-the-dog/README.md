## Collision from below

At the moment, if RHB collides with the platform, he is set on the top, 
which is great for landing but not so great if he comes from beneath the platform. 

If you were to comment out the collision with the stone right now and run straight ahead, 
you'd actually find yourself suddenly pop up onto the platform! 

Why? 
Because RHB's head actually bumps into the bottom of the platform, and that collision 
causes the land_on event to fire. 

Instead of banging his head and falling over, he teleports onto the platform!

We need to have special collision detection here. RHB can only land on the platform 
if he comes from above it; otherwise, it's game over. 

Fortunately, this can be handled in the update function with two small changes 
to the way we check collisions. 

Collisions with the platform where RedHatBoy is above the platform means landing; 
otherwise, it's the same as hitting a stone, and you get knocked out. 
You also need to be descending; otherwise, you'll get this weird effect 
where you stick to the platform while still going up in your jump. 

Let's see that change:

```rust
// src/game.rs

#[async_trait(?Send)]
impl Game for WalkTheDog {
    ...
       fn update(&mut self, keystate: &KeyState) {
             walk.boy.update();

            // land
            if walk.boy
                   .bounding_box()
                   .intersects(&walk.platform.bounding_box())
            {
                // moving down              and above the platform
                if walk.boy.velocity_y() > 0 && walk.boy.pos_y() < walk.platform.position.y {
                    walk.boy.land_on(walk.platform.bounding_box().y);
                } else {
                    walk.boy.knock_out();
                }
            }

    

```

The changes are to check whether the RedHatBoy velocity, in `y`, is greater than 0 and,
therefore, RHB is moving down. We also check whether the position in y is less than the
top of the platform's y position. This means that the boy is above the platform, so he's
landing on it; otherwise, the boy has crashed into it, and we knock him out. 

The `pos_y` and `velocity_y` functions don't exist yet, so we'll add those to RedHatBoy, 
as shown here:

```rust
// src/game.rs


impl RedHatBoy {
    ...
    fn pos_y(&self) -> i16 {
        self.state_machine.context().position.y
    }

    pub fn velocity_y(&self) -> i16 {
        self.state_machine.context().velocity.y
    }
    ...
}

```

Also make velocity public

```rust
// src/game.rs

    #[derive(Copy, Clone)]
    pub struct RedHatBoyContext {
        pub frame: u8,
        pub position: Point,
        pub velocity: Point,
    }
```

It's a little tricky to get the `y` values for `RedHatBoy` because they are actually 
on `RedHatBoyContext`, but we are able to pull it off here and wrap them in a getter for
convenience.

With that, RHB can finally run, jump over stones, land on platforms, and fall off them.
However, you've probably noticed that the collisions are really crude. He crashes into
the bottom of the platform easily because the transparent parts of the images collide.
He also can walk past the edge of the platform, again because of the transparent parts of
the image:

![Walk on air](./readme_pix/walk_on_air.png)

Let's spend a little time tweaking our bounding boxes to deal with the transparency.

### Transparency in bounding boxes

The problem with our bounding boxes is that we're using the image dimensions as the
bounding box. That means we'll have a lot of extra space around our characters for our
bounding boxes. In this screenshot, I've used the draw_rect method from earlier in this
chapter to show the bounding boxes for all three objects in our scene:

![Bounding boxes everywhere](./readme_pix/bounding_boxes_everywhere.png)

The platform has a lot of white space in the bounding box, particularly at the lower-left
and lower-right corners. RHB also has white space near the corners of his hat. 

When we turn off the collision checks on the stone and try to walk under the platform, 
RHB "collides" with the lower-left corner of the platform well before he actually hits it.

The white space around RHB's feet is a problem too; they are what cause the landing
on air effect. 
The far-right edge of his bounding box intersects with the platform, so he lands 
before he's really in the right position. 

If you could see him walk off the edge of the platform, you'd see that it has 
the same problem when he walks off. He takes several steps in mid-air before he begins to fall. 

We'll start by dealing with RHB's bounding box to make landing and falling off the platform 
look a little more realistic.

#### Fixing the game to fit

There are algorithms we can use to make the bounding box better match the actual pixels
in the image, but ultimately, none of them are necessary. 

Spend a little time with most platformers and you'll see that the collisions aren't perfect, 
and 99% of the time, it's just fine. 

After spending a little time "researching" by playing video games, I determined that
if we simply make the box only as wide as the feet, he develops a much more realistic
landing. 

This is a little counter-intuitive. If we narrow the box, his arm and hat will stick
out past the edge of the box; we'll miss collisions! Does this matter?

![Narrow bounding box](./readme_pix/narrow_bounding.png)


The answer is, "maybe." Bounding boxes and collision detection are not just mathematical
problems. They are also game design problems. Making the bounding box wrap just
around the feet felt right to me when playing the game. Maybe when you play it, that will
feel too hard when you land on a platform or the hand not colliding will bother you, so
change the box! It's not written in stone.

After experimenting, I found that I wanted to shorten the box as well so that RHB couldn't
be knocked out by grazing his hat. 

To mimic that, we can start by renaming `bounding_box` to `destination_box`, 
because that represents where the sprite is rendered to.

```rust
// src/game.rs

impl RedHatBoy {
    ...

    //fn bounding_box(&self) ->Rect {
    fn destination_box(&self) -> Rect {
        let platform = self
                        .sheet
                        .frames
                        .get("13.png")
                        .expect("13.png does not exist");
    ....
```

 
It needs to be at the position of RedHatBoy in the game but with the width and height of
the source image; otherwise, the image will appear squished. 

Then, we can re-implement the RedHatBoy bounding box, like so:

```rust
// src/

impl RedHatBoy {
    ...

    fn destination_box(&self) -> Rect {
        ...
    }

    fn bounding_box(&self) -> Rect {
        const X_OFFSET: f32 = 18.0;
        const Y_OFFSET: f32 = 14.0;
        const WIDTH_OFFSET: f32 = 28.0;
        let mut bounding_box = self.destination_box();
        bounding_box.x += X_OFFSET;
        bounding_box.width -= WIDTH_OFFSET;
        bounding_box.y += Y_OFFSET;
        bounding_box.height -= Y_OFFSET;
        bounding_box
    }
```

We start with the original dimensions of the image, `destination_box`, and simply
shrink it by some offsets. I chose the numbers by using the high-tech system of picking
numbers and looking at them. This gave me a bounding box that looked natural jumping
and falling off the cliff while not being so small that RHB never hits anything.

If you did a global find and replace on `bounding_box` and changed it to
`destination_box` , then the collision detection is incorrect. We need to use
`bounding_box` for checking collisions and `destination_box` for drawing.

Drawing should already be complete; you'll need to go into the update method in
WalkTheDog and make sure that every intersects call is on the bounding_box,
not destination_box .

---------------------

```rust
// src/


```


