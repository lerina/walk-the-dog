## Adding a UI

### Show the button on game over

We can show and hide the button in the Game update method by checking on each
frame if the game is over and if the button is present, ensuring that we only show or
hide it once, and that would probably work, but I think you can sense the spaghetti code
beginning to form if we do that. In general, it's best to avoid too much conditional logic
in update , as it gets confusing and allows for logic bugs. Instead, we can think of every
conditional check that looks like if (state_is_true) as two different states of the
system. So, if the new game button is shown, that's one game state, and if it isn't, that's
another game state. You know what that means – it's time for a state machine.

#### A state machine review

In Chapter 4, Managing Animations with State Machines, we converted RHB to a state
machine in order to make it change animations on events easily and, more importantly,
correctly. For instance, when we wanted `RHB` to jump, we went from `Running` to
`Jumping` via a `typestate` method, only changing the state one time and changing the
`velocity` and playing the `sound` one time. That code is reproduced here for clarity:



```rust
// src/game.rs

impl RedHatBoyState<Running> {
    ...
    pub fn jump(self) -> RedHatBoyState<Jumping> {
        RedHatBoyState {
            context: self
                .context
                .reset_frame()
                .set_vertical_velocity(JUMP_SPEED)
                .play_jump_sound(),
            _state: Jumping {},
        }
    }

```

The `typestates` work great, but they are also noisy if we don't need that kind of
functionality. That's why in that same chapter, we chose to model our game itself as a
simple `enum`, like so:

```rust
// src/game.rs

pub enum WalkTheDog {
    Loading,
    Loaded(Walk),
}

```

This is going to change significantly because we now have a problem that necessitates a
state machine. 
When `RHB` is knocked out, the game is over, and the new game button should appear. 
That's a side effect that needs to happen once, on a change of state, 
the perfect use case for our state machine. 

Unfortunately, refactoring to a state machine is going to require a not insignificant amount 
of code because our current method for implementing state machines is elegant but a little noisy. 

In addition, there's actually two state machines at work here, which is not obvious at first. 

The first is the one we see at the beginning, moving from `Loading` to `Loaded`, 
which you can think of as when you don't have Walk and when you do. 

The second is the state machine of `Walk` itself, which moves from `Ready` to `Walking` to `GameOver`. 

You can visualize it like this:


![Nested state machines](./readme_pix/Nested_state_machines.png)


As you can see, we have two state machines here, one going from `Loading` to `Loaded`
and the other representing the three game states of `Ready`, `Walking`, and `GameOver`.

There is a third state machine, not pictured, the famous `RedHatBoyStateMachine`
that manages the `RedHatBoy` animations. A couple of the states pictured mimic
the states in `RedHatBoyStateMachine`, where `Idle` is `Ready` and `Walking`
is `Running`, so there is a temptation to move `RedHatBoyStateMachine` into
`WalkTheDogStateMachine`. 

This could work, but remember that `Walk` doesn't have a "jumping" state and so, by doing that, you'll need to start checking a Boolean, and the modeling starts to break down. 

It's best to accept the similarity because the game is heavily dependent on what `RHB` is doing, 
but treat `RedHatBoyStateMachine` as having more fine-grained states. 

What does work is turning `Loading` and `Loaded` into `Option`. 
Specifically, we'll model our game like so:

```
struct WalkTheDogGame {
    machine: Option<WalkTheDogStateMachine>
}
```
This code isn't meant to be written anywhere yet; it's just here for clarity. 

There's a big advantage to using `Option` here, and it has to do with the way our `update` function
works. 
For clarity, I'm going to reproduce a section of our `game loop` here:


```rust
// src/game.rs

let mut keystate = KeyState::new();
*g.borrow_mut() = Some(browser::create_raf_closure(move |perf: f64| {
                    process_input(&mut keystate, &mut keyevent_receiver);
                    game_loop.accumulated_delta += (perf – game_loop.last_frame) as f32;
                    
                    while game_loop.accumulated_delta > FRAME_SIZE {
                        game.update(&keystate);
                        game_loop.accumulated_delta -= FRAME_SIZE;
                    }

```

