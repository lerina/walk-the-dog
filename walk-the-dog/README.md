## Sound Effects and Music

While our game may be playable, it's just not a game without some sound. 
To play sound in our game, we'll need to learn how to use the browser's 
`Web Audio API` for both short and long sounds.

In this chapter, we will cover the following topics:

• Adding the Web Audio API to the engine
• Playing sound effects
• Playing long music

By the end of this chapter, you won't just see RHB run, jump, and dodge obstacles,
but you'll be able to hear him too after we add sound effects and music to our game.

Let's get started!

### Adding the Web Audio API to the engine

We'll be using the browser's Web Audio API to add sound to our game.
The API is incredibly full-featured, allowing for mixing audio sources and special effects,
but we're just going to use it to play background music and sounds. In fact, the Web
Audio API is its own book and, if you're interested, you can [find one at ](https://
webaudioapi.com/book/) .


Once we've got an overview of the Web Audio API, we'll create a module to play sounds in
Rust, load the sounds in the same way as we load our images, and finally, add that sound
to the engine.

The Web Audio API may initially look familiar when compared to Canvas. As with
Canvas, you create a context that then provides an API for playing sounds. At that point,
the similarity ends. Because the Web Audio API has all the features I mentioned earlier, it
can be hard to figure out how to do the basic act of playing a sound. Unlike Canvas, there's
no drawImage equivalent called playSound or something like that. Instead, you have
to get the sound data, create AudioBufferSourceNode , connect it to a destination,
and then finally start it.

In JavaScript, the code to load and prepare
a sound for playback looks like the following:

```javascript
// if we did it in javascript it would be like this:

const audioContext = new AudioContext();
let sound = await fetch("SFX_Jump_23.mp3");
let soundBuffer = await sound.arrayBuffer();
let decodedArray = await audioContext.
decodeAudioData(soundBuffer);
```

It starts by creating a new AudioContext , which is built into the browser engine, then
fetching a sound file from the server. The fetch call eventually returns a response, which
we'll need to decode. We do this by first getting its arrayBuffer , which consumes it,
and then we use the audioContext we created at the beginning to decode the buffer
into a sound that can be played. 

Note how everything is asynchronous, which will cause us a little trouble 
in the Rust code as we map JavaScript promises to Rust futures.

The previous code should only be done once for any sound resource since loading 
and decoding the file can take significant time.

The following code will play a sound:

```javascript
// if we did it in javascript it would be like this:

let trackSource = audioContext.createBufferSource();
trackSource.buffer = decodedArray;
trackSource.connect(audioContext.destination);
trackSource.start();
```

Ugh, that's not intuitive, but it's what we have. Fortunately, we can wrap it in a few
simple functions that we'll be able to remember, and forget all about it. 

It creates the `AudioBufferSourceNode` we need with `createBufferSource`, assigns it
the array that we decoded into audio data in the previous section, connects it to the
audioContext , and finally, plays the sound with start. 
It's important to know that you cannot call start on `trackSource` twice, 
but fortunately, the creation of a buffer source is very fast and won't require us to cache it.


That's great! We know the eight lines of code to play a sound in JavaScript, but how do we
get this into our engine?

### Playing a sound in Rust

We're going to create a sound module that's very similar to our browser module,
a series of functions that just delegate right to the underlying JavaScript. It will be a very
bottom-up approach, where we'll create our utility functions and then create the final
functions that use them. We'll start by focusing on the parts we need for a play_sound
function.


Note:

Remember that you want these functions to be very small – it's a thin layer
between Rust and JavaScript – but also to change the interface to better match
what you want to do. So, eventually, rather than talking about buffer sources
and contexts, we'll want to call that play_sound function we wish existed in
the first place.

We'll start by creating the module in a file named `sound.rs` living alongside the rest of
our modules in src. 

```sh
$ touch src/sound.rs
```

Don't forget to add a reference to it in src/lib.rs , as shown here:

```rust
// src/lib.rs

#[macro_use]
mod browser;
mod engine;
mod game;
mod segments;
mod sound;

....
```

Our first function will create an `AudioContext` in a Rusty way 
as opposed to the JavaScript way we already saw, and that's as follows:

```rust
// src/sound.rs

use anyhow::{anyhow, Result};
use web_sys::AudioContext;

pub fn create_audio_context() -> Result<AudioContext> {
    AudioContext::new().map_err(|err| anyhow!("Could not create audio context: {:#?}", err))
}

```

