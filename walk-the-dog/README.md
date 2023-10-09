We've accomplished a lot. 
- We've created a game loop that will run in the browser at 60 frames per second 
while updating at a fixed step. 
- We've set up an XNA-like game "engine" and separated the engine concerns from the game concerns. 
- Our browser interface is wrapped in a module so that we can hide some of the details of the browser
implementation. 
- We're even processing input, making this work like a true game engine.

We did all this while keeping the code running as we went.
The code should be easier to work with going forward because we now have clear places
to put things. 
Browser functions go in a browser, 
engine functions in an engine, 
and the game in a game module, 

Next make RHB run, jump, and slide around.

## Managing Animations with State Machines

We're going to introduce a common game development pattern, the state machine, implemented in Rust. Rust gives us powerful constructs for state machines but also unique challenges 
due to its ownership model, so we'll dive into that and why we'll use it instead 
of deceptively simple if statements.

Applications, games included, have to manage the state of the system.
Our game is doing a lot of things and is maintaining a large game state with a
lot of mini-states inside it. 
As the application moves from one state to another, the rules of the system change. 
For example, when RHB is running, the spacebar might make him jump, 
but when he's jumping, hitting the spacebar doesn't do anything. 
The rule is you can't jump when you're already jumping.

### Defining a state machine

Perhaps the most confusing thing about state machines is the naming, 
as there are state machines, finite state machines, the state pattern, 
and more, all of which frequently get used interchangeably by programmers. 

So, for the sake of clarity, let's define them this way:

• **State machines**: A model of the state of a system, represented by a list of states and
the transitions between them
• **State pattern**: One way to implement state machines, which we will not be using in
our application, although our implementation will bear a resemblance to it

