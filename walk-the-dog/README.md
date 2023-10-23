## Refactoring for endless running

### Removing obstacles as they go off screen

If you let RHB run to the right for long enough, you'll see a crash message that looks like
this:

```
panicked at 'attempt to add with overflow', src/engine.
rs:289:20
Stack:
```

The preceding code is from the log in the browser. 
Here, the images move farther and farther to the left until they eventually reach the maximum length 
of the signed 16-bit integer. 
This is happening because we're never removing an obstacle from the obstacles
Vec when they go off screen, and we should. 

Let's add a line of code to the update function that goes 
right before we move and collide with the obstacles, as shown here:


```rust
// src/game.rs

impl Game for WalkTheDog {
    ...
    fn update(&mut self, keystate: &KeyState) {
        if let WalkTheDog::Loaded(walk) = self {
            ...
            // removing an obstacle from the obstacles
            // Vec when they go off screen
            walk.obstacles.retain(|obstacle|
                obstacle.right() > 0);
            
            // move and collide with the obstacles
            walk.obstacles.iter_mut().for_each(|obstacle| {
                obstacle.move_horizontally(velocity);
                obstacle.check_intersection(&mut walk.boy);
            });
    ...


```

The `retain` function will keep any obstacles that match the predicate that's been passed in. 
In this case, this will happen if the rightmost point of the obstacle is to the right of the left edge 
of the screen. 

This means we're looping through the list of obstacles twice.

If we were using the nightly build of Rust, we could use the `drain_filter` function to
avoid that, but our obstacles list should never be long enough for that to be an issue.

For this code to compile, you'll need to add one more method to the `Obstacle trait` – the
`right` method for the rightmost point of Obstacle. 

This can be seen in the following code:

```rust
// src/game.rs

trait Obstacle {
    ...
    fn right(&self) -> i16;
}

```

This method will need to be added to both the `Platform` and `Barrier` implementations of `Obstacle`. 
`Barrier` can just delegate to the image it's holding,
`Platform` is a little trickier because it has more than one box. 
We want to use the right edge of the **last** bounding box, as shown here:

```rust
// src/game.rs

impl Obstacle for Platform {
    ...
    fn right(&self) -> i16 {
        self.bounding_boxes()
            .last()
            .unwrap_or(&Rect::default())
            .right()
    }
}

```
This code gets the last bounding box with `last` and unwraps it since last returns an
Option. We don't want to return a Result and then force everybody to use a Result ,
so we are using `unwrap_or(&Rect::default())` to return an empty `Rect` when
`Platform` has no bounding boxes. 

One empty bounding box is effectively the same as no bounding boxes. 
Then, we get the rightmost value of the last Rect with right.

Rect doesn't have a default implementation yet, so we'll need to add a
`#[derive(Default)]` annotation to the `Rect` and `Point` structures in engine.


```rust
// src/engine.rs

#[derive(Clone, Copy, Default)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

#[derive(Default)]
pub struct Rect {
    pub position: Point,
    pub width: i16,
    pub height: i16,
}


```

The annotation automatically implements the `Default trait` for a struct by using the
default value of every field in that struct. 
Point will need the annotation because it is in the `Rect` structure, 
so for the macro to work for `Rect`, it must also work for Point. 

Fortunately, there's no real harm in adding this to them.

With that, you can let `RHB` run for as long as he wants, with no **buffer overflow**. 

Now, we need to give `RHB` many platforms to jump on. 

We will start by sharing the sprite sheet.
Let's dig into this last piece of refactoring.

------------


```rust
// src/game.rs



```

