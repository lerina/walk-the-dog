## Adding a UI

### Showing the button with Rust

We've written HTML to show the button and it looks pretty good, but we'll actually need
to show it and hide it on command. This means interacting with the browser and using
the browser module. We haven't done this in a while, so let's refresh our memory on
how we translate from the JavaScript we'd write traditionally to the Rust with `web-sys`
that we'll be using. First, we'll need code to insert the button into the ui div. There are
lots of ways to do this; we'll use `insertAdjacentHTML` so that we can just send a string
from our code to the screen. In JavaScript, that looks like this:

```javascript

let ui = document.getElementById("ui");
ui.insertAdjacentHTML("afterbegin", "<button>New Game</button>");
```

Note:
You can find the docs for this function at [insertAdjacentHTML](https://developer.mozilla.org/en-US/docs/Web/API/Element/insertAdjacentHTML)

We spent a lot of time translating this kind of code into Rust in Chapter 2, Drawing Sprites,
and Chapter 3, Creating a Game Loop, but let's refresh our memory and appease any monsters
who read books out of order. 

> Any JavaScript function or method is likely to be found in the `web-sys` crate with the name converted from `PascalCase` to snake_case, and with most of the functions returning Option . 

Frequently, you can just try that out, and it will work. 
Let's create a new function in browser and see whether that's the case, as shown here:

```rust
// src/browser.rs

pub fn draw_ui(html: &str) -> Result<()> {
    document()
        .and_then(|doc| {
            doc.get_element_by_id("ui")
                .ok_or_else(|| anyhow!("UI element not found"))
        })
        .and_then(|ui| {
            ui.insert_adjacent_html("afterbegin", html)
                .map_err(|err| anyhow!("Could not insert html {:#?}", err))
    })
}//^-- fn draw_ui
```

This `draw_ui` function assumes there is a div with the ui ID, just as the canvas
function assumes an ID of canvas. 

This means it's not incredibly generic, but we don't need a more complex solution right now. 
If we do later, we'll write more functions. 
As always, we don't want to go too far with some idea of "perfect" code because we've got a
game to finish.

Once again, the Rust version of the code is much longer, using and_then and mapping
errors to make sure we handle the error cases instead of just crashing or halting the
program as JavaScript would. 
This is another case where code is aesthetically less pleasing in Rust but, in my opinion, 
better because it highlights the possible causes of an error. 
The other function we'll need right away is used to hide the ui element, which looks like this
in JavaScript:

```javascript

let ui = document.getElementById("ui");
let firstChild = ui.firstChild;
ui.removeChild(firstChild);
```

This function grabs the first child of the ui div and removes it with the `removeChild`
method. To be completely thorough, we should loop through all the ui children and
make sure they all get removed, but we don't do that here because we already know there's
only one. We also remove the children (and don't just set their visibility to hidden) so that
they do not affect the layout, and any event listeners are removed. 

Once again, you'll want to translate JavaScript to Rust. 
In this case, `firstChild` becomes the `first_child` method 
and `removeChild` becomes `remove_child` , as shown here:

```rust
// src/browser.rs

pub fn hide_ui() -> Result<()> {
    let ui = document().and_then(|doc| {
        doc.get_element_by_id("ui")
            .ok_or_else(|| anyhow!("UI element not found"))
        })?;

    if let Some(child) = ui.first_child() {
        ui.remove_child(&child)
            .map(|_removed_child| ())
            .map_err(|err| anyhow!("Failed to remove child {:#?}", err))
    } else {
        Ok(())
    }
}//^-- fn hide_ui

```

This function is a little different than `draw_ui`, in part because `first_child` being
missing isn't an error; it just means you called `hide_ui` on an empty `UI`, and we don't
want that to error. 
That's why we use the `if let` construct and just return an `Ok(())` explicitly if it isn't present. 

The ui div was already empty, so it's fine. 

In addition, there's that weird call to `map(|_removed_child| ())`, 
which we call because `remove_child` returns the `Element` being removed. 
We don't care about it here, so we are, once again, explicitly mapping it to our expected value of unit. 

Finally, of course, we address the error with `anyhow!`.

This function reveals some duplication, so let's go ahead and refactor it out in the final
version, as follows:

We extract the duplicated code into a new function:

```rust
// src/browser.rs
use web_sys::Element;
...

fn find_ui() -> Result<Element> {
    document().and_then(|doc| {
        doc.get_element_by_id("ui")
           .ok_or_else(|| anyhow!("UI element not found"))
    })
}
```

Then use it in the previous code:

```rust
// src/browser.rs


pub fn draw_ui(html: &str) -> Result<()> {
    find_ui()?
        .insert_adjacent_html("afterbegin", html)
        .map_err(|err| anyhow!("Could not insert html {:#?}", err))
}//⁻- fn draw_ui

pub fn hide_ui() -> Result<()> {
    let ui = find_ui()?;

    if let Some(child) = ui.first_child() {
        ui.remove_child(&child)
            .map(|_removed_child| ())
            .map_err(|err| anyhow!("Failed to remove child {:#?}", err))
    } else {
        Ok(())
    }
}//^-- fn hide_ui

```

Here, we've replaced both of the repetitive `document().and_then` calls with calls to
`find_ui`, which is a private function that ensures we always get the same error when UI
isn't found. 
It streamlines a little bit of code and makes it possible to use the try `?` operator
in `draw_ui`. The `find_ui` function returns `Element`, so you need to make sure to
bring into scope `web_sys::Element`.

We've got the tools we need to draw the button set up in browser. 

To show our button programmatically, 
we can just call `browser::draw_ui("<button>New Game</button>")`. 

That's great, but we can't actually handle doing anything on the button click yet. 
We have two choices. 

The first is to create the button with an `onclick` handler 
such as `browser::draw_ui("<button onclick='myfunc'>New Game</button>")`. 
This will require taking a function in our Rust package and exposing it to the browser. 
It would also require some sort of global variable that the function could operate on. 
If myfunc is going to operate on the game state, then it needs access to the game state. 
We could use something such as an event queue here, and that's a viable approach, 
but it's not what we'll be doing.

What we're going to do instead is set the `onclick` variable in Rust code, via the
`web-sys` library, to a closure that writes to a channel. 
Other code can listen to this channel and see whether a click event has happened. 
This code will be very similar to the code we wrote in Chapter 3, Creating a Game Loop, 
for handling keyboard input.

We'll start with a function in the `engine` module that takes `HtmlElement` 
and returns `UnboundedReceiver`, as shown here:


```rust
// src/engine.rs
...
use web_sys::HtmlElement
...

pub fn add_click_handler(elem: HtmlElement) -> UnboundedReceiver<()> {
    let (click_sender, click_receiver) = unbounded();
    
    click_receiver
}

```

Don't forget to bring `HtmlElement` into scope with `use web_sys::HtmlElement`.
This doesn't do much, and it sure doesn't seem to have anything to do with a click, 
and it's not obvious why we need an `UnboundedReceiver`. 
When we add a click handler to the button, 
we don't want to have to move anything about the game into the closure. 
Using a channel here lets us encapsulate the handling of the click 
and separate it from the reacting to click event. 

Let's continue by creating the `on_click` handler, as shown here:

```rust
// src/engine.rs

pub fn add_click_handler(elem: HtmlElement) -> UnboundedReceiver<()> {
    //let (click_sender, click_receiver) = unbounded();
    let (mut click_sender, click_receiver) = unbounded();
    
    let on_click = browser::closure_wrap(Box::new(move || {
                        click_sender.start_send(());
                   }) as Box<dyn FnMut()>);

    click_receiver
}
```

The changes we've made are to make `click_sender` mutable and then move it into
the newly created closure called `on_click`. You may remember `closure_wrap` from
the earlier chapters, which needs to take a heap-allocated closure, in other words a Box,
which, in this case, will be passed a mouse event that we're not using so we can safely
skip it. 

The casting to `Box<dyn FnMut()>` is necessary to appease the compiler and
allow this function to be converted into `WasmClosure`. 

Inside that, we call the sender's `start_send` function and pass it a unit `()`. 
Since we're not using any other parameters, we can just have the receiver check for any event.

Finally, we'll need to take this closure and assign it to the `on_click` method on `elem` so
that the button actually handles it, which looks as follows:

```rust
// src/engine.rs

pub fn add_click_handler(elem: HtmlElement) -> UnboundedReceiver<()> {
    let (mut click_sender, click_receiver) = unbounded();
    
    let on_click = browser::closure_wrap(Box::new(move || {
                        click_sender.start_send(());
                   }) as Box<dyn FnMut()>);

    elem.set_onclick(Some(on_click.as_ref().unchecked_ref()));
    on_click.forget();
    
    click_receiver
}
```
We've added the call to `elem.set_onclick`, 
which corresponds to `elem.onclick =` in JavaScript. 
Note how we pass `set_onclick` a `Some` variant because `onclick` itself can be null 
or undefined in JavaScript and, therefore, can be `None` in Rust and is an `Option` type. 

We then pass it `on_click.as_ref().unchecked_ref()`, which is the pattern 
we've used several times to turn Closure into a function that `web-sys` can use.

Finally, we also make sure to forget the `on_click` handler. Without this, when we
actually make this callback, the program will crash because `on_click` hasn't been
properly handed off to JavaScript. We've done this a few times, so I won't belabour the
point here. 
Now that we've written all the code, we'll need to show a button and handle the
response to it, and we need to integrate it into our game. 

Let's figure out how to show the button.




---------

```rust
// src/game.rs



```