As usual, the Rust version of the code is more verbose than the JavaScript version. 

That's the price we pay for the positives of Rust. 
None of this code is particularly new; we're mapping `new AudioContext` to `AudioContext::new`, 
and we're mapping the `JsResult` error to an `anyhow` result that it might return, to be more Rust-friendly.

This code doesn't compile though; take a moment and think about why. 

It's the infamous feature flags for web-sys in Cargo.toml that we haven't added `AudioContext` to, 
so let's add that now:

```toml
[dependencies.web-sys]
version = "0.3.64"
features = ["console",
            "Window",
            "Document",
            "HtmlCanvasElement",
            "CanvasRenderingContext2d",
            "Element",
            "HtmlImageElement",
            "Response",
            "Performance",
            "KeyboardEvent",
            "AudioContext",
            ]
```

Now that we've set up the sound module, created the function to create `AudioContext`,
and refreshed our memory on the process of adding a new feature to the web-sys
dependency, we can go ahead and add a little more code to play sounds. 

Let's introduce all the remaining feature flags you'll need to add to web-sys in Cargo.toml:

```toml
[dependencies.web-sys]
version = "0.3.64"
features = ["console",
            ...
            "AudioContext",
            "AudioBuffer",
            "AudioBufferSourceNode",
            "AudioDestinationNode",
            ]
```
The three features, `AudioBuffer`, `AudioBufferSourceNode`, and
`AudioDestinationNode`, correspond to those same objects in the original
JavaScript code. 
For instance, 
the `let trackSource = audioContext.createBufferSource();` function returned `AudioBufferSourceNode`. 

The `web-sys` authors have chosen to hide a large number of `audio` features under individual flags, 
so we need to name them one at a time.

Now that we have the features ready, we can add the rest of the code. 
Back in the sound module, the code will look like this:


```rust
// src/sound.rs

use anyhow::{anyhow, Result};
use web_sys::{AudioBuffer, AudioBufferSourceNode, AudioContext,
              AudioDestinationNode, AudioNode};

...
fn create_buffer_source(ctx: &AudioContext) -> Result<AudioBufferSourceNode> {
    ctx.create_buffer_source()
            .map_err(|err| anyhow!("Error creating buffer source {:#?}", err))
}

fn connect_with_audio_node( buffer_source: &AudioBufferSourceNode,
                            destination: &AudioDestinationNode,
                          ) -> Result<AudioNode> {

    buffer_source.connect_with_audio_node(&destination)
                 .map_err(|err| anyhow!("Error connecting audio source to destination {:#?}", err))
}
```

In this book, we've typically gone through the code one function at a time, 
but for these two it's not necessary. 
These functions correspond to the calls to `audioContext.createBufferSource` 
and `trackSource.connect(audioContext.destionation)` respectively. 

We've converted the code from the object-oriented style of JavaScript 
into a slightly more procedural format with the functions taking parameters,
in part so that we can map errors from the `JsValue` types into proper Rust Error types
via the `anyhow!` macro.

Now that we have the three functions, we need to play a sound. We can go ahead and
write the function that plays it right below them, shown here:


```rust
// src/sound.rs

...
pub fn play_sound(ctx: &AudioContext, buffer: &AudioBuffer) -> Result<()> {
    let track_source = create_buffer_source(ctx)?;
    track_source.set_buffer(Some(&buffer));
    connect_with_audio_node(&track_source, &ctx.destination())?;

    track_source
       .start()
       .map_err(|err| anyhow!("Could not start sound!{:#?}", err))
}
```

The `play_sound` function accepts `AudioContext` and `AudioBuffer` as parameters,
then returns the result of the start call, with `JsValue` mapped to `Error`. 

We haven't created an `AudioBuffer` yet anywhere, so don't worry that you don't know how to
as we'll cross that bridge when we come to it. 

What we have here is a function that is very similar to the original JavaScript for playing a sound, 
but with the additional error handling that comes with Rust, including using the `?` operator 
to make it easier to read, and a little bit of additional work around `None` in the `track_source`.

`set_buffer(Some(&buffer));` line, where we need to wrap a reference to
`AudioBuffer` in `Some` because `track_source` has an *optional buffer*. 
In JavaScript, this is null or undefined, but in Rust, we need to use the `Option` type. 

