# Testing and Advanced Tricks

## Testing, Debugging, and Performance

Developing a game can be a long process, especially if you're a hobbyist. When you only
have 4 hours to work on it in a given week, they can't all be spent fighting the same bug.
To ensure our game works, we need to test it, find mistakes, and make sure it's not too
slow. That's what we're going to be doing here.

In this chapter, we will cover the following topics:

• Creating automated tests
• Debugging the game
• Measuring performance with the browser

After completing this chapter, you'll be able to fix the bugs we've written so far and make
sure they don't happen again.

### Creating automated tests

Some ways to test your game is working correctly involve doing the following:

• Using types to prevent programmer errors
• Playing the game yourself
• Performing automated unit tests
• Performing automated integration tests

Almost any application can benefit from automated, programmer-written unit tests and as
a program becomes even larger, it begins to benefit from integration tests as well. There's
not a consistent definition of the differences between these two types of tests as you tend
to know them when you see them, but fortunately, we can use the Rust definitions.

Rust and Cargo provide two kinds of testing:

• Unit tests via cargo test
• Integration tests via `wasm-pack` test

**Unit tests** tend to be programmer-centric.  
They are written at the method or function level, with minimal dependencies.   
You may have a test for every branch of an if/else statement,  
while in the case of a loop, you may have tests for when a list has 0, 1, or many entries.  
These tests are small and fast and should run in seconds or less.  
These are my preferred form of testing.

**Integration tests** tend to look at the app at a higher level. In the case of our code, the
integration tests automate the browser and will work based on an event (such as a mouse
click) throughout the program. These tests take longer to write, are harder to maintain,
and often fail for mysterious reasons. So, why write them? Unit tests typically do not
test parts of your application or they may only do so in small doses. This can lead to
a situation where your unit tests all pass but the game doesn't work. Most systems will
have fewer integration tests than unit tests because of their disadvantages, but they will
need them for their benefits.


In Rust, unit tests are written side by side with a module and run with cargo test . In
our setup, they will run as part of a Rust executable, running directly on the machine.

Integration tests are stored in the tests directory and only have access to things your
crate makes public. They run in the browser – potentially a headless one – with `wasm-pack` test. 
Unit tests can test internal methods directly, while integration tests must use your crate 
as a real program would.

### Test-driven development

I have a confession to make. I usually write all my code in a test-driven style, where you
write a test then make it fail for each step in the development process. Had we followed
that process during the development of this book, we'd likely have quite a few tests –
perhaps more than 100. In addition, test-driven development (TDD) exerts a lot of
pressure on the design that tends to lead to more loosely coupled code. So, why didn't we
do this?

Well, TDD has its downsides, with perhaps the largest being we'd generate a lot more code
in the form of tests. We've already written a lot of code in this book, so imagine trying to
follow along with the tests too – you can see why I felt it was best to leave out the kind of
testing I normally write. 

Test-Driven Rust isn't the title of this book after all. 
However, just because we didn't write tests first doesn't mean we don't want to be sure our code works.
That's why, in many cases, we used the type system as the first line of defense against
mistakes, such as using the typestate pattern for state transitions. 

The type system is one of the advantages of using Rust instead of JavaScript for this game.

This isn't to say that automated testing cannot provide value for our program. The Rust
ecosystem places a high value on testing, so much so that a testing framework is built
into Cargo and is automatically set up for any Rust program. 

With unit tests, we can test algorithms such as collision detection or our famous state machines. 
We can make sure that the game still does what we expect, although we can't test whether a game 
is fun or pretty. 

For that, you'll have to play the game until you hate it, but a game is a lot more fun 
if the basics work. We can use tests, along with types, to ensure the code works as
expected so that we can turn our focus to whether or not it's fun. 

To do that, we'll need to set up the `test runner` and then write some tests 
that run outside of the browser and inside the browser.

#### Getting started

As we mentioned earlier, Rust has built-in capabilities for running tests – both unit
and integration. Unfortunately, the template we used way back in Chapter 1, Hello WebAssembly, 
still has an out-of-date setup at the time of writing. If it hasn't been fixed,
running cargo test at the command prompt will fail to compile, let alone run the tests.

Fortunately, there are not a lot of mistakes. There's just some out-of-date async code for
a browser test we won't be using in the automatically generated tests. Those tests are in the
tests directory in the app.rs file. 

This is traditionally where integration tests are put in Cargo projects. 
We'll change that setup shortly by using unit tests, but first, 
let's get this to compile by deleting the incorrect `async_test` setup test. 

In `app.rs`, you can delete that function and the `#[wasm_bindgen_test(async)]` macro above it 
so that your `app.rs` file looks like this:


```rust
// tests/app.rs

use futures::prelude::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

// This runs a unit test in native Rust, so it can only use Rust APIs.
#[test]
fn rust_test() {
    assert_eq!(1, 1);
}

// This runs a unit test in the browser, so it can use browser APIs.
#[wasm_bindgen_test]
fn web_test() {
    assert_eq!(1, 1);
}

```

Note
After this book has been published, the template will be fixed and will likely
compile. I'm going to assume this, regardless of you changing the code, so that
it matches what is here going forward.

Some of the use declarations aren't needed anymore, but they will be short so you can
leave them in and ignore the warnings. 

Now, `app.rs` contains two tests – one that will run in a JavaScript environment, such as the browser, 
and one that will run as a native Rust test. 
Both of these are just examples, where 1 is still equal to 1. 

To run the native Rust tests, you can run `cargo test`, as you might be accustomed to. 
That will run the `rust_test` function, which is annotated with the `test macro`. 

You can run the browser-based tests, which are annotated with the `wasm_bindgen_test macro`, 
via the `wasm-pack test --headless` `--chrome` or `--firefox` command. 
This will run the web tests using the Chrome or firefox browser, in a headless environment. 
You can also use `--safari`, and `--node` if you wish, but you must specify 
what JavaScript environment you'll be running them in. 

Note that `--node` isn't going to work since it doesn't have a browser.

We'll start writing tests using the `#[test] macro`, which runs Rust code in the native environment, 
just like writing a standard Rust program. 
The simplest thing to test is a pure function, so let's try that.

#### Pure functions

Pure functions are functions that, given the same input, will always produce the same
output, without side effects such as drawing to the screen or accessing the network. 

They are analogous to mathematical functions, which just do a calculation, and are by far the
easiest types of code to unit test. 

These tests do not require the browser, so they use the
`#[test] ` annotation and run with `cargo test`.


The current setup runs our only Rust test in the `tests/app.rs` file, which makes it,
as far as Cargo is concerned, an *integration test*. 
I don't like that and prefer to use the Rust convention of writing *unit tests* in the file where the code 
is executed. 

In this first example, we'll test the intersects function on `Rect`, 
which is a pure function that is complicated enough to mess up. 

We'll add this test to the bottom of `engine.rs` because that's where `Rect` is defined, 
and we'll run it with `cargo test`. 

Let's add a test to the bottom of the module for the intersect method on Rect, 
as shown here:

```rust
// src/engine.rs

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn two_rects_that_intersect_on_the_left() {
        let rect1 = Rect {
            position: Point { x: 10, y: 10 },
            height: 100,
            width: 100,
        };
        let rect2 = Rect {
            position: Point { x: 0, y: 10 },
            height: 100,
            width: 100,
        };

        assert_eq!(rect2.intersects(&rect1), true);
    }
}//^-- mod tests

```

