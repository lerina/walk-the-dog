## Transitioning to sliding

Transitioning from running to sliding will involve adding a new state for sliding, 
so that we see the sliding action, but also checking for when a slide is complete 
and transitioning back into the running state. 

This will mean sliding will have its own variation on the update function. 
We can start by adding sliding on the down arrow and treating it all just like running. 

We'll go through this quickly because most of it is familiar. 
Let's start by adding sliding on the down arrow in the update method of WalkTheDog:

```rust
// filename: src/game.rs

impl Game for WalkTheDog {
    fn update(&mut self, keystate: &KeyState) {
        ...
        //if keystate.is_pressed("ArrowDown") { velocity.y += 3; }
        if keystate.is_pressed("ArrowDown") {
            self.rhb.as_mut().unwrap().slide();
        }
        ...
```
It's time to follow the compiler. 
RedHatBoy doesn't have a `slide` method, so let's add that:

```rust
// filename: src/game.rs

impl RedHatBoy {
    ...
    fn run_right(&mut self) {
        self.state_machine = self.state_machine.transition(Event::Run);
    }

    fn slide(&mut self) {
        self.state_machine = self.state_machine.transition(Event::Slide);
    }
}

```

Transitioning via `Event::Slide` doesn't exist. 
There's no `Event::Slide` at all, so let's add those next:


```rust
// filename: src/game.rs

pub enum Event {
    Run,
    Slide,
}

...

impl RedHatBoyStateMachine {
    fn transition(self, event: Event) -> Self {
        match (self, event) {
            (RedHatBoyStateMachine::Idle(state), Event::Run) 
                => state.run().into(), 
            (RedHatBoyStateMachine::Running(state), Event::Slide) 
                => state.slide().into(),
            _ => self,
        }
    }
    ...
```

There's nothing new in the preceding code block. When RHB is Running, 
it can transition to `Sliding` via the `Event::Slide` event and the `slide` method, 
which doesn't exist on the `RedHatBoyState<Running>` typestate. 

This is all very similar to how we went from `Idle` to `Running`.
To continue with the compiler, we need to add a `slide` method to the
`RedHatBoyState<Running>` typestate:

```rust
// filename: src/game.rs

mod red_hat_boy_states {
    ...
    impl RedHatBoyState<Running> {
        pub fn frame_name(&self) -> &str {
            RUN_FRAME_NAME
        }        
        ...
        pub fn slide(self) -> RedHatBoyState<Sliding> {
            RedHatBoyState {
                context: self.context.reset_frame(),
                _state: Sliding {},
            }
        }
    }

```

The `slide` method on `RedHatBoyState<Running>` converts the state into
`RedHatBoyState<Sliding>`, only calling `reset_frame` on `context` to
make sure the sliding animation starts playing at frame 0. 
We also call into on the `slide` method, which needs to convert `RedHatBoyState<Sliding>` 
into a `RedHatBoyStateMachine` variant. 

That means we need to create the variant and create a `From` implementation for it, 
as shown here:


```rust
// filename: src/game.rs

#[derive(Copy, Clone)]
enum RedHatBoyStateMachine {
    Idle(RedHatBoyState<Idle>),
    Running(RedHatBoyState<Running>),
    Sliding(RedHatBoyState<Sliding>),
}

...
impl From<RedHatBoyState<Running>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Running>) -> Self {
        RedHatBoyStateMachine::Running(state)
    }
}

impl From<RedHatBoyState<Sliding>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Sliding>) -> Self {
        RedHatBoyStateMachine::Sliding(state)
    }
}
```

At this point, you'll see errors on the `frame_name`, `context`, and `update` methods
of `RedHatBoyStateMachine` because their corresponding match calls don't have
cases for the new `Sliding` variant. 

We can fix that by adding cases to those match statements, 
which will mimic the other cases:

```rust
// filename: src/game.rs

impl RedHatBoyStateMachine {
    fn transition(self, event: Event) -> Self {
    ...
    
    fn frame_name(&self) ->&str {
        match self {
            RedHatBoyStateMachine::Idle(state) => state.frame_name(),
            RedHatBoyStateMachine::Running(state) => state.frame_name(),
            RedHatBoyStateMachine::Sliding(state) => state.frame_name(),
        }
    }
    ...
    fn context(&self) ->&RedHatBoyContext{
        match self {
            ...
            RedHatBoyStateMachine::Sliding(state) => &state.context(),
        }
    }

    fn update(self) -> Self {
        match self {
            ...
            RedHatBoyStateMachine::Sliding(mut state) => {
                state.update();
                RedHatBoyStateMachine::Sliding(state)
            },
        }
    }

}//^-- impl RedHatBoyStateMachine
```
Once again, we've replaced one compiler error with another. 
There is no Sliding state, and it doesn't have the methods we assumed it would. 

We can fix that by filling it in, adding some constants for good measure:
add 
```rust
// filename: src/game.rs

mod red_hat_boy_states {
    const SLIDING_FRAMES: u8 = 14;
    const SLIDING_FRAME_NAME: &str = "Slide";
    ...

    #[derive(Copy, Clone)]
    pub struct Running;

    #[derive(Copy, Clone)]
    pub struct Sliding;
```

then   `impl RedHatBoyState<Sliding>`

```rust
// filename: src/game.rs

mod red_hat_boy_states {
    ...
    #[derive(Copy, Clone)]
    pub struct Sliding;

    ...

    impl RedHatBoyState<Running> {
    ...

    impl RedHatBoyState<Sliding> {
        pub fn frame_name(&self) -> &str {
            SLIDING_FRAME_NAME
        }

        pub fn update(&mut self) {
            self.context = self.context.update(SLIDING_FRAMES);
        }
    }

    impl<S> RedHatBoyState<S> {
    ...

}//^-- mod red_hat_boy_states
```