The key part here is the `game.update` line, which performs a mutable borrow on the
`game` object instead of moving it into `update`. 
This is because once `game` is owned by `FnMut`, it can't be moved out. 
Trying to actually leads to this compiler error:

```
error[E0507]: cannot move out of `*game`, as `game` is a
captured variable in an `FnMut` closure
```

Mutable borrows such as this are tricky because they can make it more challenging
to navigate the borrow checker as you proceed down the call stack. In this case, it
becomes a problem if we try to implement another state machine in the same manner as
`RedHatBoyStateMachine`. 

In our state machine implementation, each `typestate` method consumes the machine and returns a new one. 

Now, let's imagine that we are modeling the entire game as `enum`, like so:

```
enum WalkTheDogGame {
    Loading,
    Loaded(Walk),
    Walking(Walk),
    GameOver(Walk)
}
```


In order to make this work with the mutable borrow in `update`, we would have to `clone`
the entire game on every state change because the from function couldn't take ownership
of it. 
In other words, the closure in our `game.update` function lends game to the `update` function. 
This can't turn around and give it to the from function – it doesn't own
it! Doing so requires cloning the entire game, potentially on every frame!

Modeling the game as holding an optional `WalkTheDogStateMachine` has two advantages:

• We can call take on Option to get ownership of the state machine.
• The type reflects that the state machine isn't available until the game is loaded.


Note::
    
    There are, naturally, many ways to model our game type, and some of them are
    going to be better than the one we'll choose here. However, before you start
    trying to do a "simpler" version of this type, let me warn you that I tried several
    different variations on this solution and ultimately found using Option to be
    the most straightforward choice. Several other implementations either ended
    with complex borrowing or unnecessary cloning. Be wary, but also be brave.
    You may find a better way than I did!
    

Before we dig into the actual implementation, which is fairly long, let's go over the design
we're implementing.

![Before](./readme_pix/before.png)


It's pretty simple, but it doesn't do all that we need it to. Now, let's redesign the state
machine.

![After](./readme_pix/after.png)


Yeah, that's a lot more code, and it doesn't even reflect the details of the implementation,
or the `From traits` we write to make it easy to convert between the `enum values` and
`structs`. Writing some macros to handle state machine boilerplate is out of the scope
of this book, but it's not a bad idea. You might wonder why every state holds its own
`Walk` instance when every single state has it, and that's because we're going to change
`Walk` on the transitions and the individual states don't have easy access to the parent
`WalkTheDogState` container data. However, where possible, we'll move common data
out of `Walk` and into `WalkTheDogState`.


Tip::
    
    This section has a lot of code, and the snippets tend to only show a few lines
    at a time so that it's not too much to process. However, as you're following
    along, you may wish to reorganize the code to be easier to find. For instance,
    I prefer to work top-down in the game module, with constants at the top
    followed by the "biggest" struct , which is WalkTheDog in this case,
    followed by any code it depends on, so that the call stack flows down the
    page. This is how https://github.com/PacktPublishing/
    Game-Development-with-Rust-and-WebAssembly/tree/
    chapter_8 is organized. You're also welcome to start breaking this up into
    more files. I won't, to make it easier to explain in book form.
    

#### Redesigning to a state machine

In a true refactoring, we would make sure the game was in a running state after each
change, but our changes are going to cause cascading compiler errors, meaning we're
going to be broken for a while, so this change isn't truly a refactoring but more of
a redesign. 

When you make this kind of change, you should absolutely get to a compiling
state as quickly as possible and stay there as long as possible, but while I did that when
writing this chapter, I'm not going to put you through all the intermediate steps. 
We'll move forward as if we know in advance that our design is going to work because we do
this time, but don't try this at home. 

If you're a regular Git user, now is an excellent time to create a branch, just in case.

We'll start by replacing this code in the game module:

```
pub enum WalkTheDog {
    Loading,
    Loaded(Walk),
}
```

