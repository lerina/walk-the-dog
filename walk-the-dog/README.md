## Start a new Game

If you remember our originally planned behavior, and I don't blame you if you don't, 
we wanted to draw a new game button on the screen when `RHB` crashed and fell over. 
Then, when it's clicked, we want to start a new game.
 
For that to happen, we'll need to do the following:

1. Check whether `RedHatBoyStateMachine` is `KnockedOut`, and if so, transition from `Walking` to `GameOver`.
2. On that transition, draw the new game button.
3. Add an `onclick` handler so that when the button is clicked, we transition back to
`Ready` with a new `Walk` instance.
4. On the transition to `Ready`, hide the button and restart the game.


All the code we wrote before was to make that change easier. Let's see whether we were right about that:

### 1. Transition from `Walking` to `GameOver`.

To transition from `Walking` to `GameOver`, we need to return the `GameOver` state 
from the `WalkTheDogState<Walking>` `update` method, but when should we do that? 

We'll need to see whether the boy is knocked out and then make the change. 

We don't have that capability yet, so we'll need to create it, and let's work top-down, 
as we have been this entire chapter. 

First, we'll change the `WalkTheDogState<Walking>` `update` method to check the non-existing method:



```rust
// src/game.rs


impl WalkTheDogState<Walking> {
                                                 // WalkTheDogState<Walking> {   
    fn update(mut self, keystate: &KeyState) ->  WalkingEndState { 
        ...
            self.walk.timeline += walking_speed;
        }

        // self
        if self.walk.knocked_out() {
            WalkingEndState::Complete(self.end_game())
        } else {
            WalkingEndState::Continue(self)
        }
    }//^-- fn update
```

Now, instead of always returning the `Walking` state, we return `WalkingEndState`,
which doesn't exist yet but will mimic the pattern we used in the update method on `WalkTheDogState<Ready>`.
 
When the current state is `knocked_out`, we will return the `Complete` variant holding 
an instance of the `WalkTheDogState<GameOver>` type.
 
That will be the state returned from the `end_game` transition, which is also not written yet.
Otherwise, we'll return `Continue` with the current `WalkTheDogState<Walking>` state as its field.

That's two functions that don't exist yet, `knocked_out` and `end_game`, along with a brand-new type. 

You can create the `WalkingEndState` type and its corresponding `From` trait to convert it into `WalkTheDogStateMachine` right now by following the same pattern we did for `ReadyEndState`. 

```rust
// src/game.rs

    ...
}//^-- impl WalkTheDogState<Walking> 

enum WalkingEndState {
    Continue(WalkTheDogState<Walking>),
    Complete(WalkTheDogState<GameOver>),
}

impl From<WalkingEndState> for WalkTheDogStateMachine {
    fn from(state: WalkingEndState) -> Self {
        match state {
            WalkingEndState::Continue(walking) => walking.into(),
            WalkingEndState::Complete(game_over) => game_over.into(),
        }
    }
}

```

We'll proceed from there by getting `knocked_out` working, which is going to be delegated from `Walk` 
to `RedHatBoyStateMachine` with some delegations in between:

In `Walk`

```rust
// src/game.rs

impl Walk {
    fn knocked_out(&self) -> bool {
        self.boy.knocked_out()
    }
    ...
}

```

And `impl RedHatBoy`

```rust
// src/game.rs

impl RedHatBoy {
    ...
    fn knock_out(&mut self) {
        self.state_machine = self.state_machine.clone().transition(Event::KnockOut);
    }

    fn knocked_out(&self) -> bool {
        self.state_machine.knocked_out()
    }
...
}
```

And `impl RedHatBoyStateMachine`

```rust
// src/game.rs


impl RedHatBoyStateMachine {
    ...
    fn knocked_out(&self) -> bool {
        matches!(self, RedHatBoyStateMachine::KnockedOut(_))
    }
}
```

We could pass `WalkTheDogState` to `RedHatBoyStateMachine` here to get the new state 
and follow the OO guideline of "tell, don't ask", but sometimes,
you just want to check a Boolean. 

Here, we ask the `Walking` state, which asks `RedHatBoy` and finally `RedHatBoyStateMachine` 
whether it is knocked out. `RedHatBoyStateMachine` uses the handy `matches!` macro to 
check `self` against an enum variant, and return whether or not they match. 

