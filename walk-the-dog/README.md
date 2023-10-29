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

#### Refactoring `RedHatBoyContext` and `RedHatBoy`

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

Tip::
    I know what you're thinking – performance! We're cloning on each transition!
    You're absolutely right, but do you know that the performance is adversely
    impacted? The first rule of performance is measure first, and until we measure
    this, we don't actually know if the final version of this code is a problem. I spent
    a lot of time trying to avoid this clone call because of performance concerns,
    and it turned out not to make much of a difference at all. Make it work, then
    make it fast.
    
Once you fix that error a few times, you're ready to add the audio and the sound to
RedHatBoyContext , but what sound will we play?

#### Adding a sound effect

Using the Web Audio API, we can play any sound format that is supported by the audio
HTML element, which includes all the common formats of WAV, MP3, MP4, and Ogg. In
addition, in 2017, the MP3 license expired, so if you're concerned about that, don't be; you
can use MP3 files for sounds without worry.


Since the `Web Audio API` is compatible with so many audio formats, you can use sound
from all over the internet, provided it's released under the appropriate license. The
sound effect we'll be using for jumping is available at 
[opengameart.org](https://opengameart.org/content/8-bit-jump-1) and is released 
under the Creative Commons public domain license, so we can use it without concern. 

You don't need to download that bundle and browse through it, although you can, 
but the jump sound is already bundled with this book's assets 
at https://github.com/PacktPublishing/Game-Development-with-Rust-and-WebAssembly/wiki/Assets 
in the sounds directory. 
The specific file we want is `SFX_Jump_23.mp3`. 
You'll want to copy that file into the static directory of your Rust project 
so that it will be available for your game.

Now that `RedHatBoyContext` is ready to hold the `Audio` struct, 
and the `SFX_Jump_23.mp3` file is available to be loaded, 
we can start adding that code. 

Start with adding `Audio` and `Sound` to `RedHatBoyContext` as shown here:

```rust
// src/game.rs

#[derive(Clone)]
pub struct RedHatBoyContext {
    pub frame: u8,
    ...
    audio: Audio,
    jump_sound: Sound,
}

```

Remember to add use declarations for `Audio` and `Sound` 
to the `red_hat_boy_states` module

```rust
// src/game.rs

use crate::{
    browser,
    engine::{self, Cell, Game, Image, KeyState, Point, Rect, Renderer, Sheet, SpriteSheet, Sound, Audio},
    segments::{stone_and_platform, platform_and_stone,},
};
...

mod red_hat_boy_states {
    use crate::engine::Point;
    //use super::HEIGHT;
    use super::{Audio, Sound, HEIGHT};
    ...
}

```

The code will stop compiling because RedHatBoyContext
is being initialized without `audio` or `jump_sound`, so we'll need to add that.

`RedHatBoyContext` is initialized in the new method of the `RedHatBoyState<Idle>`
implementation so we'll change that method to take `Audio` and `Sound` objects that we'll
pass into `RedHatBoyContext` as shown here:

```rust
// src/game.rs

    impl RedHatBoyState<Idle> {
        //pub fn new() -> Self {
        pub fn new(audio: Audio, jump_sound: Sound) -> Self {
            RedHatBoyState {
                context: RedHatBoyContext {
                    frame: 0,
                    position: Point { x: STARTING_POINT, y: FLOOR, },
                    velocity: Point { x: 0, y: 0 },
                    
                    audio,
                    jump_sound,
                },
                _state: Idle {},
            }
        }// fn new
    ...

```

We could create an `Audio` object here, but then the new method would need to return
`Result<Self>` and I don't think that's appropriate. 

This will move the compiler error, because where we call `RedHatBoyState<Idle>::new` is now wrong. 
That is in `RedHatBoy::new`, which can now also take `Audio` and `Sound` objects and pass them through.

```rust
// src/game.rs

impl RedHatBoy {
    //fn new(sprite_sheet: Sheet, image: HtmlImageElement) -> Self {
    fn new(sprite_sheet: Sheet, image: HtmlImageElement, audio: Audio, sound: Sound) -> Self {
        RedHatBoy {
            //state_machine: RedHatBoyStateMachine::Idle(RedHatBoyState::new()),
            state_machine: RedHatBoyStateMachine::Idle(RedHatBoyState::new(audio, sound)),
            sprite_sheet,
            image,
        }
    }
....

```


This leads us to our infamous `initialize` function in our `Game` implementation, which
fails to compile because it calls `RedHatBoy::new` without `Audio` or `Sound`. 

This is the appropriate place to load a file, both because it is `async` and because it returns a `result`.

We'll create an `Audio` object in `initialize`, 
load up the `sound` we want, and pass it to the `RedHatBoy::new` function, 
as shown here:

```rust
// src/game.rs

#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        match self {
            WalkTheDog::Loading => {
                let sheet = browser::fetch_json("../resources/pix/rhb.json").await?.into_serde()?;
                let audio = Audio::new()?;
                let sound = audio.load_sound("../resources/sound/SFX_Jump_23.mp3").await?;
                let rhb = RedHatBoy::new(sheet, 
                                         engine::load_image("../resources/pix/rhb.png").await?,
                                         audio,
                                         sound,);
                ...
                

```

This will get the app compiling again, but we don't do anything with audio or sound.
Remember that all this work was done because we wanted to make sure the sound is only
played once when we jump, and the way to ensure that is to put the playing of the sound
in the transition from `Running` to `Jumping`. 

Transitions are done in the various `From` implementations via methods on `RedHatBoyContext`. 

Let's write a small function called `play_jump_sound` on `RedHatBoyContext`, 

as shown here:

```rust
// src/game.rs

    impl RedHatBoyContext {
        ...
        fn play_jump_sound(self) -> Self {
            if let Err(err) = self.audio.play_sound(&self.jump_sound) {
                log!("Error playing jump sound {:#?}", err);
            }

            self
        }
    }//^-- impl RedHatBoyContext
```

This function is written a little differently than the other transition side effect functions in
this implementation, because `play_sound` returns a `result`, but in order to be consistent
with the other transition methods, `play_jump_sound` really shouldn't. 

Fortunately, failing to play a sound, while annoying, isn't fatal, 
so we'll log the error and continue if the sound couldn't be played. 

The code now compiles, but we need to add the call to `play_jump_sound` to the `transition`. 

Look for `jump` on `RedHatBoyState<Running>` and modify that `transition` to call `play_jump_sound`, 
as shown here:

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
}
```

When this compiles, run the game and you'll see, and hear, RHB jump onto a platform.

Tip::
    
    If, like most developers I know, you have 20+ browser tabs open right now, you
    may want to close them. It can slow down the browser's sound playback and
    make the sound timing off.
    

Now that you've played one sound effect, consider adding more, for example, when RHB
crashes into an obstacle, or lands cleanly, or slides. The choices are up to you! After you've
had a little fun with sound effects, let's add some background music.


### Playing long music

You might think that playing music will mean detecting whether the sound is
complete and restarting it. This is probably true for the browser's implementation,
but fortunately, you don't have to do it. 
The `Web Audio API` already has a flag on the `AudioBufferSourceNode` loop 
that will play the sound on a loop until it is explicitly stopped. 
This will make playing background audio rather simple. 
We can add a flag to the `play_sound` function in the `sound module` for the `loop` parameter, 
as shown here:

```rust
// src/sound.rs