We'll replace it with the following:

```rust
// src/game.rs

pub struct WalkTheDog {
    machine: Option<WalkTheDogStateMachine>,
}

```

This will cause compiler errors all over the place. 
This is the section where we'll take the shortcut of letting the compiler 
be broken temporarily while we implement the state machine, 
if only to make sure this chapter isn't a thousand pages long. 

So, if you're uncomfortable working with a broken code base for a long time, that's good – just 
trust that I was brilliant and got this all right on the first try. Pretend – it'll be okay.

Since we're psychic and know exactly how this design is going to work out, we can go and 
push ahead, knowing that eventually, everything will come together without errors. 

This first change is exactly what we discussed earlier – `enum WalkTheDog`
becomes a `struct` holding its machine instance, which is an optional field. 

Currently, `WalkTheDogStateMachine` doesn't exist, so we'll add that next, like so:


```rust
// src/game.rs

enum WalkTheDogStateMachine {
    Ready(WalkTheDogState<Ready>),
    Walking(WalkTheDogState<Walking>),
    GameOver(WalkTheDogState<GameOver>),
}

```

When we implement a state machine in Rust, we need enum as a container for states
so that `WalkTheDog` doesn't need to be a generic struct. 

We've moved the compiler errors down because there is no `WalkTheDogState` and no states defined. 

Let's do that next:


```rust
// src/game.rs

struct WalkTheDogState<T> {
    _state: T,
    walk: Walk,
}

struct Ready;
struct Walking;
struct GameOver;

```

Right now, the various `typestates`, `Ready`, `Walking`, and `GameOver`, 
don't store any data. This will change a little as we go on, 
but all of the `typestates` have `Walk` so that they can be saved 
in the common `WalkTheDogState` struct. 

Now that we've created the state machine, 
we need to look at where the old version of `WalkTheDog` was used. 

The first is in the small `impl` block for `WalkTheDog`, 
in the old code where we created enum , like so:

```
impl WalkTheDog {
    pub fn new() -> Self {
        WalkTheDog::Loading {}
    }
}
```

That's not going to work, and it's not compiling, so instead, let's replace it with an empty
`WalkTheDog` instance, as shown here:


```rust
// src/game.rs

impl WalkTheDog {
    pub fn new() -> Self {
        WalkTheDog { machine: None }
    }
}

```

This change replaces the old, and not used, enum with `machine` set to `None`. 
You can now think of `None` as the `Loading state`, 
and when a machine is present, you are `Loaded`. 

Speaking of loading, the next logical place to make changes is in the Game implementation for `WalkTheDog`. 
Looking at the `initialize` function that we've been in so many times, you'll see a compiler error here:

```
#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        match self {
            WalkTheDog::Loading => {    
```

The `match self` line is not going to work anymore because `self` isn't `enum`. 
What we need to do instead is match machine, and if it's `None`, then load the new machine,
and if it's present, then use `Err` in the same way we do now because initialize was somehow called twice. 

We can start by replacing both halves of the match statement, so the match should start as follows:

```rust
// src/game.rs

#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        match self.machine {
            None => {

```

Look closely to see that we now match on `self.machine`, and we match against
`None`. Before we dig into the `None` match arm, let's quickly change the match on
`WalkTheDog::Loaded(_)`, as shown here:

```rust
// src/game.rs

#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        match self.machine {
            None => {
                ...
            },
            ...
            //WalkTheDog::Loaded(_) => Err(anyhow!("Error: Game is already initialized")),
            Some(_) => Err(anyhow!("Error: Game is already initialized!")),

```

This simply changes `WalkTheDog::Loaded` to `Some`, using the same error message.

Tip::

    In order to get clearer error messages, you can `#[derive(Debug)]` on the
    `WalkTheDog` struct. Doing that has cascading effects because everything it
    depends on also has to `#[derive(Debug)]`, so we won't do that here, but
    it's a good idea, especially if you're running into issues here.
    