Now that we can check whether Red Hat Boy is knocked out, 
we have just one compiler error – no method named `end_game` found for struct `WalkTheDogState`.

It's time to implement the `end_game` transition method, 
which will represent our transition. 

We can start by implementing the `transition` to do nothing other 
than move `walk` from `Walking` to `GameOver`, as shown here:


```rust
// src/game.rs

impl WalkTheDogState<Walking> {
    fn end_game(self) -> WalkTheDogState<GameOver> {
        WalkTheDogState {
            _state: GameOver,
            walk: self.walk,
        }
    }

    fn update(mut self, keystate: &KeyState) -> WalkTheDogState<Walking> {
        ...

```

This returns us to a compiled state and means that when RHB crashes and is
knocked out, the game is in the `GameOver` state. 
However, it does nothing, so it's time for step 2 – draw the new game button.



### 2. Draw the new game button.

Many pages ago, I said: 
"To show our button programmatically, we can just call
`browser::draw_ui("<button>New Game</button>")`." 
But when do we call it? Well, we call it now, right before creating the new state:



```rust
// src/game.rs

impl WalkTheDogState<Walking> {
    fn end_game(self) -> WalkTheDogState<GameOver> {
        browser::draw_ui("<button>New Game</button>");

        WalkTheDogState {
            _state: GameOver,
            walk: self.walk,
        }
    }
    ...

```

If you add this one line of code to the transition, you'll see the new game button
we wrote way back at the beginning when our RHB crashes into a rock. 
There's a warning on this line because we don't handle the result of draw_ui, 
which we'll ignore for the moment.

### 3. Add the onclick handler to the button.

In order to add the click handler to the button, we need to get a reference to the
element we just drew. We don't have that, as the `insert_adjacent_html` function 
doesn't provide it, so we'll need to find the button we just added to the screen 
so that we can attach an event handler to it. 

We've used `get_element_by_id` twice before on document, 
so it's probably time to write a wrapper function in the `browser` module, as shown here:

```rust
// src/browser.rs

pub fn find_html_element_by_id(id: &str) -> Result<HtmlElement> {

    document()
        .and_then(|doc| {
            doc.get_element_by_id(id)
               .ok_or_else(|| anyhow!("Element with
            id {} not found", id))
        })
        .and_then(|element| {
            element
                .dyn_into::<HtmlElement>()
                .map_err(|err| anyhow!("Could not cast into HtmlElement {:#?}", err))
        })
}

```

We've made a slight change to the way we've been finding elements in this function.

Normally, we want `HtmlElement`, not a generic Element type, so in this function, 
we've gone ahead and added a call to `dyn_into` to make the conversion.

Therefore, this function first gets the document,   
then gets the element,   
and finally, converts it into the `HtmlElement` type,   
all while normalizing the errors with `anyhow!`.  

Don't forget 

```rust
// src/browser.rs


use web_sys::HtmlElement;
...

```

Now that we have a way to find the element, we can return to the transition in
game, find the newly added new game button, and then add a click handler to it, 
as shown in the following code:



```rust
// src/game.rs

impl WalkTheDogState<Walking> {
    fn end_game(self) -> WalkTheDogState<GameOver> {
        let receiver = browser::draw_ui("<button id='new_game'>New Game</button>")
                            .and_then(|_unit| browser::find_html_element_by_id("new_game"))
                            .map(|element| engine::add_click_handler(element))
                            .unwrap();

        WalkTheDogState {
            _state: GameOver,
            walk: self.walk,
        }
    }

```

We've reproduced the entire transition trait here, but there are three changes. 

The first is that we've added `id` to the new game button; naturally, that's new_game.
`browser::draw_ui("<button id='new_game'>New Game</button>")`

Then, we find the element in the document in the `and_then` block 
`.and_then(|_unit| browser::find_html_element_by_id("new_game"))`

and use map to take that element and pass it to the recently created `add_click_handler` function. 
`.map(|element| engine::add_click_handler(element))`

Now, we've got a small problem. 
We will need receiver to get click messages when they happen, 
but the `add_click_handler` function returns Result with `UnboundedReceiver`. 
The challenge is that `the end_game` function doesn't return `Result`. 

