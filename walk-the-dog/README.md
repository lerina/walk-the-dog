## Drawing RedHatBoy

The interface we want is to say `self.rhb.draw()` and see RHB drawing the `idle` animation. 
We also want to call the `run` function when we push the right arrow and see RHB run.

Let's start by implementing `draw` on RedHatBoy. 
We'll create a draw function that will mimic the draw function in WalkTheDog 
only using the shared `RedHatBoyContext` that's in `RedHatBoyState`. 
That code is as follows, written as part of the impl RedHatBoy block:

```rust
// filename: src/game.rs

impl RedHatBoy {
...
    fn draw(&self, renderer: &Renderer) {
        let frame_name = format!("{} ({}).png", 
                                 self.state_machine.frame_name(),
                                 (self.state_machine.context().frame / 3) + 1);
        let sprite = self.sprite_sheet
                        .frames
                        .get(&frame_name)
                        .expect("Cell not found");

        renderer.draw_image(
                            &self.image,
                            &Rect {
                                x: sprite.frame.x.into(),
                                y: sprite.frame.y.into(),
                                width: sprite.frame.w.into(),
                                height: sprite.frame.h.into(),
                            },
                            &Rect {
                                x: self.state_machine.context()
                                .position.x.into(),
                                y: self.state_machine.context()
                                .position.y.into(),
                                width: sprite.frame.w.into(),
                                height: sprite.frame.h.into(),
                            },);
        }//^-- fn draw
}//^-- impl 

```

This is nearly identical to the code that exists in the draw function already for our
happily running RHB. 
Instead of always using the Run animation, now we're dynamically choosing the animation 
based on the state of the system, via the frame_name function, which doesn't exist yet.

We're also getting `position` and `frame` off `context()` , another function that doesn't
exist yet. 

Again, we'll let the compiler guide us to create both of these functions; 
Compiler-Driven Development strikes again! 
The `RedHatBoyStateMachine` enum needs to provide a way to return `RedHatBoyContext` 
and `frame_name`. 
We can add those implementations, as follows:

```rust
// filename: src/game.rs

impl RedHatBoyStateMachine {
    ...
    fn frame_name(&self) ->&str {
        match self {
            RedHatBoyStateMachine::Idle(state) => state.frame_name(),
            RedHatBoyStateMachine::Running(state) => state.frame_name(),
        }
    }

    fn context(&self) ->&RedHatBoyContext {
        match self {
            RedHatBoyStateMachine::Idle(state) =>&state.context(),
            RedHatBoyStateMachine::Running(state) =>&state.context(),
        }
    }

}

```

I admit I don't love either of these methods and did consider creating a trait that the
various states would implement as an alternative. After some thought, I decided this was
simpler, and because the Rust compiler will fail if you don't match every single enum
variant, I'm willing to accept these duplicate case statements.

The `frame_name` and `context` methods both delegate to the currently active state
to get the data that's required. 

In the case of `frame_name`, this will be a method that
returns the name of the animation in `rhb.json` for a given state as defined on each state.

The `context` method is particularly odd because we always return the same field for
every single state and always will, as that data is shared across all the states. 

That's going to require a generic implementation, which we'll write in a moment. 


---

NOTE:

You might have noticed that the line self.state_machine.
context().position.x violates the Law of Demeter and objects to
it. The Law of Demeter is a style guideline for OO code that states that you
should only talk to your immediate friends, and in this case, self should only
talk to state_machine (its friend) but instead, it talks to position
via context . This couples RedHatBoy to the internal structure of
RedHatBoyContext in a way that could be avoided by adding getters for
position_x and position_y on state machine , which would
delegate to context , which would, in turn, delegate to position . The Law
of Demeter is a great guideline when setting values, and you should almost
always follow it for mutable data, but in this case the data is immutable. We
can't change the context through this getter, and the downsides of violating the
Law of Demeter are not as relevant. I don't feel it's necessary to create more
delegating functions just to avoid violating an arbitrary guideline, but if it
becomes a problem, we can always change it. For more information on this, go
to https://wiki.c2.com/?LawOfDemeter.

---

frame_name is more
straightforward, so we'll implement it first. It's a getter of the name of the frame in the
rhb.json file, and it's different for every state, so we'll put that method on every state, as
shown here:

```rust
// filename: src/game.rs

mod red_hat_boy_states {
use crate::engine::Point;
const FLOOR: i16 = 475;
const IDLE_FRAME_NAME: &str = "Idle";
const RUN_FRAME_NAME: &str = "Run";

impl RedHatBoyState<Idle> {
    ...
    pub fn frame_name(&self) -> &str {
        IDLE_FRAME_NAME
    }
}

...
impl RedHatBoyState<Running> {
    pub fn frame_name(&self) -> &str {
        RUN_FRAME_NAME
    }
}
...
```

