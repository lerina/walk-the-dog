## Refactoring for endless running

### Obstacle traits

Currently, the stone and the platform are separate objects on the Walk struct. 
If we want to add more obstacles to the game, we must add more fields to this struct. 
This is an issue if we want to have an endlessly generated list of things to jump over 
and slide under. 

What we'd like to do instead is keep a list of Obstacles, 
go through each one, and check what to do when RedHatBoy intersects them. 

Why do we want to do that? 

Let's have a look:

• It will eliminate the duplication for knocking out RHB, and eliminate future
duplication that we'd have to create to continue with our current pattern.
• We want to treat each Obstacle as the same so that we can create obstacles on
the fly.
• We'll be able to remove any obstacles that have gone off screen.

We'll start by creating an `Obstacle` trait in the `game` module, 
with one new method named `check_intersection` and two that exist already on `Platform`:


```rust
// src/game.rs

pub trait Obstacle {
    fn check_intersection(&self, boy: &mut RedHatBoy);
    fn draw(&self, renderer: &Renderer);
    fn move_horizontally(&mut self, x: i16);
}

```

Why these three methods? `stone` and `platform` are both going to implement
`Obstacle`, and we'll need to loop through them, draw them, and move them. 
So, that's why the trait contains `move_horizontally` and `draw`. 

The new method, `check_intersection`, exists because a platform lets you land on it, 
whereas a stone doesn't. 

So, we'll need an abstraction that can handle intersections differently depending
on the type of Obstacle. 

Now that we've created our trait , we can implement it on the `Platform` structure. 

We can start by pulling `draw` out of the Platform implementation 
and creating a `move_horizontally` method, as shown here:

```rust
// src/game.rs

impl Obstacle for Platform {
    // previously in impl Platform
    fn draw(&self, renderer: &Renderer) {
        let platform = self
                        .sheet
                        .frames
                        .get("13.png")
                        .expect("13.png does not exist");
        
        renderer.draw_image( &self.image,
                             &Rect {
                                 //x: platform.frame.x.into(),
                                 //y: platform.frame.y.into(),
                                 position: Point {
                                     x: platform.frame.x.into(),
                                     y: platform.frame.y.into(),
                                 },
                                 width: (platform.frame.w * 3).into(),
                                 height: platform.frame.h.into(),
                             },
                             &self.destination_box(), //&self.bounding_box(),
                           );
    }//^-- draw
    
    fn move_horizontally(&mut self, x: i16) {
        self.position.x += x;
    }
}
```

`move_horizontally` mimics the code that is currently in `update`, 
which we identified as a code smell earlier.
Finally, let's add the `check_intersection` function, 
which currently exists in the `update` method of WalkTheDog :


```rust
// src/game.rs

#[async_trait(?Send)]
impl Game for WalkTheDog {
    fn update(&mut self, keystate: &KeyState) {
        ...
            // check_intersection
            for bounding_box in &walk.platform.bounding_boxes() {
                if walk.boy.bounding_box().intersects(bounding_box) {
                    if walk.boy.velocity_y() > 0 && walk.boy.pos_y() < walk.platform.position.y {
                        walk.boy.land_on(bounding_box.y);
                    } else {
                        walk.boy.knock_out();
                    }
                }
            }

```

The version that's been implemented for `Platform` should be very similar, 
without the references to walk , as shown here:


```rust
// src/game.rs

impl Obstacle for Platform {
    ...
    fn check_intersection(&self, boy: &mut RedHatBoy) {
        if let Some(box_to_land_on) = self
                .bounding_boxes()
                .iter()
                .find(|&bounding_box| boy.bounding_box()
                .intersects(bounding_box))
        {
            if boy.velocity_y() > 0 && boy.pos_y() < self.position.y {
                    boy.land_on(box_to_land_on.y());
            } else {
                boy.knock_out();
            }
        }
    }
}
```

This code is largely the same but with one fairly significant optimization: 
instead of looping through every bounding box in Platform, 
this code uses find to get the first bounding box that's intersected. 
If there is one ( if let Some(box_to_land_on) ),
then we handle the collision. This prevents redundant checks after a collision is found. 
The rest of the code is a little bit shorter without the references to walk, which is nice. 

Now, we need to replace `Platform` in `Walk` with a reference to it on the heap, like so:


```rust
// src/game.rs

struct Walk {
boy: RedHatBoy,
backgrounds: [Image; 2],
stone: Image,
platform: Box<dyn Obstacle>, //platform: Platform,
}

```

Note::
    
    We do have an alternative to using a trait object here, which would be using an
    enum containing every type of obstacle, just like we did with our state machine.
    The tradeoff to using dynamic dispatch, via the dyn keyword, is that a lookup
    table is stored in memory. The benefit of this is that we write less boilerplate
    code, and the code doesn't need to be updated every time we add an obstacle.
    In this case, I think trait works better in the same way that an enum works
    better for a state machine, but it's worth keeping that in mind.
    

This will cause two compiler errors that we can fix by making small changes. 
In the `initialize` method of `WalkTheDog`, we are not setting platform correctly when we
create `Walk`, so let's make a small change, as shown here:


```rust
// src/game.rs

#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        match self {
            WalkTheDog::Loading => {
                ...
                let walk = Walk {   boy: rhb, 
                                    backgrounds: backgrounds,
                                    stone: Image::new(stone, Point { x: 150, y: 546 }),
                                    platform: Box::new(platform), //platform: platform,
                                };

                Ok(Box::new(WalkTheDog::Loaded(walk)))
            },
            WalkTheDog::Loaded(_) => Err(anyhow!("Error: Game is already initialized")),
        }
    }//^-- async fn initialize
```

This is only a one-line change that involves replacing platform with 
`platform: Box::new(platform)`. 

The other fix is something you'll remember being a smell – setting the position 
on `x` directly when `stone` uses a method called `move_horizontally`. 
This is why we created that method on the `Obstacle` trait on the Platform struct. 

This change can be found in the `update` function for `WalkTheDog`,
as shown here:

```rust
// src/game.rs

impl Game for WalkTheDog {
    ...
    fn update(&mut self, keystate: &KeyState) {
        if let WalkTheDog::Loaded(walk) = self {
            ...
            let velocity = walk.velocity();
            walk.platform.move_horizontally(velocity);
            walk.stone.move_horizontally(velocity);

```

Having both platform and stone have a move_horizontally function is a sign
that those interfaces can be brought together, which we'll do in a moment. Finally, we
must replace the code that we moved into check_intersection with a call to that
function. Just a little further down the update function, you'll want to update the
following code:

```rust
// src/game.rs

impl Game for WalkTheDog {
    ...
    fn update(&mut self, keystate: &KeyState) {
        if let WalkTheDog::Loaded(walk) = self {
        ...
            if second_background.right() < 0 {
                second_background.set_x(
                first_background.right());
            }

/*
            // check_intersection
            for bounding_box in &walk.platform.bounding_boxes() {
                if walk.boy.bounding_box().intersects(bounding_box) {
                    if walk.boy.velocity_y() > 0 && walk.boy.pos_y() < walk.platform.position.y {
                        walk.boy.land_on(bounding_box.y);
                    } else {
                        walk.boy.knock_out();
                    }
                }
            }
*/
            //// check_intersection comment no longer needed
            walk.platform.check_intersection(&mut walk.boy);   

            // knock_out
            if walk.boy
                   .destination_box()
                   .intersects(walk.stone.bounding_box())
            {
                walk.boy.knock_out();
            }
 
```

The call to `check_intersection` goes before the check to see whether you've crashed
into a stone and after the background updates. 

Humm so `update should look like this:


```rust
// src/game.rs

    fn update(&mut self, keystate: &KeyState) {
        if let WalkTheDog::Loaded(walk) = self {
            if keystate.is_pressed("ArrowRight") {
                walk.boy.run_right();
            }

            if keystate.is_pressed("Space") {
                walk.boy.jump();
            }

            if keystate.is_pressed("ArrowDown") {
                walk.boy.slide();
            }

            walk.boy.update();

            let velocity = walk.velocity();

            walk.platform.position.x += walk.velocity();
            walk.platform.move_horizontally(velocity); //walk.velocity());
            walk.stone.move_horizontally(velocity);
            

            // land; check_intersection comment no longer needed
            walk.platform.check_intersection(&mut walk.boy);   

            // knock_out
            if walk.boy
                   .destination_box()
                   .intersects(walk.stone.bounding_box())
            {
                walk.boy.knock_out();
            }

            // background states
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

        }//^-- if let
    }//^-- fn update
```

You may notice that the code for checking for collisions with a stone is different, 
in the sense that boy is always knocked out when you collide with it, 
but is it also conceptually the same because you are, once again,
checking for a collision with an obstacle and then doing something. 

This is why we need to turn stone, which is currently an Image type, 
into an Obstacle type.  But what type should it be?

### Barriers versus platforms

We need another type of `Obstacle` that cannot be landed on, and right now a `stone` is
an `Image`. 
Adding features to `Image` isn't appropriate because an `Obstacle` trait is a game concept 
and `Image` is part of `engine`. 

Instead, we'll create a type of `Obstacle` that always causes the user to crash into it, 
called `Barrier`, and turn stone into that. It's a very dangerous stone.

We'll start by creating a `Barrier` struct and implementing the `Obstacle` trait with placeholders, 
as shown here:



```rust
// src/game.rs

pub struct Barrier {
    image: Image,
}

impl Obstacle for Barrier {
    fn check_intersection(&self, boy: &mut RedHatBoy) {
        todo!()
    }
    fn draw(&self, renderer: &Renderer) {
        todo!()
    }
    fn move_horizontally(&mut self, x: i16) {
        todo!()
    }
}//^-- impl Obstacle for Barrier

