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

### Crashing on the collision

To crash on a collision, we'll need to check whether the two rectangles intersect.
We'll add that code to Rect in the `engine` module. 
That code is the implementation on the Rect struct, shown here:

```rust
// src/engine.rs

impl Rect {
    pub fn intersects(&self, rect: &Rect) -> bool {
        self.x < (rect.x + rect.width)
        && self.x + self.width > rect.x
        && self.y < (rect.y + rect.height)
        && self.y + self.height > rect.y
    }
}
```

We check to see whether there is any overlap and returning true if there is. 
Every time you see `rect.x + rect.width`, that's the right side, 
and `rect.y + height` is the bottom. 
Personally, I prefer to put the same rectangle on the left-hand side of this function 
for every condition, as I find it easier to read and think about. 

We'll use this code in the update function of `impl Game for WalkTheDog`. 
That code is small, but it will cause a chain reaction. 
The collision code is just after `walk.boy.update` is as follows:

```rust
// src/game.rs

impl Game for WalkTheDog {
    ...
    fn update(&mut self, keystate: &KeyState) {
        if let WalkTheDog::Loaded(walk) = self {
            if keystate.is_pressed("ArrowRight") {
            ...
            }

            walk.boy.update();
            if walk.boy
                   .bounding_box()
                   .intersects(walk.stone.bounding_box())
            {
                walk.boy.knock_out();
            }
        }
    }//^-- update
```

The check for collisions will happen right after the call to `update` on boy. 
We check whether the boy's bounding box has intersected the stone's with our brand new
`intersects` function, and if it has, we use `knock_out` on the RHB. 

The `knock_out` function doesn't exist yet; 
creating it will mean updating our state machine. 
The `KnockOut` event will cause a transition into the `Falling` state, which will
then transition into the `KnockedOut` state when the `Falling` animation has completed.

What are we waiting for? Let's knock out RHB!

#### A KnockOut event

We'll add new states to `RedHatBoyStateMachine` and "follow the compiler" to know where to fill 
in the necessary code. Rust's type system does a great job of making this kind of work easy,
giving useful error messages along the way, so I'm only going to highlight passages that are unique.

You can get started in the `game` module by adding a `KnockOut` event to `Event` enum
as shown below:

```rust
// src/game.rs


pub enum Event {
    Run,
    Slide,
    Update,
    Jump,
    KnockOut,
}
```

and a `knock_out` method onto `RedHatBoy` as with the other state machine transitions,

```rust
// src/game.rs

impl RedHatBoy {
    ...
    fn knock_out(&mut self) {
        self.state_machine = self.state_machine.transition(Event::KnockOut);
    }
    ...
}
```

This will just move the compiler error into `RedHatBoyStateMachine` because
match statements are incomplete, so you'll need to add a `KnockOut` event to
`RedHatBoyStateMachine` that will transition from `Running` to `Falling`. 

That transition is like so:

```rust
// src/game.rs

impl RedHatBoyStateMachine {
    fn transition(self, event: Event) -> Self {
        match (self, event) {
            (RedHatBoyStateMachine::Idle(state), Event::Run) => state.run().into(),
            ...
            (RedHatBoyStateMachine::Running(state), Event::KnockOut) => state.knock_out().into(),
            (RedHatBoyStateMachine::Jumping(state), Event::KnockOut) => state.knock_out().into(),
            (RedHatBoyStateMachine::Sliding(state), Event::KnockOut) => state.knock_out().into(),
           _ => self,
        }
    }//^-- fn transition
```

You might wonder why we also have transitions from `Jumping` and `Sliding` to
`Falling`; that's because if we don't do that, then the user can simply hold down the
spacebar to jump continuously, or slide at the right time, and they will pass right through
the stone. So, we need to make sure that all three of those states will transition to
Falling in order for the game not to have any bugs.