We've added two constants, `IDLE_FRAME_NAME` and `RUN_FRAME_NAME` ,
which correspond to the names of the frames for the `Idle` and `Run` sections of
our sprite sheets, respectively. We then created a new method, `frame_name`,
on `RedHatBoyState<Idle>` as well as an entirely new implementation for
`RedHatBoyState<Running>` , which also has a frame_name method.

It's worth thinking about whether we could use a trait object ( https://bit.
ly/3JSyoI9 ) instead of our enum for RedHatBoyStateMachine , and it probably is
possible.

Let's add a `context` method.
That method is going to do the same thing for every state, return the context, 
and we can write it generically for all of them, as shown here:

```rust
// filename: src/game.rs

mod red_hat_boy_states {
....
#[derive(Copy, Clone)]
pub struct RedHatBoyState<S> {
    context: RedHatBoyContext,
    _state: S,
}

impl<S> RedHatBoyState<S> {
    pub fn context(&self) -> &RedHatBoyContext {
        &self.context
    }
}

```

This is a pretty cool feature of Rust. Since we have a generic struct, we can write methods
on the generic type, and it will apply to all the types. 

Finally, there is one more compiler error, in the draw function where we reference the frame or position fields on context.
These fields are private, but as long as `RedHatBoyContext` is an immutable type, we can
each make of those public, as follows:

```rust
// filename: src/game.rs

mod red_hat_boy_states {
...
#[derive(Copy, Clone)]
pub struct RedHatBoyContext {
    pub frame: u8,
    pub position: Point,
    pub velocity: Point,
}
...
```

Finally, we need to call that method on `RedHatBoy` in the `WalkTheDog#draw`
function. 
You can add that in this, admittedly awkward, one-liner right at the end of the
draw function:

```rust
// filename: src/game.rs

#[async_trait(?Send)]
impl Game for WalkTheDog {
    ...

    fn draw(&self, renderer: &Renderer) {
        ...
        self.rhb.as_ref().unwrap().draw(renderer);
    }
```


This prepared us for what we're going to do next, moving RHB around and changing
animations. 
Speaking of animations, the Idle version of RHB isn't doing anything yet, because frame never changes. 
When RHB is idle, he stands while breathing slowly, so let's get that started, shall we?

### Updating RHB

Our `RedHatBoy` struct is going to have an update function, which will, in turn,
delegate to an update function on the state machine. 
It's a new method because every state is going to need to update, 
in order to advance the animation. 
We'll call update on `RedHatBoy` from update on `WalkTheDog`. 
That's a lot of updates, but it's really just delegation:

```rust
// filename: src/game.rs

#[async_trait(?Send)]
impl Game for WalkTheDog {
    ...

    fn update(&mut self, keystate: &KeyState) {
        let mut velocity = Point { x: 0, y: 0 };
        
        if keystate.is_pressed("ArrowDown") { velocity.y += 3; }
        if keystate.is_pressed("ArrowUp") { velocity.y -= 3; }
        if keystate.is_pressed("ArrowRight") { velocity.x += 3; }
        if keystate.is_pressed("ArrowLeft") { velocity.x -= 3; }
        if keystate.is_pressed("ArrowLeft") { velocity.x -= 3; }
        
        self.position.x += velocity.x;
        self.position.y += velocity.y;

        self.rhb.as_mut().unwrap().update();
}

impl RedHatBoy {
    ...
    fn update(&mut self) {
        self.state_machine = self.state_machine.update();
    }
}

impl RedHatBoyStateMachine {
    ...
    fn update(self) -> Self {
        match self {
            RedHatBoyStateMachine::Idle(mut state) => {
                if state.context.frame < 29 {
                    state.context.frame += 1;
                } else {
                    state.context.frame = 0;
                }

                RedHatBoyStateMachine::Idle(state)
            }
            RedHatBoyStateMachine::Running(_) => self,
        }
    }
}

```

In the update function on WalkTheDog , we've only added one new line, at the end of
the update function:

```rust
self.rhb.as_mut().unwrap().update();
```

The `unwrap()` is because of the fact that `rhb` is Option, and we'll fix that in a little bit. 