Now that both halves of the match properly match an `Option` type, we need to modify
`initialization` to return the proper type. At the bottom of the `None` branch, you will
want to create a state machine like the one shown here, right before returning the value:

```rust
// src/game.rs


#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        match self.machine {
            //WalkTheDog::Loading => {
            None => {
                ...
                let timeline = rightmost(&starting_obstacles);

                let machine = WalkTheDogStateMachine::Ready(WalkTheDogState {
                        _state: Ready,
                        walk: Walk {
                            boy: rhb,
                            backgrounds: [
                                Image::new(background.clone(),
                                Point { x: 0, y: 0 }),
                                Image::new(
                                    background,
                                    Point {
                                        x: background_width,
                                        y: 0,
                                    },
                                ),
                            ],
                            obstacles: starting_obstacles,
                            obstacle_sheet: sprite_sheet,
                            stone,
                            timeline,
                        },
            	});
                ...
```

This is very similar to the code before; the construction of `Walk` is unchanged, but
it's obscured by all the state machine noise. We are binding the machine variable to
`WalkTheDogStateMachine::Ready` with the initialized `WalkTheDogState` instance, 
which, in turn, sets its internal `_state` value to `Ready`, and with the state getting to have `Walk`. 

It's noisy, and after we get this file back to compiling, we'll do true refactoring to make that line 
a little cleaner, but put a pin in that for now.

Now, we made it so that initialize returns a new `Result<Box<dyn Game>>`
a while back, so we'll need to return a new `Game` instance next. 

So, right after adding machine, add the following:


```rust
// src/game.rs

                ...
                //Ok(Box::new(WalkTheDog::Loaded(walk)))
                Ok(Box::new(WalkTheDog { machine: Some(machine),}))

            },
            //WalkTheDog::Loaded(_) => Err(anyhow!("Error: Game is already initialized")),
            Some(_) => Err(anyhow!("Error: Game is already initialized!")),
        }
    }//^-- async fn initialize
```

Note::
    
    Given that initialize takes self and doesn't really use it, it's debatable
    whether it should be in the Game trait. Creating a separate trait, such as
    `Initializer`, will require a lot of modifications and is an exercise for
    the reader.
    


This takes care of making sure initialize returns a game with a machine in the
right state. We have two more big trait methods, update and draw , to take care of, and
update is filled with compiler errors, so let's do that next.


#### Spreading update into the state machine

The `update` function is filled with compiler errors, is the core of the game's behavior, and
has an additional challenge. Instinctively, you might think you can modify the beginning
of the function like so:

```
impl Game for WalkTheDog {
    ...
    fn update(&mut self, keystate: &KeyState) {
        if let Some(machine) = self.machine {
            ...
```
The `if let Some(machine) = self.machine` line will eventually fail to compile
with the error:

```
error[E0507]: cannot move out of `self.machine.0` which is
behind a mutable reference
...
```


Now, you may try, as I did, to fix this by changing the line 
to `if let Some(machine) = &mut self.machine`. 
This will work until you try to implement a transition on `WalkTheDogState`. 

Because you have a borrowed machine, you'll also have a borrowed state 
when you later match on the state, as with the following example:

```
impl Game for WalkTheDog {
    ...
    fn update(&mut self, keystate: &KeyState)
        if let Some(machine) = &mut self.machine {
        match machine {
            WalkTheDogStateMachine::Ready(state) => {
```

Here, the state value is borrowed, unlike in most other cases where the match arms take
ownership of the value, and it's not instantly obvious. It will be if we write a transition
from `Ready` to `Walking`. In order to write `state._state.run_right()` and get to
`Walking`, your transition will need to look like this in order to compile:

```
impl WalkTheDogState<Ready> {
    fn start_running(&mut self) -> WalkTheDogState<Walking> {
        self.run_right();
        WalkTheDogState {
            _state: Walking,
            walk: self.walk,
        }
    }
}
```

Note that we are transitioning from `&mut WalkTheDogState<Ready>>` to
`WalkTheDogState<Walking>`, which is an odd conversion and a hint that this
is wrong. 