Otherwise, both the JavaScript and Rust versions do the same thing to play a sound:

1. Create `AudioBufferSource` from `AudioContext`.
2. Set `AudioBuffer` on the source.
3. Connect `AudioBufferSource` to the `AudioContext` destination.
4. Call `start` to play the sound.


This seems like a lot, but in reality, it's very fast, so there's not much use in caching
`AudioBufferSource`, especially since you can only call start once. Now that we
can play a sound, it's time to load a sound resource and decode it, so that we have an
`AudioBuffer` to play. 

Let's do that now.

### Loading the sound

To load a sound from the server, we'll need to translate the following code, which you've
already seen, into Rust.

```javascript
// if we did it in javascript it would be like this:

const audioContext = new AudioContext();
let sound = await fetch("SFX_Jump_23.mp3");
let soundBuffer = await sound.arrayBuffer();
let decodedArray = await audioContext.
decodeAudioData(soundBuffer);
```


Fetching the resource is something we can already do in our browser module, but we
don't have a handy way to get its `arrayBuffer`, so we'll need to add that. 

We'll also need to create a Rust version of `decodeAudioData`. 

Let's start with the changes we need to add to browser, 
which are modifications to existing methods. 

We'll want to split the old fetch_json function, 
which looks like this:


```rust
// src/browser.rs

pub async fn fetch_json(json_path: &str) -> Result<JsValue> {
    let resp_value = fetch_with_str(json_path).await?;
    let resp: Response = resp_value
        .dyn_into()
        .map_err(|element| anyhow!("Error converting {:#?} to Response", element))?;

    JsFuture::from(
        resp.json()
            .map_err(|err| anyhow!("Could not get JSON from response {:#?}", err))?,
    )
    .await
    .map_err(|err| anyhow!("error fetching JSON {:#?}", err))
}
```

We need to split it into two functions that 
- first fetch `Result<Response>`, 
- then a second that converts it into JSON


```rust
// src/browser.rs

pub async fn fetch_response(resource: &str) -> Result<Response> {
    fetch_with_str(resource)
                        .await?
                        .dyn_into()
                        .map_err(|err| anyhow!("error converting fetch to Response {:#?}", err))
}


pub async fn fetch_json(json_path: &str) -> Result<JsValue> {
    let resp = fetch_response(json_path).await?;
    JsFuture::from( resp.json()
                        .map_err(|err| anyhow!("Could not get JSON from response {:#?}", err))?,
    )
    .await
    .map_err(|err| anyhow!("error fetching JSON {:#?}", err))
}

```

This is a classic case of the second person pays for abstraction, where we wrote the code we
needed in Chapter 2, Drawing Sprites, to load JSON, but now we need a version of fetch
that can handle multiple kinds of responses, specifically, sound files that will be accessible
as an `ArrayBuffer` instead. 
That code will need `fetch_response` but will convert it into a different object. 

Let's write that code now, right below `fetch_json`:

```rust
// src/sound.rs

...

pub async fn fetch_array_buffer(resource: &str) -> Result<ArrayBuffer> {
    let array_buffer = fetch_response(resource)
                        .await?
                        .array_buffer()
                        .map_err(|err| anyhow!("Error loading array buffer {:#?}", err))?;

    JsFuture::from(array_buffer)
                        .await
                        .map_err(|err| anyhow!("Error converting array buffer into a future {:#?}", err))?
                        .dyn_into()
                        .map_err(|err| anyhow!("Error converting raw JSValue to ArrayBuffer {:#?}", err))
}
```

Just as `fetch_json` does, this starts by calling `fetch_response` with the passed-in
resource. Then, it calls the `array_buffer()` function on that response, which will
return a promise that resolves to `ArrayBuffer`. 
Then, we convert from a promise to `JsFuture` as usual, in order to use the await syntax. 
Finally, we call `dyn_into` to convert the `JsValue` that all Promise types return into `ArrayBuffer`. 

I've skipped over it, but at each step, we use `map_err` to convert the `JsValue` errors into `Error` types.


The `ArrayBuffer` type is a JavaScript type that isn't available to our code yet. It's a core
JavaScript type, defined in the `ECMAScript` standard, and in order to use it directly, we
need to add the `js-sys` crate. 