`Falling` doesn't exist yet, neither as a member of the `RedHatBoyStateMachine` enum nor as a `struct`. 
The typestates for `Sliding`, `Jumping`, or `Running` don't have `knock_out` methods, 
and there's no `From trait` implemented to convert from `Falling` into `RedHatBoyStateMachine::Falling`.

You'll need to add both of those, just like before, and fill in the rest of the compiler errors.

You'll find that you need two new constants, the number of frames in the falling animation
and the name of the falling animation in the sprite sheet. 
You can look at rhb.json and figure out the values, or look at the following listings:

```rust
// src/game.rs

mod red_hat_boy_states {
    use crate::engine::Point;

    const FLOOR: i16 = 479;
    ...
    const FALLING_FRAMES: u8 = 29; // 10 'Dead' frames in the sheet, * 3 - 1.
    const FALLING_FRAME_NAME: &str = "Dead";
```


If you've made all the proper boilerplate changes, you'll end up making a transition from
Running to Falling that looks like the following code:

```rust
// src/game.rs

mod red_hat_boy_states {
    use crate::engine::Point;

    ...

    impl RedHatBoyState<Running> {
        pub fn slide(self) -> RedHatBoyState<Sliding> {
            ...
        pub fn knock_out(self) -> RedHatBoyState<Falling> {
            RedHatBoyState { 
                             context: self.context,
                             _state: Falling {},
            }
        }
        ...
    }
```
The transition properly moves into the Dead animation, but it doesn't stop RHB's forward
motion. Let's change the transition to stop RedHatBoy :

```rust
// src/game.rs

mod red_hat_boy_states {
    use crate::engine::Point;

    ...

    impl RedHatBoyState<Running> {
        pub fn knock_out(self) -> RedHatBoyState<Falling> {
            RedHatBoyState { //context: self.context,   
                             context: self.context.reset_frame().stop(),
                             _state: Falling {},
            }
        }
```

```rust
// src/game.rs

mod red_hat_boy_states {

    ...
    impl RedHatBoyContext {
        ...
        fn run_right(mut self) -> Self {
            ...
        }

        fn stop(mut self) -> Self {
            self.velocity.x = 0;
        }
    }
}
```

You'll want to do the same transition when going from `Sliding` to `Falling` 
and `Jumping` to `Falling` as well so that the transitions match. 
That will halt the character's forward motion but will not stop the death animation 
from playing over and over again.

That's because we never transition out of the `Falling` state and into `KnockedOut`,
which itself doesn't exist yet. 

Fortunately, we've done code like this before. 
Remember in Chapter 4, Managing Animations with State Machines, 
we transitioned out of the `Sliding` animation and back into the `Running` animation 
when the slide animation was complete. 

That code, which is in the update function of `RedHatBoyState<Sliding>`, 
is reproduced here

```rust
// src/game.rs

    impl RedHatBoyState<Sliding> {
        ...
        pub fn update(mut self) -> SlidingEndState {
            self.update_context(SLIDING_FRAMES);

            if self.context.frame >= SLIDING_FRAMES {
                SlidingEndState::Running(self.stand())
            } else {
                SlidingEndState::Sliding(self)
            }
        }


```

In this code, we check every `update` and see whether the `Sliding` animation is complete
via if `state_machine.context.frame>= SLIDING_FRAMES`. 
If it is, we return the `Running` state instead of the `Sliding` state. 

In order to get this far, you already had to add an `update` method to `RedHatBoyState<Falling>`, 
likely with a generic default that played the animation. 

Now, you'll need to mimic this behavior and transition into the new `KnockedOut` state. 
Specifically, you'll need to do the following:

1. Create a `KnockedOut` state.
2. Create a transition from `Falling` to `KnockedOut`.
3. Check in the `update` action whether the `Falling` animation is complete, and if
so, transition to the `KnockedOut` state instead of staying in `Falling`.
4. Create an enum to handle both end states of the update method in
`RedHatBoyState<Falling>`, as well as the corresponding `From trait`, to
convert from that to the `RedHatBoyStateMachine` -appropriate enum variant.