Much of this is documented in the [Rust book at](https://doc.rust-lang.org/book/ch11-00-testing.html), 
but a little review never hurts anyone. 
We start by using the `#[cfg(test)]` attribute macro to tell `Cargo` not to compile 
and run this code except when we're running tests. 
Then, we create a tests module using the `mod` keyword to isolate our tests from the rest 
of the code. After that, we bring into scope the `engine` code with use `super::*`. 
Then, we write our test by writing a function, `two_rects_that_intersect_on_the_left`, which is
annotated with the `#[test] macro` so that the test runner can pick it up. 

The rest of this is a pretty standard test. 

It creates two rectangles, where the second overlaps the first, and
then makes sure that the intersects function returns true . 
You can run this test with `cargo test`, where you'll see the following output:

```
Finished test [unoptimized + debuginfo] target(s) in 1.05s
     Running unittests src/lib.rs (target/debug/deps/walk_the_dog-91fd784866f8434c)

running 1 test
test engine::tests::two_rects_that_intersect_on_the_left ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/app.rs (target/debug/deps/app-8dbf6eb66a05e337)

running 1 test
test rust_test ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

You'll see two sets of results with ok. 1 passed;. 
The first result references our new test, `two_rects_that_intersect_on_the_left`, which will pass. 
Then, you will see `rust_test run`, which will also pass. 
The `rust_test` test is in `tests\app.rs` and was created with the project skeleton. 
It is run as an integration test because it is in the tests directory – this is the Cargo standard. 

The difference between `unit tests` and `integration tests` is that the integration tests 
are run as a separate crate and use the production code as a separate library. 

This means they use the code in the same way a user of your crate would, but they cannot call internal 
or private functions. It's easier to get complete coverage when you're running unit tests 
with the caveat that they may be less realistic. 

Our code is not meant to be used as a crate, so we won't be using many integration tests.

Now that we've written our first unit test for our code, we can write a lot more tests for
this intersects method, including when the following occurs:

• When the rectangles overlap on the top or bottom
• When the rectangles overlap on the right
• When the rectangles don't overlap – that is, when the function returns false

We should have a test for every branch in the intersects function. 

We leave these tests as an exercise for you since repeating them would be redundant. 

As our code base grows, it would be ideal if much of our code could easily be tested like this, 
but unfortunately, for this game, a lot of it interacts with the browser, 
so we will have two different ways to test that. 

The first way is to replace the browser with a stub so that we don't need to run browser-based tests. 
We'll do that in the next section.

#### Hiding the Browser module

Way back in Chapter 3, Creating a Game Loop, we **separated browser functions** into
a `browser` module. 

We can use this as a seam to inject test versions of the browser functions 
that will run as native Rust code and allow us to write tests.


Note::
    
    The term seam comes from the book Working Effectively with Legacy Code, by
    Michael Feathers ( https://amzn.to/3kas1Fa ). It's written in C++
    and Java but is still the best book on legacy code you can find. 
    A seam is a place where you can insert test behavior to replace real behavior, 
    while an enabling point is a point in the code that allows that to happen.
    

A **seam** is somewhere we can alter the behavior of the program 
without altering the code in that place. 

Look at the following code from the game module:

```rust
// src/game.rs

impl WalkTheDogState<GameOver> {
    ...
    fn new_game(self) -> WalkTheDogState<Ready> {
        browser::hide_ui();

        WalkTheDogState {
            _state: Ready,
            walk: Walk::reset(self.walk),
        }
    }
}

```

We'd like to test that **when** the game goes from the `GameOver` state to the `Ready` state,
the **UI is hidden**. 

We can do this with integration tests by checking whether the div property that contains the UI 
is empty after this transition. 

We may want to do this, but such tests are frequently a little harder to write and maintain. 
This is especially true as the game grows. 

Another approach, which we'll use here, is to replace the `browser module` 
with a version of it that doesn't interact with the browser. 
The **seam** is `hide_ui`, which is a behavior we can replace without actually changing the code, 
while the **enabling point** is the `use` declaration, which is where we brought in the `browser` module.

We can enable using a *test version of the browser module* with **conditional compilation**.

In the same way that the `#[cfg(test)]` macro only includes the test module when
compiling in test mode, we can import different versions of the `browser` module with
`cfg` directives, as shown here:

```rust
// src/game.rs

#[cfg(test)]
mod test_browser;

#[cfg(test)]
use test_browser as browser;

#[cfg(not(test))]
use crate::browser;
```

The preceding code can be found at the top of the `game` module, where we were
previously importing `crate::browser`. Here, we can use the `mod` keyword to bring
the contents of the `test_browser module` in from the `src/test_browser.rs` file, 
but only when we're running a test build. 
Then, we can use `test_browser` as browser to make the functions available via `browser::` calls – again, 
only in test builds – in the same way as we call the browser production code. 

Finally, we can add the `#[cfg(not(test))]` annotation to use `crate::browser` to prevent the real
browser code from being imported into the test.

Now we have:

```rust
// src/game.rs

...

use self::red_hat_boy_states::*;

#[cfg(test)]
mod test_browser;

#[cfg(test)]
use test_browser as browser;

#[cfg(not(test))]
use crate::browser;

use crate::{
    //browser,
    engine::{ self, Cell, Game, Image, KeyState, Point, Rect, 
              Renderer, Sheet, SpriteSheet, Sound, Audio},
    segments::{stone_and_platform, platform_and_stone,},
};
...
```


See [Mocking in Rust with conditional compilation](https://klau.si/blog/mocking-in-rust-with-conditional-compilation/)


If you do this and run `cargo test`, you'll see a lot of errors, 
such as cannot find function `fetch_json` in module `browser` , because even though we're
importing a test module, we haven't filled it in with any code. 
In this situation, it's a good idea to follow the compiler errors, 
which will point out that there's no file yet in `src/test_browser.rs`. 
It will also list the functions that are used in the `game` module but aren't defined 
in our `test_browser.rs` file. 

To get past this, you can create the `test_browser.rs` file and bring in the bare minimum 
that's needed to get back to compiling, as shown here:

```rust
// src/test_browser.rs

use anyhow::{anyhow, Result};
use wasm_bindgen::JsValue;
use web_sys::HtmlElement;

pub fn draw_ui(html: &str) -> Result<()> {
    Ok(())
}

pub fn hide_ui() -> Result<()> {
    Ok(())
}

pub fn find_html_element_by_id(id: &str) -> Result<HtmlElement>
{
    Err(anyhow!("Not implemented yet!"))
}

pub async fn fetch_json(json_path: &str) -> Result<JsValue> {
    Err(anyhow!("Not implemented yet!"))
}

```
As you can see, only four functions are used in game that have been defined in browser,
and we've filled in just enough to compile. 

To use this for testing, we're going to need to place simulated implementations 
with some sort of state they keep track of. 

The other thing you may notice is that `JsValue` and `HtmlElement` are both being used 
in this code since they won't work when you run Rust native tests. 

They require a browser runtime, so to continue along this path, 
we'll eventually need to make test versions of `HtmlElement` and `JsValue` 
or create wrapper types for them, potentially in the `engine` module. 

Let's leave those as is for now, though, and try to write our first test
using the standard Rust testing framework. 

We'll want to test the state machine change I mentioned previously by setting up the game 
in the `GameOver` state and transitioning to the `Running` state, 
then checking that the `UI` was hidden. 

The beginning of that test looks as follows:

```rust
// src/game.rs
...

#[cfg(test)]
mod tests {
    use super::*;
    use futures::channel::mpsc::unbounded;
    use std::collections::HashMap;
    use web_sys::{AudioBuffer, AudioBufferOptions};

    fn test_transition_from_game_over_to_new_game() {
        let (_, receiver) = unbounded();
        let image = HtmlImageElement::new().unwrap();
        let audio = Audio::new().unwrap();
        let options = AudioBufferOptions::new(1, 3000.0);
        let sound = Sound {
            buffer: AudioBuffer::new(&options).unwrap(),
        };
        let rhb = RedHatBoy::new(
            Sheet {
                frames: HashMap::new(),
            },
            image.clone(),
            audio,
            sound,
        );
        let sprite_sheet = SpriteSheet::new(
            Sheet {
                frames: HashMap::new(),
            },
            image.clone(),
        );
        let walk = Walk {
            boy: rhb,
            backgrounds: [
                Image::new(image.clone(), Point { x: 0, y: 0 }),
                Image::new(image.clone(), Point { x: 0, y: 0 }),
            ],
            obstacles: vec![],
            obstacle_sheet: Rc::new(sprite_sheet),
            stone: image.clone(),
            timeline: 0,
        };
        let state = WalkTheDogState {
            _state: GameOver {
                new_game_event: receiver,
            },
            walk: walk,
        };
    }//^-- fn test_transition_from_game_over_to_new_game
}//-- mod tests 
```

Oh boy – that's a lot of code to test a few lines of Rust, and it's not even a complete test yet.
It's just setting up the game in the state that we need it to be in before we transition into
a Ready state. 

A lot is being revealed about our design, specifically that it's what I may call naïve'. 
It's very hard to construct objects, and while the game, engine, and browser modules are separate, 
they are still pretty tightly coupled. It works but it in a fashion that only solves the problem 
in front of us. 

That's completely acceptable – we had specific goals to build a small endless runner and we did it, 
but this also means that if we wanted to start extending our game engine so that it's more flexible, 
we would need to make further changes. 

I tend to view **software design** more like **sculpting** than constructing. 
You start with a big block of code and chip away at it until it looks like what you want, 
rather than a blueprint that you follow to get the perfect house.

Some of the aspects of our design that this test is revealing are as follows:

• It's not easy to create new `Walk` structures.
• The `game` module is far more coupled to `web-sys` and `wasm-bindgen` than we thought.

We made the intentional choice not to try and create perfect abstractions early in the project. 
This is one of the reasons we didn't write this code in a test-driven style initially.
TDD would have strongly pushed in the direction of further abstraction and layering,
which would have hidden the game code we're trying to learn here. 

As an example, instead of using `HtmlImageElement` or `AudioBuffer`, 
we may have written wrappers or abstractions around those objects 
(we already have an Image struct), which is probably better for growing our project 
in the medium to long term but is harder to understand in the short term.

This is a long-winded way of saying that this code is now hard to write isolated unit tests
for because we didn't build it with them in mind. 

If you were able to run this test, you would see the following:

```
thread 'game::tests::test_transition_from_game_over_to_new_
game' panicked at 'cannot call wasm-bindgen imported functions
on non-wasm targets', /Users/eric/.cargo/registry/src/
github.com-1ecc6299db9ec823/web-sys-0.3.52/src/features/gen_HtmlImageElement.rs:4:1
```

It turns out that even though we replaced the production `browser` with `test_browser`, 
we're still trying to call browser code. 
I have already pointed out `HtmlElement` and `JsValue`, 
but this test also includes `AudioBuffer` and `AudioBufferOptions`. 

As is, this code doesn't compile without more feature flags being enabled and changes being made to engine. 

It's just too tightly coupled to the browser still.

The act of trying to use this code in a test harness demonstrated the power of coupling,
and it is often useful to take legacy code and get it into a harness to identify these
dependency problems and break them. 

Unfortunately, this is a time-consuming process that we are not going to continue using in this section, 
although it may appear on my blog at paytonrules.com at some point. 
Still waiting but See: [Mock Objects (manually) in Rust](https://paytonrules.com/post/mock-objects-in-rust/)

Instead, we'll test this code via a test that runs in the browser.


---------


```rust
// tests/app.rs



```