What you can't see is that this code won't compile. 

Returning the new `WalkTheDogState` with `walk` is a move that we cannot do 
because state is borrowed. 
The `start_running` method doesn't own `state`, so it can't take ownership
of `state.walk` and, therefore, can't return the new instance. 

The workaround for this is to `clone` the entire `Walk` each time we transition, 
but there's no need for that inefficiency. 

We can, instead, take ownership of machine all the way back up in the `Game` implementation, 
through the aptly named `take` function. 

Instead of using a mutable borrow on the machine, we'll call `take`, 
as shown here:


```rust
// src/game.rs

impl Game for WalkTheDog {
    ...
    fn update(&mut self, keystate: &KeyState) {
        if let Some(machine) = self.machine.take() {
            
```

This is the same code as earlier, but instead, we call the `take` method on
`Option<WalkTheDogStateMachine>`. 

This replaces the state `machine` in `self` with `None` and binds the existing `machine` to the variable 
in `if let Some(machine)`.

Now, inside that scope, we have complete ownership of machine and can do whatever
we like to it, before eventually calling `replace` on the state machine in self to move
it back in at the end of this function. 

It's a little awkward, but it gets around mutable borrows. 
It also introduces a potential error in that when control exits the update function, 
machine could still be set to `None`, effectively halting the game by accident.
In order to prevent that from happening, before we continue updating this function, we'll
add `assert` just outside the if let statement, as shown here:


```rust
// src/game.rs


impl Game for WalkTheDog {
    ...
    fn update(&mut self, keystate: &KeyState) {
        if let Some(machine) = self.machine.take() {
            ...
        }
        assert!(self.machine.is_some());

```

Unfortunately, this is a runtime error, not a compile-time one, 
but it's going to let us know right away whether we mess up the next section. 

This assert may be overkill, because we are going to dramatically reduce the amount of code 
inside the if let block; in fact, it will be just one line. 

First, we'll add a call to a non-existent function called `update` on our `state machine`, as follows:



```rust
// src/game.rs

impl Game for WalkTheDog {
    ...
    fn update(&mut self, keystate: &KeyState) {
        if let Some(machine) = self.machine.take() {
            self.machine.replace(machine.update(keystate));

            if keystate.is_pressed("ArrowRight") {
            ...
```

Immediately after `if let Some(machine)`, 
add the `self.machine.replace(machine.update(keystate))` line. 
All the code below `replace` in the `if let` block is going to become part 
of various `update` functions in the implemented states, 
so what you'll want to do is either cut and paste that code to some place you can get
it, or just comment it out. 

Next, we'll create `impl` on `WalkTheDogStateMachine` with this new `update` method, 
which will return the new state. 
An empty version of that will look like this:

```rust
// src/game.rs

impl WalkTheDogStateMachine {
    fn update(self, keystate: &KeyState) -> Self {
    }
}

```

Now, you can call that from the the `update` method in `Game`, which looks like this:

```rust
// src/game.rs

impl Game for WalkTheDog {
    ...
    fn update(&mut self, keystate: &KeyState) {
        if let Some(machine) = self.machine.take() {
            self.machine.replace(machine.update(keystate));
        }
        assert!(self.machine.is_some());
    }
    ...
```

The `update` method in `WalkTheDogStateMachine` is a little empty, and we should
probably put some code in it. 
We could call `match self` in the `update`, and then write the behavior 
for each state in this `update` function, calling things such as 
`state._state.walk.boy.run_right()` , which would work but it is hideous. 

Instead, we'll match on `self` and then delegate to the individual `state` types. 
This will result in a pretty redundant match statement, as shown here:

```rust
// src/game.rs


impl WalkTheDogStateMachine {
    fn update(self, keystate: &KeyState) -> Self {
        match self {
            WalkTheDogStateMachine::Ready(state) => state.update(keystate).into(),
            WalkTheDogStateMachine::Walking(state) => state.update(keystate).into(),
            WalkTheDogStateMachine::GameOver(state) => state.update().into(),
        }
    }
}
```
We saw a variation of this pattern before in `RedHatBoyStateMachine`, where we have
to match on each variant of `enum` in order to delegate to the state, and unfortunately,
there's not a great way around it. 
Fortunately, it's small. 
This little match statement won't compile because none of the `typestates` types have an `update` method. 
In fact, there are no implementations for the `typestates` at all. 

Let's continue our delegation by creating placeholder implementations for all three of them, 
as shown here:

```rust
// src/game.rs

impl WalkTheDogState<Ready> {
    fn update(self, keystate: &KeyState) -> WalkTheDogState<Ready> {
        self
    }
}

impl WalkTheDogState<Walking> {
    fn update(self, keystate: &KeyState) -> WalkTheDogState<Walking> {
        self
    }
}

impl WalkTheDogState<GameOver> {
    fn update(self) -> WalkTheDogState<GameOver> {
        self
    }
}

```

It's worth refreshing our memory on how `typestates` work. 
A `typestate` is a structure that is `generic` over a `state`. 

So `WalkTheDogState<T>` is the structure, and we implement transitions 
between `states` by adding methods to implementations of `WalkTheDogState<T>`, 
where `T` is one of the concrete `states`. 

These placeholders all just return `self`, so update isn't doing anything yet. 

Look closely and you'll notice that `GameOver` doesn't take `KeyState` because it won't need it.


The `update` method on `WalkTheDogStateMachine` tries to use into to convert each `typestate` 
back into `enum`, but we haven't written those yet. 
Recalling Chapter 4, Managing Animations with `State Machines`, again, 
we need to implement `From` to convert back from the various `states` to the `enum` type. 

These are implemented here:


```rust
// src/game.rs


impl From<WalkTheDogState<Ready>> for WalkTheDogStateMachine {
    fn from(state: WalkTheDogState<Ready>) -> Self {
        WalkTheDogStateMachine::Ready(state)
    }
}
impl From<WalkTheDogState<Walking>> for WalkTheDogStateMachine {
    fn from(state: WalkTheDogState<Walking>) -> Self {
        WalkTheDogStateMachine::Walking(state)
    }
}
impl From<WalkTheDogState<GameOver>> for WalkTheDogStateMachine {
    fn from(state: WalkTheDogState<GameOver>) -> Self {
        WalkTheDogStateMachine::GameOver(state)
    }
}
```

This is boilerplate just to get things started, but it demonstrates how each of these works.
The `update` method on `WalkTheDogStateMachine` uses match to get the `state` value on each variant. 

Then, the `update` method is called on the various `typestates`. 
Each `update` method returns the `state` it transitions into, although 
right now, they all return `self`. 

Finally, back in the `update` method on `WalkTheDogStateMachine`s, 
we call `into` to convert the `typestate` back into an `enum` variant.

Note::

    You might remember that for `RedHatBoyStateMachine`, we used a
    `transition` function and an `Event` enum to advance the `state machine`. 
    The new `WalkTheDogStateMachine` enum has fewer events, so additional
    complexity isn't necessary.
    

It's time to think about what each state should actually do. 
Previously, every one of these states was kind of shoved together 
in the `Game` `update` method – for instance, the following old code:

```
impl Game for WalkTheDog {
    ...
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
    ...
```

In the old system, if the game was Loaded , then boy could `run_right` if you pressed
the ArrowRight button and could `jump` if you pressed Space.
 
This worked fine, but it's worth noting the following:

• The `run_right` function does nothing if `RHB` is already running.
• The `jump` and `slide` functions do nothing if `RHB` isn't running.

We handle this quite well in our `RedHatBoyStateMachine`, and will continue to do so, 
but what this reveals is that once `RHB` starts moving to the right, we don't really
care if the player has pushed the `ArrowRight` button, and we don't really care if they
push it again. 
Similarly, if the player hasn't pressed `ArrowRight`, there's no real reason
to check whether they pressed `Space` or `ArrowDown`. 
This all fits well with our new `WalkTheDogStateMachine`. 
When the `game` is `Ready`, we'll check whether the user has hit `ArrowRight` 
and transition the `state`. Otherwise, we'll just stay in the same state.