Important Note::
    
    The Rust Programming Language has an implementation of the traditional
    state pattern, using a trait object, which you can find here: 
    [TRLPB: Object Oriented design patterns](https://doc.rust-lang.org/book/ch17-03-oo-design-patterns.html). 
    It's quite good but is not idiomatic Rust, and we won't be using it.


The state machine both helps us keep a mental model of the system in our heads and
prevents us from making foolish mistakes in code, such as playing the running animation
while RHB is jumping. The drawback, of course, is that you need to understand state
machines, so let's get that covered. We'll use RHB as our example. RHB can be Running,
Idle, Jumping, Sliding, Falling, or KnockedOut.

### Implementing with types

The Object-Oriented (OO) state pattern is typically implemented as a variation on the
strategy pattern, where you swap out different objects that all implement the same state
interface at runtime based on the various transitions.

Also see rust state machine pattern at [Hoverbear](https://hoverbear.org/blog/rust-state-machine-pattern/)

#### The typestate pattern

Typestate is a fancy name for embedding the state of an object in its type. 
The way it works is that you have a generic structure with one generic parameter representing 
the state. 
Then, each state will have methods that can return new states. 
So, instead of each state having common methods, each state has its own methods 
that return the new state.


```
 +------------------------------------------------+  +------------------------+
 | State<GenericStateOne>                         |  | State<GenericStateTwo> |
 .------------------------------------------------.  .------------------------.
 | next(self, parameters): State<GenericStateTwo> |  |  update(&mut self)     |
 +------------------------------------------------+  +------------------------+
```

In this diagram, `State<GenericStateOne>` has a next method, 
which consumes `self` and returns `State<GenericStateTwo>`. 
Meanwhile, `State<GenericStateTwo>` only has an `update` method, 
which takes `&mut self` a mutably borrowed self. 
The implications of this are that the compiler will catch you if you try
to call `next` on `State<GenericStateTwo>`. 

In the traditional OO pattern, all states must handle all the same methods 
because they share an interface, so this kind of defense isn't possible. 
Often, this means implementing methods you don't actually care about, and
either returning an error state or Self , and then debugging at runtime.


In addition, we can use the `mod` keyword and Rust's rules about privacy to make it
impossible to create any state in an invalid state. We can make it impossible to move
from `GenericStateOne` to `GenericStateTwo` without calling next by keeping
the internals of State private so you can't just construct it. 
This is called making illegal states unrepresentable, 
and it's a great way to make sure you don't make mistakes in your programs.

Tip::
    
        There's a lot of great information on typestates in Rust. 
        There's an excellent talk by Will Crichton from 
        [Strange Loop]( https://youtu.be/bnnacleqg6k?t=2015 ), as well as blogs at 
        https://docs.rust-embedded.org/book/static-guarantees/typestate-programming.htm 
        and http://cliffle.com/blog/rust-typestate/ .


• Each state of the object is represented by a separate struct.  
• You can only advance from one state to another by methods on that struct.  
• You can guarantee you can only create valid states using privacy rules.  

Finally, we're going to need an enum to hold our typestate. 
Each state is generic, so to continue in our preceding example, 
any struct that will interact with our state machine will need 
to hold either `State<GenericStateOne>` or `State<GenericStateTwo>`. 
In order to do that, we would either need to make the containing struct generic as well, 
and then create new versions of the containing struct every time the state changes, 
or wrap the generic object in an enum .

We'll use an enum because it prevents the generic nature of the typestate 
from propagating throughout the program, allowing the typestate to be an implementation detail. 
We're going to write the kind of state machine that Rust is very good at.


### Managing animation
We'll create our state machine to manage the different animations. Specifically, when
RHB isn't moving, he's Idle , but when he's moving, he's Running . When he jumps, he's
Jumping .

Those different RHB states correspond to the different animations 
managed using a state machine. 

We'll first create the RHB with a state machine and then integrate it 
into our current application. 
We'll implement this top-down, starting with a struct that represents RHB 
and letting the compiler errors drive further development. 

This is sometimes called `Compiler-Driven Development` although it's not 
a formalized approach such as Test-Driven Development (aka no books, 
seminars, workshops, or conferences). It can work extremely well in a language 
with a robust type system and great compiler errors, such as Rust. 

Let's start with how we'll represent RHB.
The RedHatBoy struct will contain the state machine, the sprite sheet, 
and the image because eventually, it will draw itself:

```rust
// filename: src/game.rs

struct RedHatBoy {
    state_machine: RedHatBoyStateMachine,
    sprite_sheet: Sheet,
    image: HtmlImageElement,
}

```

Let's create RedHatBoyStateMachine :

```rust
// filename: src/game.rs

#[derive(Copy, Clone)]
enum RedHatBoyStateMachine {
    Idle(RedHatBoyState<Idle>),
    Running(RedHatBoyState<Running>),
}
```

It might still be unclear why we're using `enum` when we'll be creating 
all of these `typestate` structures. 
RedHatBoyState, which doesn't exist yet, is a generic type that contains another type, 
where those types represent the various states. So, why the redundant enum? 
Because we want to be able to switch easily between the states without using the heap 
or dynamic dispatch. 
We'll see that the enum wrapper becomes extremely useful as we implement the state machine.


we haven't created either of those states or the RedHatBoyState x. 
This is what is meant by Compiler-Driven Development. 

We can start by creating RedHatBoyState

```rust
// filename: src/game.rs

mod red_hat_boy_states {
    use crate::engine::Point;


    #[derive(Copy, Clone)]
    pub struct RedHatBoyState<S> {
        context: RedHatBoyContext,
        _state: S,
    }

    #[derive(Copy, Clone)]
    pub struct RedHatBoyContext {
        frame: u8,
        position: Point,
        velocity: Point,
    }

}

```

All the code relating to the individual states will go in its own module, 
`red_hat_boy_states`, so that we can only make public the methods required 
by the rest of the game module. 
This will make it impossible to accidentally create a state without using 
the methods provided, and therefore, impossible to accidentally make an invalid transition. 
The only way to transition from `RedHatBoyState<Idle>` to `RedHatBoyState<Running>` 
is going to be through the methods on `RedHatBoyState<Idle>`. 
It's important that both `RedHatBoyState` and `RedHatBoyContext` are **public** 
but **their members are private**, so we can use them as intended.

Inside the new module, `RedHatBoyState` is a simple generic type that contains `_state`, 
which is never read, hence the underscore, and `RedHatBoyContext`.
Now, `RedHatBoyContext` is a structure with data that's common to all the states. 
In this case, that's the frame being rendered, the position, and the velocity. 

We'll need it so that the state transitions can modify the state of RHB. 
Putting all of this in the `red_hat_boy_states` module means that we haven't 
changed the compiler error message. 
We need to import that module into the game module with 
`use self::red_hat_boy_states::*;` , which you can add anywhere in the game module.


```rust
// filename: src/game.rs
...
use self::red_hat_boy_states::*;

...
```

Both `Idle` and `Running` don't exist. 
We can create both of these easily, with empty structures 
inside the `red_hat_boy_states` module.

```rust
// filename: src/game.rs
...

mod red_hat_boy_states {
    use crate::engine::Point;

    ...

    #[derive(Copy, Clone)]
    pub struct Idle;

    #[derive(Copy, Clone)]
    pub struct Running;

```

### Transitioning between states

We'll add a method on `RedHatBoyState<Idle>` to go from `Idle` to `Running` :

```rust
// filename: src/game.rs

mod red_hat_boy_states {
    ....
    impl RedHatBoyState<Idle> {
        pub fn run(self) -> RedHatBoyState<Running> {
            RedHatBoyState {
                context: self.context,
                _state: Running {},
            }
        }
    }
```

This is the transition from `Idle` to `Running`, and the run method is where the magic
happens. This is just a function that takes a `RedHatBoy<Idle>` state and converts it to
a `RedHatBoy<Running>` state, and for now, doesn't change any `RedHatBoyContext` data. 
You might wonder then, what magic?
This means that to transition from `Idle` to `Running`, you can use run, but it also means
you can't transition from `Running` back into `Idle`, and that makes sense because the
game doesn't allow that behavior. 

The function also takes `mut self`, so that when it's called, it consumes the current state. 
This means that if you want to somehow keep `Idle` around after transitioning to `Running`, 
you have to clone it, and if you do that, you probably really meant to do it.


You also can't create the Running state directly, because its data members are private,
which means you can't just create that state by mistake. You can't create the Idle state
either, and that's a problem because it's the start state. We'll address that in a moment, but
first, let's dive into how we'll interact with the states through our state machine.

### Managing the state machine

Initially, we might be tempted to implement our state machine 
by adding methods on the `RedHatBoyStateMachine` enum , as follows:

```rust
// Dont do this

#[derive(Copy, Clone)]
enum RedHatBoyStateMachine {
    Idle(RedHatBoyState<Idle>),
    Running(RedHatBoyState<Running>),
}

impl RedHatBoyStateMachine {
    fn run(self) -> Self {
        match self {
            RedHatBoyStateMachine::Idle(state) =>
            RedHatBoyStateMachine::Running(state.run()),
            _ => self,
        }
    }
}
```

This isn't terrible, but it means that every method on our state machine will likely need
to match the current variant of the `RedHatBoyStateMachine` enum. 
Then, it would return the new variant based on either the transition 
or self when the transition isn't currently valid. 
In other words, while the compiler will error if we call run on the Running state, 
it won't error if we call run on `RedHatBoyStateMachine` when the current variant is Running. 

This kind of error, where we call run by mistake on the wrong state, 
is exactly what we're trying to get away from with our typestates. 
We'd go to all the trouble of writing these typestates only to immediately 
throw away one of the benefits in every method on the `RedHatBoyStateMachine` enum .

Unfortunately, we can't completely get away from that problem, because we are using an
enum to contain our states. 

There's no way to implement methods on variants of an enum as we can with generic structures, 
and if we're going to wrap the state in an enum, we'll have to match on the variant. 
What we can do is reduce the surface area of that kind of error by reducing 
the number of methods that operate in the states. 

Specifically, instead of calling run on the enum, 
we'll create a transition function that takes Event. 
That is going to look like the following code:

```rust
#[derive(Copy, Clone)]
enum RedHatBoyStateMachine {
    Idle(RedHatBoyState<Idle>),
    Running(RedHatBoyState<Running>),
}

pub enum Event {
    Run,
}

impl RedHatBoyStateMachine {
    fn transition(self, event: Event) -> Self {
        match (self, event) {
            (RedHatBoyStateMachine::Idle(state), Event::Run) => {
                RedHatBoyStateMachine::Running(state.run())
            }
            _ => self,
        }
    }
}

```
We've solved the problem caused by the enum with another enum! 
This is very Rusty of us. 
In this case, we've created an enum named `Event` to represent every event 
that could happen to our machine and replaced the method named `run` 
with a method named `transition`.

So, instead of many small methods for run, jump, and similar, 
we will have one method named `transition` and a bunch of `Event` variants. 

How does this improve things? 
Because there is only one match statement that we have to update 
when we want to add a transition, instead of potentially adding multiple 
little match statements. 

Keep in mind that this function takes `mut self`, which means 
calling transition will consume `self` and return a **new** `RedHatBoyStateMachine` 
just as the run method does on `RedHatBoyState<Idle>`.

#### Using Into for clean code
We can actually improve the ergonomics of this method using the `From trait`. 
If you're unfamiliar, the `From trait` is a Rust feature that lets us define 
how to convert from one type to another. 
Implementing the `From trait` on your type will also implement the `Into trait`, 
which will provide an `into` method that will make it easy 
to convert from one type to another.

We know that if we have `RedHatBoyState<Running>`, 
it will convert into the `RedHatBoyStateMachine::Running` variant, 
and if we write the conversion by implementing the `From trait`, 
we will be able to replace that wrapping with a call to `into`.

The following is what the implementation of the `From trait` looks like:

```

impl From<RedHatBoyState<Running>> for RedHatBoyStateMachine {
    fn from(state: RedHatBoyState<Running>) -> Self {
        RedHatBoyStateMachine::Running(state)
    }
}
```

This can be placed right under the implementation of RedHatBoyStateMachine . It
defines how to convert from RedHatBoy<Running> to RedHatBoyStateMachine ,
and it's the same small amount of code we wrote in the transition method. Because
we have this now, we can make that method a little more succinct, as shown here:

```rust
impl RedHatBoyStateMachine {
    fn transition(self, event: Event) -> Self {
        match (self, event) {
            (RedHatBoyStateMachine::Idle(state), Event::Run) => state.run().into(), 
            //{
            //    RedHatBoyStateMachine::Running(state.run())
            //}
            _ => self,
        }
    }
}

```

Replacing calls like `RedHatBoyStateMachine::Idle::Running(state.run)`
with `into` isn't just prettier and more concise; 
it also means that if run changes to return a different state, 
the transition method can stay the same, as long as a `From trait` has
been written to go from the state to the `RedHatBoyStateMachine` enum . 
It's a nice little change that makes our code more flexible.

It's a little odd that the `RedHatBoyStateMachine` enum is what we call our state
machine because we don't normally associate enumerated types with behavior, but this
method is why we call it a machine. 
We use enum to hold the various generic states, 
and we use the ability to add methods to an enum 
to make it a lot more ergonomic to use. 

The various states know how to transition from one state to another, 
and the machine knows when to do the transitions.