In Chapter 9, Testing, Debugging, and Performance, we'll investigate how to debug 
this kind of condition, but for now, we'll just grit our teeth and add `unwrap`.


Now that we have receiver that will get a message whenever the player clicks
New Game, we need to do something with it. 

We'll need to check it in the `update` function for the `GameOver` state 
and when we receive the event transition to the `Ready` state. 

That's going to mean adding the `receiver` to the `GameOver struct`, as follows:

```rust
// src/game.rs

struct GameOver {
    new_game_event: UnboundedReceiver<()>,
}

```

This will prompt you to add the use declaration for `futures::channel::mpsc::UnboundedReceiver`. 

```rust
// src/game.rs
...
use futures::channel::mpsc::UnboundedReceiver;
...
```

Now that `GameOver struct` has the field, 
we'll need to pass it along in the transition, as shown here:


```rust
// src/game.rs

impl WalkTheDogState<Walking> {

    ...
    fn end_game(self) -> WalkTheDogState<GameOver> {
        let receiver = browser::draw_ui("<button id='new_game'>New Game</button>")
            ...

            WalkTheDogState {
            //_state: GameOver,
            _state: GameOver { new_game_event: receiver,},
            walk: self.walk,
        }
    }

```
This is the final change to this method, and it's just adding the field to `GameOver`.
Interestingly it's the first time we've added a field to any of our state structures, but
it's something you're likely to do more of over time as you extend this game. 

Various states have data that's unique to them, and they belong in the state struct.

It's time to return to the `WalkTheDogState<GameOver>` implementation and
its `update` method, which currently just returns the `GameOver` state, leaving the
game in that state forever. 

Instead, we'll want to check whether the new game event has happened (because the button was clicked) 
and then return the `Ready` state to start over again. 
That small bit of code is reproduced here:


```rust
// src/game.rs

impl WalkTheDogState<GameOver> {
    fn update(mut self) -> GameOverEndState {
        if self._state.new_game_pressed() {
            GameOverEndState::Complete(self.new_game())
        } else {
            GameOverEndState::Continue(self)
        }
    }
}

```
And

```rust
// src/game.rs

impl GameOver {
    fn new_game_pressed(&mut self) -> bool {
        matches!(self.new_game_event.try_next(), Ok(Some(())))
    }
}

```

In the `WalkTheDogState<GameOver>` implementation, we check the state
to see whether the new game button has been pressed, and if it has, we return
the `GameOverEndState::Complete` variant; otherwise, we return the
`GameOverEndState::Continue` variant. 

This is the same pattern we've used in every other update method, 
and you can go ahead and reproduce the `GameOverEndState enum` 
and its corresponding `From trait` to convert the type to a `WalkTheDogStateMachine enum`. 

That code is not reproduced here, but remember that if you get stuck, 
you can find the sample code at https://
github.com/PacktPublishing/Game-Development-with-Rust-and-
WebAssembly/tree/chapter_8 .
```rust
// src/game.rs

impl GameOver {
    ...
}
...

enum GameOverEndState {
    Continue(WalkTheDogState<GameOver>),
    Complete(WalkTheDogState<Ready>),
}

impl From<GameOverEndState> for WalkTheDogStateMachine {
    fn from(state: GameOverEndState) -> Self {
        match state {
            GameOverEndState::Continue(game_over) => game_over.into(),
            GameOverEndState::Complete(ready) => ready.into(),
        }
    }
}

```


In the `GameOver` implementation, we have the details to check whether 
`new_game_event`, corresponding to the player's click, has happened. 
Calling `try_next` will return `Result` immediately, 
without blocking, or `Ok` if the channel is still open, 
regardless of whether anything is in it. 

Remember that we are running at 60 frames per second and cannot use the blocking calls. 

Finally, we use the handy `matches!` macro to check whether the channel was successfully 
sent a message of unit, or `Ok(Some(()))`. 

If the event is there, New Game has been pressed, and the function returns true.

This code doesn't compile because we don't have a transition written from
`GameOver` to `Ready`, which is what we'll write in the next step.


### 4. Restart the game on New Game.

Restarting the game will mean doing two things on the `new_game` transition. 
The first is hiding the button or "UI,"  
and the second is recreating `Walk` from scratch.

The first is actually easier, so we'll start with that:

```rust
// src/game.rs

impl WalkTheDogState<GameOver> {
    ...
    fn new_game(self) -> WalkTheDogState<Ready> {
        browser::hide_ui();

        WalkTheDogState {
            _state: Ready,
            walk: self.walk,
        }
    }
}

```

This is another transition, this time from `GameOver` to `Ready`, with the side effect
of hiding the UI. 
It then moves to a new state with the same walk we ended with, which is not quite what we want.

The button is hidden but `RHB` is still knocked out. 

Moving from `GameOver` to `Ready` means creating a new `Walk` instance from the old one, 
so the game starts over. 

This is a bit of a challenge because we no longer have access to the various
images and sprite sheets we used to create `Walk` and `RedHatBoy` in the first place.
What we'll do is clone those from an existing one, via a **constructor** function on
the `Walk` implementation. 

We won't call this clone because that term means an identical copy, 
whereas this is really a reset. 
You can see the implementation here:

```rust
// src/game.rs

impl Walk {
    fn reset(walk: Self) -> Self {
        let starting_obstacles = stone_and_platform(
                                    walk.stone.clone(),
                                    walk.obstacle_sheet.clone(), 0);
        let timeline = rightmost(&starting_obstacles);

        Walk {
            boy: walk.boy,
            backgrounds: walk.backgrounds,
            obstacles: starting_obstacles,
            obstacle_sheet: walk.obstacle_sheet,
            stone: walk.stone,
            timeline,
        }
    }
    ...
}

```

The `reset` function consumes `Walk` and returns a new one. 
It recreates `starting_obstacles` the same way they are created in `initialize`, 
and then recalculates timeline. 
Then, it constructs a new `Walk`, moving all the values from `Walk` 
except `starting_obstacles` and `timeline`. 

This function is not quite right though, as it will reset `Walk` 
but leave `boy` in its `KnockedOut` state. 

We'll need a similar `reset` function for `boy`, as shown here:

```rust
// src/game.rs

impl RedHatBoy {
    fn new(sprite_sheet: Sheet, image: HtmlImageElement, audio: Audio, sound: Sound) -> Self {
        ...
    }

    fn reset(boy: Self) -> Self {
        RedHatBoy::new(
            boy.sprite_sheet,
            boy.image,
            boy.state_machine.context().audio.clone(),
            boy.state_machine.context().jump_sound.
            clone(),
        )
    }

    ...

```

Writing `reset` on `RedHatBoy` is a lot easier than it was on `Walk` because we
created a **constructor** function, `new`, for `RedHatBoy` a long time ago. 

We should do the same for `Walk`, but that refactoring is up to you. 

Keep in mind that for this to compile, the `audio` and `jump_sound` fields 
on `RedHatBoyContext` need to be public.


```rust
// src/game.rs

    #[derive(Clone)]
    pub struct RedHatBoyContext {
        pub frame: u8,
        pub position: Point,
        pub velocity: Point,

        pub audio: Audio,
        pub jump_sound: Sound,
    }
```


Now that we have a reset function for RedHatBoy , we can use it in the Walk
reset function, like so:


```rust
// src/game.rs

impl Walk {
    ...
    fn reset(walk: Self) -> Self {
        ...
        Walk {
            boy: RedHatBoy::reset(walk.boy), // walk.boy,
            ...
        }
    }
}

```

We also need to call this in the original transition 
from `GameOver` to `Ready`, as follows:

```rust
// src/game.rs

impl WalkTheDogState<GameOver> {
    ...
    fn new_game(self) -> WalkTheDogState<Ready> {
        browser::hide_ui();
        
        WalkTheDogState {
            _state: Ready,
            walk: Walk::reset(self.walk), //self.walk,
        }
    }
}

```

If you do all that, you'll find that when you click the new game button, 
the game resets and the player is back at the start. 

You should be able to hit the right arrow key and start walking again. 
You should, but it doesn't work because we haven't accounted 
for one feature of the UI – the focus.

### 5. Focus!

It turns out there's one more thing to do when we click the new game button to
make the game ready to play again. 

When the game was started, we set up the canvas to have the focus 
so that it would receive keyboard input. 

We did this with the `tabIndex` field in the original HTML. 
When the player clicks `New Game`, they transfer the focus to the button and then hide the button, 
which means nothing will get the keyboard events we are listening to. 