We can modify `WalkTheDogState<Ready>` to reflect this new reality. The first change
to the function will be to do that check, as shown here:


```rust
// src/game.rs

impl WalkTheDogState<Ready> {
    fn update(self, keystate: &KeyState) -> ReadyEndState {
        if keystate.is_pressed("ArrowRight") {
            ReadyEndState::Complete(self.start_running())
        } else {
            ReadyEndState::Continue(self)
        }
    }
}

```

There's one type and one method that doesn't exist, so this code does not compile yet.

The transition of `start_running` doesn't exist yet, although we discussed writing
something like it. 

We also don't have the `ReadyEndState` type. 

Let's address that second one first.

We used this pattern earlier for any `typestate` method that can return more than
one `state`, such as the `update` method on `Jumping` or `Sliding`. 

We create a new `enum` that can represent either of the return states. 

In the case of the `update` method for `WalkTheDogState<Ready>`, 
the game can either still be `Ready` at the end of an
`update( ReadyEndState::Continue )` or 
be done and transitioning to `Walking( ReadyEndState::Complete )`.

Let's start by implementing the `From trait` 
to convert from `ReadyEndState` to `WalkTheDogStateMachine`:

```rust
// src/game.rs


enum ReadyEndState {
    Complete(WalkTheDogState<Walking>),
    Continue(WalkTheDogState<Ready>),
}

impl From<ReadyEndState> for WalkTheDogStateMachine {
    fn from(state: ReadyEndState) -> Self {
        match state {
            ReadyEndState::Complete(walking) => walking.into(),
            ReadyEndState::Continue(ready) => ready.into(),
        }
    }
}
```

This is some boilerplate that you've seen before. 
We have two states for `ReadyEndState` because there are two states 
that the `WalkTheDogState<Ready>` `update` method can end in. 
In order to get from `ReadyEndState` to `WalkTheDogStateMachine`,
we create a `From` trait and match on both variants of `ReadyEndState` 
and extract their fields from them. 
Those are both `typestates`, `WalkTheDogState<Ready>` and
`WalkTheDogState<Walking>`, respectively, so we use their into methods to
convert them into the `WalkTheDogStateMachine` type. 

Those traits were already written earlier.

The call to `self.start_running` is still not going to work because we haven't written it yet! 
What happens when the player hits `ArrowRight`? `RedHatBoy` starts walking!

Remember that to transition from one state to another, we write a `typestate` method
named after the transition, which looks like so:


```rust
// src/game.rs

impl WalkTheDogState<Ready> {
    ...
    fn start_running(mut self) -> WalkTheDogState<Walking> {
        self.run_right();

        WalkTheDogState {
            _state: Walking,
            walk: self.walk,
        }
    }

```

Let's refresh our memory on these. 

Every state transition is written as a method on the various `typestates` – in this case, 
`WalkTheDogState<Ready>`, where the source state is `self` and the return value is the destination state. 
Here, we transition from `Ready` to `Walking` by writing a method called `start_running`.


The actual implementation isn't doing much. We start by calling `self.run_right`, which doesn't exist yet, 
so we have to write it. After sending `RHB` running, we transition into the `Walking` state by returning a new `WalkTheDogState` instance with `_state` of `Walking`. 
Take a close look at the function signature for `start_running` and you'll notice it takes `mut state`. 
This means **taking exclusive ownership** over `self`, which we can do because we have complete ownership of everything in the state. 
That is one of the reasons we created `Option<WalkTheDogStateMachine>` originally! 

However, it's not obvious why we take `mut state` here instead of `state`, in part because 
`run_right` doesn't exist. 

When we add our new delegation method, that should become clear, so let's do that 
right now with the following code:



```rust
// src/game.rs

impl WalkTheDogState<Ready> {
    ...
    fn run_right(&mut self) {
        self.walk.boy.run_right();
    }
}

```