This is somewhat surprising, as we are already pulling in `wasm-bindgen` and `web-sys`, 
which are both dependent on JavaScript, so why do we need to pull in yet another crate for `ArrayBuffer`? 

This has to do with how the various crates are arranged. 
The `web-sys` crate **has all the web APIs** 
where `js-sys` is limited to code that is in the `ECMAScript` standard. 

Up to now, we haven't had to use anything in core JavaScript except what was exposed by web-sys, but this changes with ArrayBuffer.

In order for this code to compile, you'll need to add `js-sys = "0.3.64"`

It is already in dev-dependencies , so you can just
move it from there. 

```toml
...
[dependencies]
wasm-bindgen = { version = "0.2.87", features = ["serde-serialize"] }
...
js-sys = "0.3.64"

...

# These crates are used for running unit tests.
[dev-dependencies]
wasm-bindgen-test = "0.3.37"
#js-sys = "0.3.64"

```

You'll also need to add a use js_sys::ArrayBuffer declaration
to bring into scope the ArrayBuffer struct .

```rust
// src/browser.rs
...
use js_sys::ArrayBuffer;

...

```

Now that we can fetch a sound file and get it as an `ArrayBuffer`, 
we're ready to write our version of `await audioContext.decodeAudioData(soundBuffer)`. 

By now, you may have noticed that we're following the same pattern for wrapping every
JavaScript function like this:

1. Convert any function that returns a promise, such as `decode_audio_data`, 
into `JsFuture` so you can use it in **asynchronous** Rust code.

2. Map any errors from `JsValue` into your own error types; in this case, we're using
`anyhow::Result` but you may want more specific errors.

3. Use the `?` operator to propagate errors.


4. Check for feature flags, particularly when using `web_sys` and you just know a
library exists.

To this, we'll add one more step.

5. Cast from `JsValue` types to more specific types using the `dyn_into` function.

Following that same pattern, the Rust version of decodeAudioData goes in the sound
module, like this:

```rust
// src/sound.rs

pub async fn decode_audio_data( ctx: &AudioContext, 
                                array_buffer: &ArrayBuffer,) -> Result<AudioBuffer> {

    JsFuture::from( ctx.decode_audio_data(&array_buffer)
                       .map_err(|err| anyhow!("Could not decode audio from array buffer {:#?}", err))?,
                  ).await
                    .map_err(|err| anyhow!("Could not convert promise to future {:#?}", err))?
                    .dyn_into()
                    .map_err(|err| anyhow!("Could not cast into AudioBuffer {:#?}", err))
}

```


You'll need to make sure you add use declarations for `js_sys::ArrayBuffer` and
`wasm_bindgen_futures::JsFuture`, and also `wasm_bindgen::JsCast` 
to bring the `dyn_into` function into scope. 


```rust
// src/sound.rs
...
use js_sys::ArrayBuffer;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::JsCast;
```

Once again instead of directly calling the
method on `AudioContext`, in this case `decodeAudioData`, we've created a function
that wraps the call. 
It borrows a reference to `AudioContext` as the first parameter and takes the `ArrayBuffer` type 
as the second parameter. 

This allows us to encapsulate the mapping of errors and casting of results into a function.

This function then delegates to `ctx.decode_audio_data`, passing it `ArrayBuffer`,
but if that's all it did we wouldn't really need it. It then takes any error from 
`ctx.decode_audio_data` and maps it to `Error` with `anyhow!`; 
in fact, as you can see,
it will ultimately do this at every step in the process, pairing that with the `?` operator
to propagate the error. 
It takes a promise from `decode_audio_data` and creates `JsFuture` from it, 
then immediately calls `await` to wait for completion, corresponding to the `await` call in JavaScript. 

After handling any errors converting the promise to `JsFuture`, we use the `dyn_into` function to cast it 
to `AudioBuffer`, ultimately handling any errors with that as well.

That function is the most complicated of the wrapper functions, so let's reiterate the steps
we did when translating from one line of JavaScript to nine lines of Rust:

1. Convert any function that returns a promise into `JsFuture` so you can use it in
asynchronous Rust code.
In this case, `decode_audio_data` returned a promise, and we converted it into
`JsFuture` with `JsFuture::from`, then immediately called `await` on it.