We've added another small function to the `RedHatBoy` struct update that simply updates
`state_machine` via the state machine's `update` function. 
This one line, and others like it, are why the state machine needs to be Copy. 
If it's not, then because `update` consumes `self` via the parameter of `mut self`, 
you'd have to use something like `Option` to move `self` into `update`, 
and then reset it again. 
By making everything `Copy`, you get a much more ergonomic `update` function.

Finally, the meat of the behavior is in the `RedHatBoyStateMachine#update` function. 
Here, we match on `self` and update the current frame on a mutable state parameter, 
and then return a new `Idle` state with a moved `context` with an updated frame. 

Unfortunately, this code doesn't compile; 
`context` isn't a public data member so you can't assign it. 

For now, we'll go ahead and make `context` public, but 

> this should bother you.
 
Remember that `Law of Demeter` guideline. 

It's one thing to get an immutable data value, 
another thing entirely to set a mutable value. 

This is the kind of coupling that could cause real problems down the line. 
We're not going to fix it right now, so go ahead and make `context` public, 
but we will be keeping a very close eye on this code.

At this point, if you look at `update` for `WalkTheDog` and `update` for
`RedHatBoyStateMachine`, you'll see similarities. 
One is updating the running RHB in the upper left corner, 
and one is updating the idle RHB in the lower left. 
The time has come to begin combining these two objects. 
Let's go ahead and do that.

### Adding the Running state

One thing to keep in mind about states is that they exist whether you
implement a state machine or not. While we haven't implemented anything in
`RedHatBoyState<Running>` , the `Running` state currently exists in `WalkTheDog`;
RHB is running all around the void right now! We just need to move the details into our
state machine, so that we as programmers can actually see the states and what they do
as one coherent unit.

We can do that quickly by just modifying `update` in `RedHatBoyStateMachine` to
match the version in `Idle`, with the different frame count for the run animation. 
That's shown as follows:

```rust
// filename: src/game.rs

impl RedHatBoyStateMachine {
...
    fn update(self) -> Self {
        match self {
            RedHatBoyStateMachine::Idle(mut state) => {
                ...

                RedHatBoyStateMachine::Idle(state)
            },
            //RedHatBoyStateMachine::Running(_) => self,
            RedHatBoyStateMachine::Running(mut state) => {
                if state.context.frame < 23 {
                    state.context.frame += 1;
                } else {
                    state.context.frame = 0;
                }

                RedHatBoyStateMachine::Running(state)
            }

        }
    }
}//^-- impl 
```

Now, the state machine is theoretically capable of drawing the run animation, 
but we haven't written anything to cause that transition. 

The other thing missing is potentially more subtle. 
The Running animation has 23 frames, and the Idle animation has 29 .
If we were to transform from Idle to Running with the frame count at 24, the game
would crash.


Finally, I think we can all agree that the kind of duplication 
that we have here can be improved. 
The only difference between the two functions is the frame count. 

---

> So, we have a few things to do:

1. Refactor the duplicated code.
2. Fix the Law of Demeter violation.
3. Move RHB on every update .
4.Ensure that the frame count resets to 0 when transitioning between states.
5. Start Running on transition.
6. Start Running on the right arrow.
7. Delete the original code.

---

