## Colliding with an obstacle

To have collisions, we'll have to actually put the bounding boxes we've seen on both
RHB and the stone. Then, in the update function of WalkTheDog , we'll need to detect
that collision, and when that collision happens, we'll move RHB into the Falling
and KnockedOut states, which correspond to the Dead animation in the sprite sheet.
Much of that code, particularly the state machine, will be very familiar.

### A bounding box for a stone

The stone is the simplest of the bounding boxes because we can just use the size of `HTMLImageElement`. 
This won't always be the case. If you look at the images of the stone with a bounding box around it, 
you will notice that it is larger than the stone's actual size, particularly at the corners. 
For the time being, this will be good enough, but as we proceed, we'll need to keep this in mind.

To add a bounding box to the `Image` implementation, which is in the `engine` module,
we'll want to calculate the bounding box when `Image` is created, in its new function, as
shown here:

```rust
// src/engine.rs

pub struct Image {
    element: HtmlImageElement,
    position: Point,
    bounding_box: Rect,
}

impl Image {
    pub fn new(element: HtmlImageElement, position: Point) -> Self {
        let bounding_box = Rect {   x: position.x.into(),
                                    y: position.y.into(),
                                    width: element.width() as f32,
                                    height: element.height() as f32,
                            };


        //Self { element, position }
        Self {  element,
                position,
                bounding_box,
        }

    ...
}

```

Here, we've added `bounding_box` to the `Image` struct , and we construct it in the
new function using `width` and `height` from its `HTMLImageElement` backing. 
It's worth noting that we had to cast the element.width() and element.height() calls to f32. 
This should be safe, but if later we're drawing a very large image, then it may become a problem. 

It's also worth noting that by creating the bounding box in the new function, 
we're ensuring that anytime position is updated, we also need to update `bounding_box`. 
We could work around this by calculating bounding_box every time, and that's a fine solution, 
but it does mean potentially losing performance. 

In this case, we'll keep both `position` and `bounding_box` private in struct to ensure 
they don't get out of sync. `Image` objects don't move yet, anyway.
Given that `bounding_box` is private, we'll need to give it an `accessor`, 
so let's do that now:

```rust
// src/engine.rs

impl Image {
    ...
    pub fn bounding_box(&self) ->&Rect {
        &self.bounding_box
    }
}
```

Now we can use `bounding_box` for our `draw_rect`

```rust
// src/engine.rs

impl Image {
    ...

    /*
    pub fn draw_rect(&self, renderer: &Renderer) {
        renderer.draw_rect( &Rect{ x:self.position.x.into(), 
                                   y: self.position.y.into(), 
                                   width: self.element.width() as f32, 
                                   height: self.element.height() as f32}
        );
    }
    */

    pub fn draw_rect(&self, renderer: &Renderer) {
        renderer.draw_rect(self.bounding_box());
    }
...
}
```


### A bounding box for RedHatBoy

The bounding box on `RedHatBoy` is a little more complicated for the same reasons that
the sprite sheet was more complicated. It needs to align with where the sheet is, and it
needs to adjust based on the animation. 
Therefore, we won't be able to do what we did for `Image` and store one `bounding_box` tied to the object. 
Instead, we'll calculate its bounding box based on its current state and the sprite sheet. 

The code in `game.rs` will actually look very similar to draw , as seen here:


```rust
// src/game.rs

impl RedHatBoy {
    ...
    fn bounding_box(&self) ->Rect {
        let frame_name = format!(
            "{} ({}).png",
            self.state_machine.frame_name(),
            (self.state_machine.context().frame / 3) + 1
        );
        let sprite = self
            .sprite_sheet
            .frames
            .get(&frame_name)
            .expect("Cell not found");

        Rect {
                x: (self.state_machine.context().position.x
                + sprite.sprite_source_size.x as i16).into(),
                y: (self.state_machine.context().position.y
                + sprite.sprite_source_size.y as i16).into(),
                width: sprite.frame.w.into(),
                height: sprite.frame.h.into(),
        }
    }
```

