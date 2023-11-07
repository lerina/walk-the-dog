# Testing and Advanced Tricks

## Debugging the game

### Debugger

To debug a traditional program, be it in Java, C#, or C++, we must set breakpoints and
step through the code. In JavaScript, we can type the word debugger to set a breakpoint,
but although WebAssembly runs in the browser, it isn't JavaScript. 

So, how do we debug it?

There's a lot of conflicting information about debugging with WebAssembly. 
How do you debug WebAssembly? 

Well, according to the official Rust WebAssembly documentation, it's simple – you can't!


Unfortunately, the debugging story for WebAssembly is still immature.
On most Unix systems, DWARF is used to encode the information that a
debugger needs to provide source-level inspection of a running program.
There is an alternative format that encodes similar information on
Windows. Currently, there is no equivalent for WebAssembly. Therefore,
debuggers currently provide limited utility, and we end up stepping through
raw WebAssembly instructions emitted by the compiler, rather than the
Rust source text we authored.

[SOURCE](https://rustwasm.github.io/docs/book/reference/debugging.html)

So, there you have it – no debugging, section over. That was easy.
Debbugers don't work yet. 

But it's not that simple.
Debuggers are still useful for inspecting the JavaScript that interacts with our WebAssembly, and inspecting raw wasm state.


And of course, you can debug your application – you just can't use your browser's developer tools 
to step through the Rust code in a debugger. 

The technology isn't there yet. But that doesn't mean we don't debug; 
it just means we'll take more of an old-school approach to debugging.


Earlier, I mentioned that when I write code, I typically write a lot of tests. I also typically
don't use a debugger very often. If we break our code into smaller chunks that can be
easily exercised by tests, a debugger is rarely required. 

That said, we didn't do that for this project, so we'll need a way to debug existing code. 

We'll start by logging, then getting stack traces, 
and finally using linters to prevent bugs before they happen.

Note::

    The reality is not as cut and dry as the Rust Wasm site would state. Chrome
    developer tools have added support for the DWARF debugging format to the
    browser, as detailed here: https://developer.chrome.com/blog/
    wasm-debugging-2020/ . This standard format, whose specification can
    be found at https://dwarfstd.org/ , unfortunately is not supported
    by wasm-bindgen at the time of writing. You can see progress on this
    issue here: https://github.com/rustwasm/wasm-bindgen/
    issues/2389 . By the time you read this book, the debugging tools may
    be modernized in Rust Wasm, as well as in browsers outside of Chrome, but
    for the time being, we must use more traditional tools such as println!
    and logging.
    

### Log versus error versus panic

If you've been following along and got confused at some point, then you've probably used
the `log!` macro we wrote in Chapter 3, Creating a Game Loop, to see what was going on.

If you have been doing that, congratulations! You've been debugging the same way I did
when I wrote the code originally. 

**Print line** debugging is still standard in many languages and it's pretty much 
the only form of debugging that's guaranteed to work anywhere. 

If you haven't done that, then it looks like this:


```rust
// src/game.rs

impl WalkTheDogStateMachine {

    fn update(self, keystate: &KeyState) -> Self {
        log!("Keystate is {:#?}", keystate);

        match self {
            WalkTheDogStateMachine::Ready(state) => state.update(keystate),
            WalkTheDogStateMachine::Walking(state) => state.update(keystate),
            WalkTheDogStateMachine::GameOver(state) => state.update(),
        }
    }

```

In the preceding example, we are logging `KeyState` on every tick through the `update` function. 
This isn't a great log because it's going to show an empty KeyState 60 times a second, 
but it's good enough for our purposes. 

However, there's one flaw in this log: `KeyState` doesn't implement the Debug trait. 
You can add it by adding the `derive(Debug)` annotation to the `KeyState` struct, like so:


```rust
// src/engine.rs


#[derive(Debug)]
pub struct KeyState {
    pressed_keys: HashMap<String, web_sys::KeyboardEvent>,
}

```


When you add this, the console will log all your key state changes, 
which will be useful if your keyboard input is broken:

> In general, any pub struct should use `#[derive(Debug)]`, 

but this isn't the default option since it could make compile times long on large projects. 
When in doubt, go ahead and use #[derive(Debug)]` and log the information. 

Now, maybe `log!` isn't noticeable enough for you, and you want the text to be bright, 
obvious, and red. 

For that, you'll need to use console.error in JavaScript and write a macro such as the log macro, 
which we already have in the browser module. 


```rust
// src/browser.rs

// It's a macro that allows you to log in to the console with log!
// using a syntax such as the format! function.
// Taken from https://rustwasm.github.io/book/game-of-life/debugging.html

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}
```

This time the macro looks like this:

```rust
// src/browser.rs


macro_rules! error {
    ( $( $t:tt )* ) => {
        web_sys::console::error_1(&format!( $( $t )* ).into());
    }
}
```

This is the same as the log macro but uses the `error` function on the console object instead og `log`.
There are two advantages to the `error` function. 
The first is that it's red, 
while the other is that it also will show you the stack trace. 

Here's an example of error being called when
the player is knocked out in Chrome:


![Error log](./readme_pix/error_log.png)


It's not the most readable stack trace in the world, but after seeing a few lines of
the `console::error_1` function, you can see that this `log` was called from
`WalkTheDogState<Walking>::end_game`. 
This log is really for true errors, as opposed to just informational logging, 
and this stack trace may not show up clearly in all browsers. 

You'll also want to be cautious with leaving this log in the production code 
as you may not want to expose this much information to a curious player. 

We'll want to make sure it's not in the production deployment, 
which we'll create in Chapter 10, Continuous Deployment.


Finally, if you want to make sure the program stops when an error occurs, we'll want to go
ahead and use the `panic!` macro. 

Some errors are recoverable but many are not, 
and we don't want our program to limp along in a broken state. 

In Chapter 1, Hello WebAssembly, we included the `console-error-panic-hook` crate 
so that if the program were to panic, we'd get a stack trace. 

Let's replace calling `error!` with calling `panic!` and see the difference:

![Panic log](./readme_pix/panic_log.png)


Here, you can see it looks a little different, but the information is mostly the same. 
There is one thing at the very top where it says `src/game.rs:829`, which tells you exactly where
panic was called. 
In general, you will probably prefer to use panic compared to error if you need to have the error 
in your production code because that kind of error should be rare and fail fast. 

The error function is more useful during debugging, so you'll end up removing those.

There's another kind of error that we've been ignoring at times, and that's the warnings
and errors that are given to you by the compiler and linter. 

We can use the Rust ecosystem's tools to detect mistakes before we ever run the program. 
Let's look into that now.

### Linting and Clippy

One of the features that makes the Rust compiler great is that it has a `linter` built into it,
in addition to the warnings and errors it already provides. If you're unfamiliar, a linter
is a static code analysis tool that typically finds style errors and, potentially, logic errors
above and beyond what the compiler can find. The term comes from the lint you find on
clothing, so you can think of using a linter like rubbing a lint brush on your code. 

We've been getting some warnings from the compiler that we've been ignoring for a while now,
most of which look like this:

```
warning: unused `Result` that must be used
    --> src/game.rs:1032:9
     |
1032 |         browser::hide_ui();
     |         ^^^^^^^^^^^^^^^^^^
     |
     = note: this `Result` may be an `Err` variant, which should be handled
help: use `let _ = ...` to ignore the resulting value
     |
1032 |         let _ = browser::hide_ui();
     |         +++++++

```
These are all cases where an error could occur, but we probably don't want to crash if it
does, so panicking or calling `unwrap` isn't an option. 
Propagating the `Result` type is an option, but I don't think we want to prevent moving 
from one state to another if there's a small browser issue. 

So, instead, we'll use the error case to log here. 
You can see it at https://bit.ly/3q1936N in the sample source code. 
Let's modify the code so that we log any errors:

```rust
// src/game.rs

impl WalkTheDogState<GameOver> {
    ...
    fn new_game(self) -> WalkTheDogState<Ready> {
        //browser::hide_ui();
        if let Err(err) = browser::hide_ui() {
            error!("Error hiding the browser {:#?}", err);
        }

        WalkTheDogState {
            ...
```

Here, we have changed the `browser::hide_ui()` line to 
`if let Err(err) = browser::hide_ui()` and we log if an error occurs. 
We can see what that error log will look like by forcing `hide_ui` to return an error for a moment:


![A fake error](./readme_pix/fake_error.png)

The stack trace is cut off in book form, but you can see that we got an error log 
with `Error hiding the browser` and then This is the error in the `hide_ui` function, 
which is the error message I forced into `hide_ui`. 
The stack trace also shows `game::Ready`, 
which would show you that you were transitioning into the `Ready` state 
if you had infinite room to show the entire message.

Every single warning that's being generated should be dealt with. Most of the warnings
are the same kind – that is, Result types where the Err variant is ignored. These can
be removed by handling the Err case with a log or by calling panic if the game should
truly crash at this time. 
For the most part, I've used the if let pattern but if `request_animation_frame` fails, 
then I just use unwrap. 
I don't see how the game could work if that's failing.

There is one more warning we've been ignoring that we should address, as shown here:



---------


```rust
// src/game.rs



```