Now RHB can run and slide.

Stopping RHB from sliding is a little different than what we've done before. 
What we need to do is identify when the slide animation is complete, 
then transition right back into running without any user input. 

We'll start by checking whether the animation is done in the update method of the enum, 
which represents our machine, and then create a new transition from sliding back into running. 
We can do that by modifying the `RedHatBoyStateMachine` `update` method to check 
after updating in the sliding branch, as follows:

```rust
// filename: src/game.rs

impl RedHatBoyStateMachine {
    ...

    fn update(self) -> Self {
        match self {
            ...
            RedHatBoyStateMachine::Running(mut state) => {
            ...

            RedHatBoyStateMachine::Sliding(mut state) => {
                state.update(SLIDING_FRAMES);
                if state.context().frame >= SLIDING_FRAMES {
                    RedHatBoyStateMachine::Running(
                    state.stand())
                } else {
                    RedHatBoyStateMachine::Sliding(state)
                }
            }
        }
    }
```


This doesn't compile yet, because `stand` isn't defined yet and because `SLIDING_FRAMES`
is in the `red_hat_boy_states` module. You might think that we can make 
`SLIDING_FRAMES` public and define a `stand` method, or we could move `SLIDING_FRAMES`
into the game module. 
These will both work but I think it's time to look 
a little more holistically at our update method.

Every arm of the match statement updates the current state 
and then returns a new state. 
In the case of `Running` and `Idle`, it was always the same state, 
but in the case of `Sliding`, sometimes it's the `Running` state. 

It turns out `update` is a transition, just one
that sometimes transitions to the state it started from.

If we wanted to be strict about it, we could say that `Sliding` transitions to an Updating
state when it gets an `Update` event, then it can transition back to `Sliding` or `Running`. 

This is a case where the state exists, at least conceptually, 
but we don't actually have to create it in our code.

`update` on the Sliding state is really best modeled as a transition 
because it's a method that ultimately returns a state. 
Come to think of it, that's exactly what the other arms in the `update` method are too! 
Yes, they don't ever transition to another state, but each branch calls `update` 
and then returns a state. 

So, before we add `Sliding` to the `update` method, 
let's refactor to make `update` a transition for both of the other states.

Since we're using Compiler-Driven Development, 
we'll change the update method to work as if `update` is already a transition:

```rust
// filename: src/game.rs

pub enum Event {
    Run,
    Slide,
    Update,
}

```

now match must handle the Update Variant

```rust
// filename: src/game.rs

impl RedHatBoyStateMachine {
    fn transition(self, event: Event) -> Self {
        match (self, event) {
            //Run
            (RedHatBoyStateMachine::Idle(state), Event::Run) => state.run().into(), 
            
            //Slide            
            (RedHatBoyStateMachine::Running(state), Event::Slide) => state.slide().into(),

            //Update
            (RedHatBoyStateMachine::Idle(state), Event::Update) => state.update().into(),
            (RedHatBoyStateMachine::Running(state), Event::Update) => state.update().into(),

            _ => self,
        }
    }

    // new update replacing the old one
    fn update(self) -> Self {
        self.transition(Event::Update)
    }

```

With these changes, we've turned `Update` into `Event` and added two more arms 
to match in the transition method. Both of those arms work the same way as the
other transitions: 
they call a method on the typestate and then convert the state into the
`RedHatBoyStateMachine` enum with the `From` trait (that's the `.into()` ).

For now, there's no way to convert from the (), or Unit, to a value 
of the `RedHatBoyStateMachine` type. 

That's not what we want to fix; 
we want to make both of the update calls on the states return new states. 
Those changes are next:

```rust
// filename: src/game.rs

mod red_hat_boy_states {
    impl RedHatBoyState<Idle> {
        ...
        /*
        pub fn update(&mut self) {
            self.context = self.context.update(IDLE_FRAMES);
        }
        */

        pub fn update(mut self) -> Self { //NOTE mut not &mut
            self.context = self.context.update(IDLE_FRAMES);
                
            self
        }

        ...
    }//^-- impl RedHatBoyState<Idle>


    impl RedHatBoyState<Running> {
        ...
        pub fn update(mut self) -> Self {
            self.context = self.context.update(RUNNING_FRAMES);

            self
        }
    }//^-- impl RedHatBoyState<Running>



```

The changes are small but important. 
The update method for `RedHatBoyState<Idle>` and `RedHatBoyState<Running>` both return `Self` now, because even though the state doesn't change, these are still typestate methods that return a new state. 

They also take `mut self` now instead of `&mut self`. 
You can't return self if you mutably borrow it, so this method stopped compiling. 

More importantly, this means these methods don't make unnecessary copies. 
They take ownership of self when called, and then return it. 
So, if you're worried about an optimization problem because of extra copies, 
you don't have to be.

Now, we're down to one compiler error, which we've seen before:

```sh
the trait 'From<red_hat_boy_states::RedHatBoyState<red_hat_boy_
states::Idle>>' is not implemented for 'RedHatBoyStateMachine'
```


We didn't implement a conversion from the `Idle` state back to the
`RedHatBoyStateMachine` enum. 

That's similar to the other ones we wrote,
implementing `From<RedHatBoyState<Idle>>` , as shown here:

```rust
// filename: src/game.rs

...
impl From<RedHatBoyState<Sliding>> for RedHatBoyStateMachine {
...

impl From<RedHatBoyState<Idle>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Idle>) -> Self {
        RedHatBoyStateMachine::Idle(state)
    }
}
```

Remember that these implementations of the `From` trait are not 
in the `red_hat_boy_states` module. 

The `red_hat_boy_states` module knows about the **individual states**
but does not know about `RedHatBoyStateMachine`. 
That's not its job.