This function on `WalkTheDogState<Ready>` calls `run_right` on `boy` through its `walk` field. 
The `run_right` method on `boy` requires a mutable borrow, and that's why we require a mutable borrow 
on the previous delegate. It's also why we needed to take `mut state` in the `start_running()` method earlier. 

You can't mutably borrow something that isn't mutable in the first place.

In order to keep the code clean, we're doing a little more delegation now than we were earlier. 
This makes our methods smaller and easier to understand, but the trade-off is that the behavior 
will be spread across multiple places. 
I think in the end, this will make our code easier to think about,  because we won't have to consider 
too much code at any one time, so the trade-off is worth it. 

We'll have to be careful that we don't lose track of any of our original code 
as we break it up into chunks and spread it around.


#### Re-implementing draw

Now, we've removed all the compiler errors in the original update method, in part
by removing a large chunk of its functionality, and we can continue by updating the
Walking state to ensure that it's working, but I believe that's a long time without any
meaningful feedback from the game. 
After all, at this point, the game doesn't compile and doesn't draw. 
How do we know anything is working? 

Let's instead take a moment and update the Game `draw` method so that we can actually 
get the code to compile again and see how it's working.

The draw method will start by taking a page from the update method and replacing
its current implementation with a delegation to WalkTheDogStateMachine , as shown here:

```rust
// src/game.rs

impl Game for WalkTheDog {
    ...
    fn draw(&self, renderer: &Renderer) {
        renderer.clear(&Rect::new(Point { x: 0, y: 0 }, 600, 600));

        if let Some(machine) = &self.machine {
            machine.draw(renderer);
        }
    }
}
```

There are two things that are a little different from the changes we made to update. 

The first is that we only borrow `self.machine` because we don't need mutable access. 

We also still clear the screen at the top of draw. 
That happens on every state change, so there's no reason to not just do it then. 
Besides, it will help us debug if we make any mistakes, since the screen will turn white.

Let's continue the delegation by adding a `draw` method to `WalkTheDogStateMachine`
that can extract the `state` from each case for `drawing`, as shown here:


```rust
// src/game.rs


impl WalkTheDogStateMachine {
    ...
    fn draw(&self, renderer: &Renderer) {
        match self {
            WalkTheDogStateMachine::Ready(state) => state.draw(renderer),
            WalkTheDogStateMachine::Walking(state) => state.draw(renderer),
            WalkTheDogStateMachine::GameOver(state) => state.draw(renderer),
        }
    }
}
```
This is virtually identical to the update method we wrote earlier, except on a borrowed
self instead of consuming self . The rest is just delegations to the various states. Unlike
update, every state draws in the exact same way, so we can fill those in with one method,
as shown here:

```rust
// src/game.rs

//REDESIGN this is new.
struct WalkTheDogState<T> {
    ...
}

impl<T> WalkTheDogState<T> {
    fn draw(&self, renderer: &Renderer) {
        self.walk.draw(renderer);
    }
}


```

Any state will delegate draw to `Walk` because the drawing doesn't actually change based
on state. We can finally go ahead and re-implement the `draw` method, this time on `Walk`,
as shown here:


```rust
// src/game.rs

impl Walk {
    fn draw(&self, renderer: &Renderer) {
        self.backgrounds.iter().for_each(|background| { background.draw(renderer); });
        self.boy.draw(renderer);
        self.obstacles.iter().for_each(|obstacle| { obstacle.draw(renderer); });
    }
    ...
}

```
This code is not new, but I don't blame you if you forgot it. 
It's our old draw code from Chapter 6, Creating an Endless Runner, 
only with the walk variable replaced by self . The rest is identical.

At this point, you'll notice something exciting – the code compiles again! 
But if you look closely at the game, you'll see that it's a little static.

Red Hat Boy has stopped animating! He doesn't do his little idle animation because
we're not calling update like we used to; it's almost time to go back to fixing the
update method.


---------


```rust
// src/game.rs



```