#### Refactor the duplicated code.
The code that updates `context.frame` suffers from a code smell called 
Feature Envy ( https://bit.ly/3ytptHA ) because the update function is 
operating over and over again on context. 
Why not move that function to RedHatBoyContext? That's shown here:

```rust
// filename: src/game.rs

const IDLE_FRAMES: u8 = 29;
const RUNNING_FRAMES: u8 = 23;
...
impl RedHatBoyStateMachine {
    fn update(self) -> Self {
        match self {
            RedHatBoyStateMachine::Idle(mut state) => {
                state.context = state.context.update(IDLE_FRAMES);
                RedHatBoyStateMachine::Idle(state)
            }
            RedHatBoyStateMachine::Running(mut state) => {
                state.context = state.context.update(RUNNING_FRAMES);
                RedHatBoyStateMachine::Running(state)
            }
        }
    }

}//^-- impl
```

And now we put the duplicated code in `impl RedHatBoyContext` in a new `update`

```rust
mod red_hat_boy_states {
...

    #[derive(Copy, Clone)]
    pub struct RedHatBoyContext {
    ...
    }

    impl RedHatBoyContext {
        pub fn update(mut self, frame_count: u8) -> Self {
            if self.frame < frame_count {
                self.frame += 1;
            } else {
                self.frame = 0;
            }
            self
        }
    }
```
RedHatBoyContext now has an update function that increments the frame, 
looping it back to 0 when the total frame count is reached. 

Note how it works the same way as our transitions, consuming self, 
and returning a new RedHatBoyContext, although in reality, 
it's the same instance the entire time.

This gives us the same kind of functional interface that we're using elsewhere. 
The total frame count changes with each state, so we pass that in as a parameter, 
using constants for clarity.



#### Fix the Law of Demeter violation.

Looking at the two arms of each match statement, they are nearly identical, 
both mutating context in the way we didn't like earlier. 

Now is a good time to address it, which we can do by making the field private 
on `RedHatBoyState<S>` again, 

```rust
// filename: src/game.rs


    #[derive(Copy, Clone)]
    pub struct RedHatBoyState<S> {
        // pub context: RedHatBoyContext,
        context: RedHatBoyContext,
        _state: S,
    }
```
and creating new `update` methods on the respective `RedHatBoy` state implementations, as
shown here:

```rust
// filename: src/game.rs

mod red_hat_boy_states {
    ...
    const IDLE_FRAMES: u8 = 29;
    const RUNNING_FRAMES: u8 = 23;
    ....
    impl RedHatBoyState<Idle> {
        ....
        pub fn update(&mut self) {
            self.context = self.context.update(IDLE_FRAMES);
        }
    }

    impl RedHatBoyState<Running> {
        ...
        pub fn update(&mut self) {
            self.context = self.context.update(RUNNING_FRAMES);
        }
    }
}

```

Make sure you move the `RUNNING_FRAMES` and `IDLE_FRAMES` constants 
into the `red_hat_boy_states` module.

```
mod red_hat_boy_states {
    use crate::engine::Point;
    ...
    const IDLE_FRAMES: u8 = 29;
    const RUNNING_FRAMES: u8 = 23;

    ...
```

`context` is no longer inappropriately public, and each
individual state handles its own updating. The only difference between them is the
constant they use, and it's fitting to have that bundled with the implementation
itself. 


We'll need to modify the update method on RedHatBoyStateMachine to call
this new method on each of the states:

```rust
// filename: src/game.rs

    impl RedHatBoyStateMachine {
        ....
        fn update(self) -> Self {
            match self {
                RedHatBoyStateMachine::Idle(mut state) => {
                    state.update();
                    RedHatBoyStateMachine::Idle(state)
                }
                RedHatBoyStateMachine::Running(mut state) => {
                    state.update();
                    RedHatBoyStateMachine::Running(state)
                }
            }
        }
    }
```

Each of the arms in update now updates the state, and then returns the state.
There's some duplication here that's a little suspicious; 
we'll take another look at that shortly.


#### Move RHB on every update

If RHB is going to run in the running state, it needs to respect the velocity. 
In other words, update animates the frame, but it doesn't move, 
so let's add that to the `RedHatBoyContext` `update` method:

```rust
// filename: src/game.rs

    fn update(mut self, frame_count: u8) -> Self {
        ...
        self.position.x += self.velocity.x;
        self.position.y += self.velocity.y;
        self
    }
```

Of course, RHB won't move yet because we aren't changing the velocity. That will
come soon.

#### Ensure that the frame count resets to 0 when transitioning between states.

There are two categories of changes on the game object that can happen in our state
machine. 
- There are changes that happen when the state doesn't change. That's what
update is and right now those are written in `RedHatBoyStateMachine`. 
- There are also changes that happen on a transition, and those happen in the transition
functions that are defined as methods of the type classes.

We already transitioned from `Idle` to `Running` via the `run` method, and we can
make sure to reset the frame rate on the transition. That's a small change you can
see here:

```rust
// filename: src/game.rs

impl RedHatBoyContext {
    ...
    fn reset_frame(mut self) -> Self {
        self.frame = 0;
        
        self
    }
}
```

```rust
// filename: src/game.rs

impl RedHatBoyState<Idle> {
    ....
    pub fn run(self) -> RedHatBoyState<Running> {
        RedHatBoyState {
            //context: self.context,
           context: self.context.reset_frame(),
           _state: Running {},
        }
    }
}
```


`RedHatBoyContext` has grown a function called `reset_frame`, which resets its
frame count to 0 and returns itself. By returning itself, we can chain calls together,
which will come in handy shortly. 
The run method has also evolved to call `reset_frame()` on `RedHatBoyContext` 
and use that new version of context in the new `RedHatBoyState` struct.


#### Start Running on transition.

Now that we have prevented crashes by restarting animations on transitions, let's
start running forward on a transition. This is going to be very short:

```rust
// filename: src/game.rs

mod red_hat_boy_states {
    ....
    const RUNNING_SPEED: i16 = 3;

    ...
    impl RedHatBoyContext {
        ...
        fn run_right(mut self) -> Self {
            self.velocity.x += RUNNING_SPEED;
            
            self
        }
    }


```

and also

```rust
// filename: src/game.rs

mod red_hat_boy_states {
    impl RedHatBoyState<Idle> {
        pub fn run(self) -> RedHatBoyState<Running> {
            RedHatBoyState {
                //context: self.context.reset_frame(),
                context: self.context.reset_frame().run_right(),
                _state: Running {},
            }
        }
    }

```
We've sprouted another method on `RedHatBoyContext` called `run_right`,
which simply adds forward speed to the velocity. 
Meanwhile, we've chained a call (see!) to `run_right` in the transition. 
Don't forget to add the RUNNING_SPEED constant to the module.

#### Start Running on the right arrow.

Finally, we actually need to call this event when the `ArrowRight` button is pressed.
In `update` for `impl Game for WalkTheDog`:

```rust
    impl Game for WalkTheDog {
        ...
        fn update(&mut self, keystate: &KeyState) {
            let mut velocity = Point { x: 0, y: 0 };
            ...
            //if keystate.is_pressed("ArrowRight") { velocity.x += 3; }
            if keystate.is_pressed("ArrowRight") {
                velocity.x += 3;
                self.rhb.as_mut().unwrap().run_right();
            }   
        }

```


And the `run_right` method

```rust
// filename: src/game.rs

impl RedHatBoy {
    ...
    fn update(&mut self) {
        self.state_machine = self.state_machine.update();
    }

    fn run_right(&mut self) {
        self.state = self.state.transition(Event::Run);
    }
}
```
This will now start our RHB running, so much so that he'll run right off the screen!


We don't need to move backward in our actual game, nor stop, so we won't.


#### Delete the original code.

Now that the new and improved RHB is moving, it's time to get rid of all the
references in WalkTheDog to the 
sheet, 
the element, 
the frame ...

basically anything that isn't the RedHatBoy struct :

```rust
// filename: src/game.rs

pub struct WalkTheDog {
    rhb: Option<RedHatBoy>,
}
```

Rather than boring you with endless deletes, 
I'll simply say you can delete all the fields that aren't rhb 
and follow the compiler errors to delete the rest of the code.


Start with `image`.

```rust
pub struct WalkTheDog {
    //image: Option<HtmlImageElement>,
    sheet: Option<Sheet>,
    frame: u8,
    position: Point,
    rhb: Option<RedHatBoy>,
}

impl WalkTheDog {
    pub fn new() -> Self {
        WalkTheDog {
            //image: None,
            sheet: None,
            frame: 0,
            position: Point {x: 0, y: 0},
            rhb: None,
        }
    }
}

#[async_trait(?Send)]
impl Game for WalkTheDog {
    ...
    fn draw(&self, renderer: &Renderer) {

/*
        self.image.as_ref().map(|image| {
            renderer.draw_image(&self.image.as_ref().unwrap(),
                &Rect {  x: sprite.frame.x.into(),
                        y: sprite.frame.y.into(),
                        width: sprite.frame.w.into(),
                        height: sprite.frame.h.into(),
                },
                &Rect { x: self.position.x.into(),
                        y: self.position.y.into(),
                        width: sprite.frame.w.into(),
                        height: sprite.frame.h.into(),
                },
            );
        });
*/

        self.rhb.as_ref().unwrap().draw(renderer);
    }//^-- draw()

```

then `sheet` and so on

```rust
pub struct WalkTheDog {
    //image: Option<HtmlImageElement>,
    //sheet: Option<Sheet>,
    frame: u8,
    position: Point,
    rhb: Option<RedHatBoy>,
}

impl WalkTheDog {
    pub fn new() -> Self {
        WalkTheDog {
            //sheet: None,
            frame: 0,
            position: Point {x: 0, y: 0},
            rhb: None,
        }
    }
}

#[async_trait(?Send)]
impl Game for WalkTheDog {
    ...
    fn draw(&self, renderer: &Renderer) {
        let current_sprite = (self.frame / 3) + 1;
        let frame_name = format!("Run ({}).png", current_sprite);
        /*let sprite = self.sheet.as_ref()
                               .and_then(|sheet| sheet.frames.get(&frame_name))
                               .expect("Cell not found");
        */
        renderer.clear( &Rect {
            ...
```

When you're done, WalkTheDog becomes very short, as it should be. 
As for the arrow keys, you only need to worry about the ArrowRight key, 
and moving to the right.



