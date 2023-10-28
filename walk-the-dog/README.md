## Sound Effects and Music

### Playing sound effects

Adding sound effects to our game is a challenge for several reasons:

• Effects must only occur once:

We'll be adding a sound effect for jumping (boing!) and want to make sure that it
only happens one time. Fortunately, we have something for that already, our state
machine! We can use `RedHatBoyContext` to play a sound when something
happens, something like this (don't add it yet):

```
impl RedHatBoyContext {
    ...
    fn play_jump_sound(audio: &Audio) {
    audio.play_sound(self.sound)
    }
}
```

This leads directly into our second challenge.

• Playing audio on transitions:

We want to play the sound at the moment of transition, but most transitions won't
play a sound. Remember our state machine uses transition to transition from
one event to another, and while we could pass in the audio there it would only be
used by a small portion of the code in that method. 
It's a code smell, so we won't do that. 

`RedHatBoyContext` will have to own the audio and the sound. 

This isn't ideal, we'd prefer there to be only one audio in the system, 
but that's not workable with our state machine. 
That leads to our third problem.

• `AudioContext` and `AudioBuffer` are not `Copy`:

In order to use syntax such as `self.state = self.state.jump();`
in the `RedHatBoy` implementation and have each state transition consume
`RedHatBoyContext`, we needed `RedHatBoyContext` to be Copy.

Unfortunately, `AudioContext` and `AudioBuffer` are not `Copy`, 
which means `Audio` and `Sound` cannot be `Copy` 
and, therefore, if `RedHatBoyContext` is going to hold `audio` and a `sound`, 
it cannot also be a `copy`. 

This stinks, but we can fix it by refactoring `RedHatBoyContext` and `RedHatBoy` 
to use the clone function as needed.

Having `RedHatBoyContext` own an audio means that there will be more than one
`Audio` object in the system potentially, where the other will play music. 
This is redundant but mostly harmless, so it's the solution we'll go with. 
It gets us moving forward with development, and in the end, the solution works well. 

> When in doubt, choose the solution that ships.


Note::
    
    You may wonder why we don't store a reference to Audio in
    RedHatBoyContext . Ultimately, Game is static in our engine, and
    therefore, an Audio reference must be guaranteed to live as long as Game if
    it's stored as a reference on RedHatBoyContext .
    There are other options, including using the service locator pattern
    ( https://bit.ly/3A4th2f ) or passing in the audio into the update
    function as a parameter, but they all take longer to get us to our end goal of
    playing a sound, which is the real goal of this chapter.
    

Before we can add a sound effect to the game, we're going to refactor the code to hold an
Audio element. Then we'll play the sound effect.

### Refactoring `RedHatBoyContext` and `RedHatBoy`

We're going to prepare `RedHatBoyContext` and `RedHatBoy` to hold audio and a song
before we actually do it because that will make it easier to add the sound. 

Let's start by making `RedHatBoyContext` just clone, as shown here:

```rust
// src/gane.rs
    
    #[derive(Clone,)]    // #[derive(Copy, Clone)]
    pub struct RedHatBoyContext {
        pub frame: u8,
        pub position: Point,
        pub velocity: Point,
    }

```


All we've done is removed the Copy trait from the derive declaration. This will cause
compiler errors on RedHatBoyStateMachine and RedHatBoyState<S> , which
both derive Copy , so you'll need to remove that declaration on those structures as well.
Once you've done that, you'll see a bunch of errors like this:

```
nerror[E0507]: cannot move out of `self.state` which is behind a mutable reference
```


As expected, the calls to `self.state.<method>`, where the method takes `self`, 
all fail to compile, because `RedHatBoyStateMachine` doesn't implement `Copy` anymore.

The solution, and we'll do this on every line with this compiler error, is to **explicitly clone** 
the state when we want to make the change. 

Here's the run_right function with the error


```rust
// src/game.rs

impl RedHatBoy {
    ...
    fn run_right(&mut self) {
        //self.state_machine = self.state_machine.transition(Event::Run);
        self.state_machine = self.state_machine.clone().transition(Event::Run);
    }

```


Perhaps the most teeth-grindingly offensive instance of this 
is in the transition method, where we will get a move because of the match statement.
The trouble with this section is that `self` is moved into the match statement and cannot
be returned in the default case. 

Trying to use `match` and `self` to get around the issue causes all of the `typestate` methods, 
such as `land_on` and `knock_out`, to fail because they need to consume `self`. 

The cleanest fix is to clone `self` as shown here:

```
impl RedHatBoyStateMachine {
    fn transition(self, event: Event) -> Self {
        //match (self, event) {
        match (self.clone(), event) {
            ...
            _ => self,
        }
    }
```


---------

```rust
// src/game.rs



```