To calculate `bounding_box`, we start by creating `frame_name` from the state name and
the current frame, just like how we did in the draw, and then we calculate `Rect` from
those values using the same calculations we did when we updated the draw function.

In fact, it's a good time to clean up some of the duplications in those two pieces of code,
using refactoring. 

Let's extract functions to get the frame and sprite name, still in the RedHatBoy implementation:


```rust
// src/game.rs

use crate::engine::Cell;
...

impl RedHatBoy {
    ...

    fn frame_name(&self) -> String {
        format!(
            "{} ({}).png",
            self.state_machine.frame_name(),
            (self.state_machine.context().frame / 3) + 1
        )
    }

    fn current_sprite(&self) -> Option<&Cell> {
        self
            .sprite_sheet
            .frames
            .get(&self.frame_name())
    }

```

Don't forget `use crate::engine::Cell;`. Compiler will tell you anyway :-)


```rust
// src/game.rs

    fn bounding_box(&self) -> Rect {
        /*        
        let frame_name = format!(
            "{} ({}).png",
            self.state_machine.frame_name(),
            (self.state_machine.context().frame / 3) + 1
        );
        
        let sprite = self
            .sprite_sheet
            .frames
            .get(&frame_name)
            .expect("Cell not found");
        */

        let sprite = self.current_sprite().expect("Cell not found");

        Rect {
                x: (self.state_machine.context().position.x
                + sprite.sprite_source_size.x as i16).into(),
                y: (self.state_machine.context().position.y
                + sprite.sprite_source_size.y as i16).into(),
                width: sprite.frame.w.into(),
                height: sprite.frame.h.into(),
        }
    }

```

Going further, we can shrink draw by removing the duplicated code from bounding_
box and making a much smaller draw function:


```rust
// src/game.rs

impl RedHatBoy {
    ...
    fn draw(&self, renderer: &Renderer) {
        /*let frame_name = format!(
            "{} ({}).png",
            self.state_machine.frame_name(),
            (self.state_machine.context().frame / 3) + 1
        );

        let sprite = self
            .sprite_sheet
            .frames
            .get(&frame_name)
            .expect("Cell not found");
        */
        let sprite = self.current_sprite().expect("Cell not found");
```

And the second `Rect` can be replaced by `bounding_box`


```rust
// src/game.rs

impl RedHatBoy {
    ...
    fn draw(&self, renderer: &Renderer) {
        let sprite = self.current_sprite().expect("Cell not found");

        renderer.draw_image(
            &self.image,
            &Rect {
                x: sprite.frame.x.into(),
                y: sprite.frame.y.into(),
                width: sprite.frame.w.into(),
                height: sprite.frame.h.into(),
            },
            /*
            &Rect {
                //x: self.state_machine.context().position.x.into(),
                //y: self.state_machine.context().position.y.into(),
                x: (self.state_machine.context().position.x
                + sprite.sprite_source_size.x as i16).into(),
                y: (self.state_machine.context().position.y
                + sprite.sprite_source_size.y as i16).into(),
                width: sprite.frame.w.into(),
                height: sprite.frame.h.into(),
            },
            */

            &self.bounding_box(),
        );
    }
...
```

so the draw is now just

```rust
// src/game.rs

impl RedHatBoy {
    ...
    fn draw(&self, renderer: &Renderer) {
        let sprite = self.current_sprite().expect("Cell not found");

        renderer.draw_image(
            &self.image,
            &Rect {
                x: sprite.frame.x.into(),
                y: sprite.frame.y.into(),
                width: sprite.frame.w.into(),
                height: sprite.frame.h.into(),
            },
            &self.bounding_box(),
        );
    }
...
```

This makes for must smaller, cleaner implementations, but it's worth paying attention to
the fact that we're looking up current_sprite twice on every frame. We won't work
to fix it now because we're not seeing any troubles, but we may want to memoize this
value later.

Now that we have both bounding boxes, we can actually see whether RHB collides with
the stone.