2. Map any errors from `JsValue` into your own error type; in this case, we're using
`anyhow::Result`, but you may want more specific errors.
We did this three times, as every call seemed to return a `JsValue` version of the
result, adding clarifying language to the error messages.

3. Cast from `JsValue` types to more specific types using the `dyn_into` function.
We did this to convert the ultimate result of `decode_audio_data` from
`JsValue` to `AudioBuffer`, and Rust's compiler could infer the appropriate type
from the return value of the function.

4. Don't forget to use the `?` operator to propagate errors; note how this function does
that twice.
We used the `?` operator twice to make the function easier to read.

5. Check for feature flags, particularly when using `web_sys` and you just know a library exists.

`AudioBuffer` is feature flagged, but we added that back at the beginning.
This process is a bit more complicated to explain than it is in practice. For the most part,
you can follow the compiler and use tools such as rust-analyzer to do things such as
automatically add use declarations.

Now that we've got all the utilities, we need to play a sound. It's time to add that feature to
the engine module so our game can use it.


### Adding audio to the engine

The functions we just created in the sound module could be used by the engine
directly via delegation functions, but we don't want to make the game worry about
`AudioContext`, `AudioBuffer`, and things like that. Just like `Renderer`, we'll create
an `Audio` struct that encapsulates the details of that implementation. 

We'll also create a `Sound` struct to convert `AudioBuffer` into a friendlier type for the rest of the system.

Those will be very small, as shown here:

```rust
// src/engine.rs

#[derive(Clone)]
pub struct Audio {
    context: AudioContext,
}

#[derive(Clone)]
pub struct Sound {
    buffer: AudioBuffer,
}
```

These structs are added to the bottom of the engine module, but they can really be put
anywhere in the file. Don't forget to import `AudioContext` and `AudioBuffer`! 

```rust
// src/engine.rs
...
use web_sys::{AudioContext, AudioBuffer,};
...
```

If you're finding yourself getting confused as engine and game get larger, you're welcome
to break that up into multiple files with a `mod.rs` file and a directory, but to follow along,
everything needs to end up in the engine module. I'm not going to do that because,
while it makes the code a bit easier to navigate, it makes it harder to explain and follow
along with. Breaking it up into smaller chunks later is an excellent exercise to make sure
you understand the code we're writing.

Now that we have a struct representing `Audio` holding `AudioContext`, and a
corresponding `Sound` holding `AudioBuffer`, we can add impl to Audio, which uses
the functions we wrote earlier to play a sound. 

Now, we'll want to add `impl` to the `Audio` struct to play sounds and load them. 

Let's start with the load implementation, which is probably the hardest, 
as seen here:


```rust
// src/engine.rs

impl Audio {
    pub fn new() -> Result<Self> {
        Ok(Audio { context: sound::create_audio_context()?,})
    }

    pub async fn load_sound(&self, filename: &str) -> Result<Sound> {
        let array_buffer = browser::fetch_array_buffer(filename).await?;
        let audio_buffer = sound::decode_audio_data(&self.context, &array_buffer).await?;
        
        Ok(Sound { buffer: audio_buffer,})
    }
}
```


This `impl` will start with two methods, the familiar new method that creates an `Audio`
struct with `AudioContext`. Pay attention to the fact that new returns a result in this
case, because `create_audio_context` can fail. 
Then, we have the `load_sound` method, which also returns a result, this time of the `Sound` type, 
which is only three lines. 
This is a sign we did something right with the way we organized our functions in the sound and browser modules, 
as we can simply call our `fetch_array_buffer` and `decode_audio_data` functions to get `AudioBuffer` 
and then wrap it in a `Sound` struct. 

We return a result and propagate errors via `?`. 
If loading a sound was simple, then playing it is easy in this method on the `Audio` implementation:

```rust
// src/engine.rs

impl Audio {
        ...

    pub fn play_sound(&self, sound: &Sound) -> Result<()> {
        sound::play_sound(&self.context, &sound.buffer)
    }   

}
```

For `play_sound`, we really just delegate, passing along `AudioContext` that `Audio`
holds and `AudioBuffer` from the passed-in sound.

We've written a module to play sounds in the API, added loading sounds to the browser,
and finally created an audio portion of our game engine. That's enough to play a sound
effect in the engine; 

now we need to add it to our game, and here it's going to get complicated.




---------

```rust
// src/sound.rs



```