You can see this effect by clicking New Game and then clicking the canvas after the button disappears. 

If you click the `canvas`, it regains the focus, and you can play the game again.

We can transfer the focus back to the canvas automatically in the `hide_ui` function 
of the `browser` module. 

It's debatable whether it belongs here, since you may have cases where you want to hide the UI 
but not reset the focus, but our game doesn't have that case, so I think we're safe. 

This change is here:

```rust
// src/browser.rs

pub fn hide_ui() -> Result<()> {
    let ui = find_ui()?;

    if let Some(child) = ui.first_child() {
        ui.remove_child(&child)
            .map(|_removed_child| ())
            .map_err(|err| anyhow!("Failed to remove child {:#?}", err))

            .and_then(|_unit| {
                canvas()?
                    .focus()
                    .map_err(|err| anyhow!("Could not set focus to canvas! {:#?}", err))
            })
    } else {
        Ok(())
    }
}//^-- fn hide_ui


```

After the first call to `map_err` for removing the child, we've added a second 
`and_then` call, which takes unit from the earlier map call, promptly ignores it, 
and then requests `focus` on `canvas`. 

The error from the focus call doesn't return an anyhow! type, so the compiler complains, 
and we fix that with a `map_err` call.

The `focus` function is a JavaScript function we call through `web-sys`, 
which is documented on the MDN ( https://mzl.la/30YGOMm ).

With that change, you can click New Game and start another try. We did it!



### Pre-loading

You might notice that the button visibly loads when it shows up on screen – that is to
say that the image and font aren't downloaded to the browser yet, so it doesn't appear
instantaneously. This is standard behavior for web browsers. In order to make sure that
you don't have to wait for an entire page worth of images, fonts, and other assets to load
before you see a page, browsers will load assets lazily. 

This is so common that your eyes may not have noticed it when the New Game button appeared, 
but it doesn't look right in an interactive application. 

Fortunately, there's a quick way we can fix it. We can tell the
browser to `preload` the `Button.svg` and the `kenney_future_narrow-webfont.woff2` 
assets immediately when the page is loaded so that when the button appears, it's
instantaneous. 

Open the `index.html` file and make the changes shown here:

```rust
// www/html/index.html

<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <title>initial index</title>
  <link rel="stylesheet" href="../css/styles.css">
  <link rel="icon" href="../favicon_96x96.png">
  <link rel="preload" as="image" href="../resources/pix/Button.svg">
  <link rel="preload" as="font"  href="../resources/fonts/kenney_future_narrow-webfont.woff2">
</head>
<body>


<!--
<div id="ui" style="position: absolute">
<button>New Game</button>
</div>
-->

  <div id="ui"></div>
  <canvas id="canvas" style="outline: none" tabindex="0" height="600" width="600">
    Your browser does not support the Canvas.
  </canvas>

...

```

The `link` tag with the `preload` attribute will preload assets before rendering the page.

You'll want to minimize this behavior generally because you don't want the user to have
to wait a very long time with a blank screen, and if you were to make a very large game
with many assets, you should probably use a more flexible solution in code with a loading
screen. 

Our game is small right now, so this works perfectly well. 

With this change, the new game button not only appears but is snappy.

## Summary

You can look at the end of this chapter in two ways. 
The first might be to say, "All that for a button?", and you would have a point. 
After all, our UI is only one new game button, and while that's true, 
we actually covered quite a bit. 

We have integrated the DOM into our app via web-sys 
and have, in turn, adjusted our game to handle it. 

By utilizing the DOM, we were able to leverage the browser for behavior 
such as clicks and hovers, without having to detect where within the canvas 
the mouse was and creating clickable areas. 

You can now create far more complex UIs using tools such as CSS Grid and Flexbox, 
so if you are familiar with web development, which you've been doing for this
entire book, so you are, you'll be able to make quality UIs for your games. 

If you're looking for some place to start, 
try adding a score to this game. 
You can increment the score in the update, 
and show it at the end menu, or at the right corner during the game, 
or both!

I look forward to seeing it. With that, we will move on from new feature development 
to making sure that our current features work, and work fast. 

It's now time to start doing some testing and
debugging, so we'll dive into that in the next chapter.


---------


```rust
// src/game.rs



```