```

Tip::
    
    I generated this skeleton with rust-analyzer while using 
    the add-missing-members action. In my editor (emacs), 
    this is as simple as typing c v. 
    In Visual Studio Code, simply click the lightbulb and choose Implement
    missing members. The todo! macro throws a runtime exception if this code
    is called without any implementation, and it is meant to signal temporary code
    that is there to please the compiler.
    


Note::
    
    Right now, all `Barrier` objects have to be an `Image`, whereas a `Platform`
    uses a `sprite sheet`. You may want to use sprite sheets for everything, or even
    one sprite sheet for everything, and that's fine – better, even. We'll leave things
    as is here because we've redesigned this application enough already.
    

Before we fill in all those `todo!` blocks, let's add a typical new method to create the
`Barrier` object:

```rust
// src/game.rs

impl Barrier {
    pub fn new(image: Image) -> Self {
        Barrier { image }
    }
}
```

Now, we can fill in the functions. The `draw` and `move_horizontally` functions can
delegate to `Image`, as shown here:


```rust
// src/game.rs

impl Obstacle for Barrier {
    ...
    fn draw(&self, renderer: &Renderer) {
        self.image.draw(renderer);
    }

    fn move_horizontally(&mut self, x: i16) {
        self.image.move_horizontally(x);
    }
```

The final function, `check_intersection`, will be a little different. 
Unlike a `Platform`, which `boy` can land on, a `Barrier` is always crashed into. 
The code for this already exists in the `update` method of `WalkTheDog` because 
it's what we used for `stone`. 

Let's mimic that implementation here:

```rust
// src/game.rs

impl Obstacle for Barrier {
    ...
    fn check_intersection(&self, boy: &mut RedHatBoy) {
        if boy.bounding_box().intersects(self.image.bounding_box()) {
            boy.knock_out()
        }
    }
}
```

`Barrier` isn't being used anywhere yet. So, we could start by changing `stone` from an
`Image` into a `Barrier`. 

However, we're going to go a little further than that. 

We're going to create a list in `Walk` that contains all the `Obstacle` types. 
This will let us reduce the amount of specific code in Walk, 
and it will make it far simpler to generate new obstacles on the fly. 

Remember that's what we're refactoring for. 
Let's make our list and add it to the Walk struct, as shown here:


```rust
// src/game.rs


/*
pub struct Walk {
    boy: RedHatBoy,
    backgrounds: [Image; 2],
    stone: Image,
    platform: Box<dyn Obstacle>, //platform: Platform,
}
*/

pub struct Walk {
    boy: RedHatBoy,
    backgrounds: [Image; 2],
    obstacles: Vec<Box<dyn Obstacle>>,
}

```

Note that we've removed `platform` and `stone` from `Walk`, 
we'll need to update the rest of its implementation and replace direct references 
to `stone` and `platform` with references to the `Obstacle` vector. 

This doesn't mean we won't ever mention `platform` and `stone` again; 
we still have to load the image and sprite sheet, but we'll only mention it once. 

Once again, we'll look at the compiler error messages, which are complaining a
lot about the `initialize`, `update`, and `draw` methods in `WalkTheDog`. 

Let's start by making changes to the initialize function, as shown here:


```rust
// src/game.rs

impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        ...
        
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

We're only changing the construction of the `Walk` construct, replacing the references to
`stone` and `platform` by initializing the `obstacles` vector. 
The first item in the vector is now a `Barrier` but that's just the `stone` object 
that we created earlier wrapped in the new `Barrier` struct. 
The second is the `platform` object that we created previously.

Everything has to be in a Box so that we can use the `Obstacle` trait. 

The next few changes we'll make must be done in the `update` method. 
We'll rearrange the code a little bit to `update` `boy` first, 
then our `backgrounds`, and finally our `obstacles`, 

as shown here:


```rust
// src/game.rs

impl Game for WalkTheDog {
...
    fn update(&mut self, keystate: &KeyState) {
        if let WalkTheDog::Loaded(walk) = self {    
            ...
           if second_background.right() < 0 {
                second_background.set_x(
                first_background.right());
            }

            walk.obstacles.iter_mut().for_each(|obstacle| {
                obstacle.move_horizontally(velocity);
                obstacle.check_intersection(&mut walk.boy);
            });
```

There should be no direct references to `stone` or `platform` in `update`. 

Now, the code for checking for the movement of obstacles 
and whether they intersect should only be four lines long 
and be at the bottom of the update method – and that's generously counting the closing brace. 

Make sure you use the `iter_mut` method since we are mutating obstacle in the loop. 

One of the ways we can tell that we are moving in the right direction in our design 
is that we're writing less code that works with more things. 

Finally, we will need to draw all our obstacles, 
which can be handled by updating the draw method, as shown here:


```rust
// src/game.rs

impl Game for WalkTheDog {
    ...
    fn draw(&self, renderer: &Renderer) {
        ...
        if let WalkTheDog::Loaded(walk) = self {
            ...
            walk.obstacles.iter().for_each(|obstacle| {
                obstacle.draw(renderer);
            });
        }
    }
}
```

In this case, we can use `for_each` and a plain `iter()`. 

As you may have guessed, when we want to add more obstacles to the screen, 
we will just add them to the obstacles list.

At this point, the code should be working again; 
RHB should hop his way over a platform and a stone and then crash into it. 
Now, all we need to take care of is the crash that occurs 
if we let RHB keep running. 

We'll handle that next.

------------


```rust
// src/game.rs



```