fn create_track_source( ctx: &AudioContext, 
                        buffer: &AudioBuffer) -> Result<AudioBufferSourceNode> {
    let track_source = create_buffer_source(ctx)?;
    track_source.set_buffer(Some(&buffer));
    connect_with_audio_node(&track_source, &ctx.destination())?;

    Ok(track_source)
}

pub enum LOOPING {
    NO,
    YES,
}

```
The `create_track_source` function, which is actually a refactoring
of the `play_sound` function. It takes the first three lines of it and extracts them into
a separate function for readability. After that, we create a `LOOPING` enum and use it to
check whether we should call `set_loop` on `track_source`.

Our play_sound with a flag looks like this:

```rust
// src/sound.rs


pub fn play_sound( ctx: &AudioContext, 
                   buffer: &AudioBuffer,
                   looping: LOOPING) -> Result<()> {
    let track_source = create_track_source(ctx, buffer)?;
    if matches!(looping, LOOPING::YES) {
        track_source.set_loop(true);
    }

    track_source
        .start()
        .map_err(|err| anyhow!("Could not start sound!{:#?}", err))
}

```

By adding this flag, our program stops compiling because `Audio` in the `engine` 
is still calling `play_sound` with two parameters.

We can quickly fix that, as shown here:

```rust
// src/engine.rs

impl Audio {
    ...
    pub fn play_sound(&self, sound: &Sound) -> Result<()> {
        //sound::play_sound(&self.context, &sound.buffer)
        sound::play_sound(&self.context, &sound.buffer, sound::LOOPING::NO)
    }  
...
}

```

We'll also add a new method to play background music, which is just playing a sound with
looping turned on:

```rust
// src/engine.rs

impl Audio {
    ...
    pub fn play_looping_sound(&self, sound: &Sound) -> Result<()> {
        sound::play_sound(&self.context, &sound.buffer, sound::LOOPING::YES)
    }  
...
}
```

I like how the `engine` has progressively less flexibility than the sound module. 
The `sound` and `browser` modules are wrappers around the browser functionality; 
the `engine` provides utilities to help you make a game. 
Now that the `engine` provides a way to play background music, 
we can actually add it to the `game`. 

In the assets, there's a second file in the sounds directory, `background_song.mp3`, 
which you can copy into the static directory of this project. 
Once you've done that, we can load and play the background music in our `Game::initialize` function:


```rust
// src/game.rs

#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        match self {
            WalkTheDog::Loading => {
                ...
                let audio ...
                    ...
                let sound ...
                    ...
                let background_music = audio.load_sound("../resources/sound/background_song.mp3").await?;
                //play it now                
                audio.play_looping_sound(&background_music)?;

```


Tip::
    
    Check out https://gamesounds.xyz/ 
    for royalty-free sounds for your games.
    
There you have it! A proper game with music and sound effects! Now to add a UI, so we
can actually click New Game on it.    

### Summary

In this chapter, you added sounds to your game using the Web Audio API and got an
overview of the API itself. The Web Audio API is very broad and has a ton of features, and
I'd encourage you to explore it. Your first challenge is to use the gain property to change
the volume of the music, which is rather loud at the moment. The Web Audio API also
supports features such as stereo surround sound and programmatically generated music.

Have some fun and try it out!

You also added a new module to the game, and further extended the game engine to
support it. We even covered refactoring and made some trade-offs to ensure the game
would finish without requiring a time-consuming ideal design. I encourage you to take
some time to add more sound effects to the game; you have the skills now to make
RHB thud when he lands or crashes into a rock. 

Speaking of crashing into rocks, you're probably sick of having to hit refresh every time you do that, 
so in the next chapter, we'll add a small UI with a wonderful New Game button.


---------

```rust
// src/game.rs



```





