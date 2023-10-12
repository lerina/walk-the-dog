## Transitioning to sliding and back again

Part of the reason we used the typestate pattern for our individual states 
is so that we get compiler errors when we make a mistake. 

For instance, if we call run when we are in the `Running` state, 
it won't even compile because there is no such method. 

There is one place this doesn't hold, the transition method on the `RedHatBoyStateMachine` enum .

If you call `transition` with a `RedHatBoyStateMachine` variant 
and an `Event` variant pair that don't have a match, it returns Self.

That's why our RHB is sitting down. 

He transitions to `Sliding` and then stops updating, staying in the same state forever. 

We'll fix that by adding the match for the `Update` event
and then, you guessed it, follow the compiler to implement the **sliding animation**.

This starts by adding the match to the transition method, as shown here:

```rust
// src/game.rs

impl RedHatBoyStateMachine {
    fn transition(self, event: Event) -> Self {
        match (self, event) {
        ...
        (RedHatBoyStateMachine::Sliding(state), Event::Update) => state.update().into(),
        _ => self,
        }
    }

```

This match is just like the others; we match on Sliding and Update and call update .
Just like before, we'll get an error: 

```sh
the trait 'From<()>' is not implemented for 'RedHatBoyStateMachine'
```

The Sliding state still has an update method that doesn't return a state. 
That's not going to work with our current setup, but it's not as simple 
as making the update method return Self, as on the other two states.

Remember, there are two possible states that can come from the update method
on Sliding: `Sliding` and `Running`. 
How is that going to work with our current setup? 
What we'll need to do is have update return 
an `SlidingEndState` enum that can be either `Sliding` or `Running`, 
and then we'll implement a `From` trait that will convert that 
into the appropriate variant of `RedHatBoyStateMachine`.

We can modify the update method on `RedHatBoyState<Sliding>` 
to work like the one we proposed at the beginning of this section:

```rust
// src/game.rs

mod red_hat_boy_states {
    ...
    impl RedHatBoyState<Sliding> {
        ...
    /*
        pub fn update(&mut self) {
            self.context = self.context.update(SLIDING_FRAMES);
        }
    */

        pub fn update(mut self) -> SlidingEndState {
            self.context = self.context.update(SLIDING_FRAMES);
            if self.context.frame >= SLIDING_FRAMES {
                SlidingEndState::Complete(self.stand())
            } else {
                SlidingEndState::Sliding(self)
            }
        }


```

We've taken the code that we originally considered putting in the
`RedHatBoyStateMachine` `update` method and moved it into the `update` method
of `RedHatBoyState<Sliding>`. 

This makes sense conceptually; 

> the state should know how it behaves. 

On every update, we update context, and then check whether the animation is complete, 
with `if self.context.frame >= SLIDING_FRAMES`. 

If the animation is complete, we return one variant of this new enum that doesn't exist yet:
`SlidingState`. 

The `SlidingState` variant can either be `Complete` or `Sliding`.


Following the compiler yet again, we have two obvious problems: 
there is no `stand` method and there is no `SlidingEndState` enum. 

We can handle both of these right here, next to the code we just wrote, as shown:

```rust
// src/game.rs

impl RedHatBoyState<Sliding> {
    ...
    pub fn stand(self) -> RedHatBoyState<Running> {
        RedHatBoyState {
            context: self.context.reset_frame(),
            _state: Running,
        }
    }
    ...
}//^-- impl RedHatBoyState<Sliding>

pub enum SlidingEndState {
    Complete(RedHatBoyState<Running>),
    Sliding(RedHatBoyState<Sliding>),
}

```

The only side effect of the transition to Running is that we call `reset_frame` again on
context. 
Remember this has to be done on every transition, otherwise, the program can
try to animate the new state with frame, which isn't valid and will crash. 

So, we'll reset the frame back to 0 on every transition.
`context: self.context.reset_frame(),`


This leaves us with a compiler error to fix once again. 
This time, it's the following:

```sh
the trait 'From<SlidingEndState>' is not implemented for 'RedHatBoyStateMachine'
```

Pay close attention to that source trait. 
It's not coming from one of the states but from the intermediate `SlidingEndState`. 
We'll solve it the same way as before, with a `From` trait, 
but we'll need to use a match statement to pull it out of the enum:

```rust
// src/game.rs

impl From<SlidingEndState> for RedHatBoyStateMachine {
    fn from(end_state: SlidingEndState) -> Self {
        match end_state {
            SlidingEndState::Complete(running_state) => running_state.into(),
            SlidingEndState::Sliding(sliding_state) => sliding_state.into(),
        }
    }
}

mod red_hat_boy_states {
    ...
```

Here, we match on `end_state` to get the actual State out of enum, 
and then call `into` on that state again to get to `RedHatBoyStateMachine`. 

A little boilerplate, but it makes it easier to do the conversion.

And now we have it! Run the game now and you'll see RHB take a short slide and pop
back up again to the running state. 

Now that we've added three animations, it's time to deal with these ugly lines 
in the WalkTheDog implementation: 
- `self.rhb.as_mut()`.
- `unwrap().slide()`.

We treat `rhb` as an Option type, not because it's ever really going to be None, 
but because **we don't have it yet** before the `WalkTheDog` struct is initialized. 

After `WalkTheDog` is initialized, `rhb` can never be `None` again because 
the state of the system has changed. 
Fortunately, we now have a tool for dealing with that, the good old state machine!