1. Create a `KnockedOut` state.

```
   #[derive(Copy, Clone)]
   pub struct KnockedOut;

   impl RedHatBoyState<KnockedOut> {
        pub fn frame_name(&self) -> &str {
            FALLING_FRAME_NAME
        }
    }

```

The only thing new here is that RedHatBoyState<KnockedOut> will not need
the update method because, in the KnockedOut state, RHB doesn't do anything.
We won't go through that code step by step, and instead, I highly encourage you to
try it yourself.



2. Create a transition from `Falling` to `KnockedOut`.

```
    #[derive(Copy, Clone)]
    pub struct Falling;


    pub enum FallingEndState {
        Falling(RedHatBoyState<Falling>),
        KnockedOut(RedHatBoyState<KnockedOut>),
    }

    impl RedHatBoyState<Falling> {
        pub fn frame_name(&self) -> &str {
            FALLING_FRAME_NAME
        }

        pub fn knock_out(self) -> RedHatBoyState<KnockedOut> {
            RedHatBoyState {
                context: self.context,
                _state: KnockedOut {},
            }
        }

```



3. Check in the `update` action whether the `Falling` animation is complete, and if
so, transition to the `KnockedOut` state instead of staying in `Falling`.

```        
        pub fn update(mut self) -> FallingEndState {
            self.update_context(FALLING_FRAMES);
            if self.context.frame >= FALLING_FRAMES {
                FallingEndState::KnockedOut(self.knock_out())
            } else {
                FallingEndState::Falling(self)
            }
        }

```

4. Create an enum to handle both end states of the update method in
`RedHatBoyState<Falling>`, as well as the corresponding `From trait`, to
convert from that to the `RedHatBoyStateMachine` -appropriate enum variant.

```
#[derive(Copy, Clone)]
enum RedHatBoyStateMachine {
    Idle(RedHatBoyState<Idle>),
    ...
    Falling(RedHatBoyState<Falling>),
    KnockedOut(RedHatBoyState<KnockedOut>),
}
...
impl From<RedHatBoyState<Falling>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Falling>) -> Self {
        RedHatBoyStateMachine::Falling(state)
    }
}

impl From<RedHatBoyState<KnockedOut>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<KnockedOut>) -> Self {
        RedHatBoyStateMachine::KnockedOut(state)
    }
}

impl From<FallingEndState> for RedHatBoyStateMachine {
    fn from(state: FallingEndState) -> Self {
        match state {
            FallingEndState::Falling(falling) => falling.into(),
            FallingEndState::KnockedOut(knocked_out) => knocked_out.into(),
        }
    }
}

```

The transitions

```rust 

impl RedHatBoyStateMachine {
    fn transition(self, event: Event) -> Self {
        match (self, event) {
            (RedHatBoyStateMachine::Idle(state), Event::Run) => state.run().into(),
            (RedHatBoyStateMachine::Running(state), Event::Jump) => state.jump().into(),
            (RedHatBoyStateMachine::Running(state), Event::Slide) => state.slide().into(),

            (RedHatBoyStateMachine::Idle(state), Event::Update) => state.update().into(),
            (RedHatBoyStateMachine::Running(state), Event::Update) => state.update().into(),
            (RedHatBoyStateMachine::Jumping(state), Event::Update) => state.update().into(),
            (RedHatBoyStateMachine::Sliding(state), Event::Update) => state.update().into(),
            (RedHatBoyStateMachine::Falling(state), Event::Update) => state.update().into(),

            (RedHatBoyStateMachine::Running(state), Event::KnockOut) => state.knock_out().into(),
            (RedHatBoyStateMachine::Jumping(state), Event::KnockOut) => state.knock_out().into(),
            (RedHatBoyStateMachine::Sliding(state), Event::KnockOut) => state.knock_out().into(),
```

