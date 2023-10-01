# Layered architecture

        Game
          ↓
        Engine
          ↓
        Browser

The one rule of this architecture is that layers can only use things at or below their layer.
So working from the bottom, the browser layer is going to be a bunch of small functions that are specific to the browser. For instance, our window function will end up here.
Meanwhile, the engine layer is going to be tools that work across our game, such as the GameLoop structure. Finally, the game is the layer that contains our actual game logic.
Eventually, we'll spend most of our development time in this layer, although initially, we'll spend a lot of time in the Engine and Browser layers until they have settled.

Why do this? Our aforementioned rule was that any change in architecture has to make
future changes easier, so let's identify what makes changes hard right now:

• Keeping everything in one long function makes the code hard to follow.
• Extracting all the Browser code will allow us to unify error handling.

The first point reflects that our brains can only hold so much. Keeping all the code in one place means scrolling up and down trying to find where things are and trying to remember virtually all of the code. Extracting code into various constructs such as modules, functions, and structs with names lets us reduce the amount of information in our heads. This is why the right design feels good to program in. Too much abstraction and you've replaced keeping track of all the details of the program with keeping track of all the abstractions.

The second reason for the layered approach is specific to Rust and the wasm-bindgen
functions, which all return JsValue as their error type. While this works in a browser, it does not work well when intermingling with the rest of a Rust program because JsValue does not implement the std::Error::error type that most other Rust errors implement.

In the browser module, we'll map JsValues
to a standard error, using the anyhow crate. We'll also use it to hide the weird details of the API, creating one that's tailored to our purposes.

---

Add the anyhow crate, which we'll use to unify the error handling across
WebAssembly and pure Rust code.

This crate provides a few features that we'll be using extensively:
• An anyhow::Error type that conforms to the std::error::Error trait
• An anyhow! macro that lets us create error messages that conform to the type,
with strings
• An `anyhow::Result<T>` type that is a shortcut for `Result<T,anyhow::Error>`












