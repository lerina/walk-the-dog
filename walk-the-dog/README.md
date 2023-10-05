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


---

Almost every function that calls into JavaScript will return a JsValue type, because JavaScript is a dynamically typed language. 
We know that the element returned by `get_element_by_id` will return `HtmlCanvasElement` , at least if we've retrieved the right JavaScript node, 
so we can convert it from `JsValue` to the correct element. 
This is what `dyn_into` does – it converts from `JsValue` to appropriate Rust types. 
In order to use `dyn_into` , 
you must bring into scope (import) `wasm_bindgen::JsCast`

```rust
pub fn context() -> Result<CanvasRenderingContext2d> {
    canvas()?
        .get_context("2d")
        .map_err(|js_value| anyhow!("Error getting 2d context {:#?}", js_value))?
        .ok_or_else(|| anyhow!("No 2d context found"))?
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .map_err(|element| {
            anyhow!( "Error converting {:#?} to CanvasRenderingContext2d", element)
        })
}
```

We can add a function for spawn_local

Tip
If you are writing a wrapper like this and aren't sure what the signature should
be, start by looking at the function you're wrapping and mimic its signature.

```rust
pub fn spawn_local<F>(future: F) 
    where F: Future<Output = ()> + 'static, {
        wasm_bindgen_futures::spawn_local(future);
}
```

---

## Creating a game loop

1. Any program running in the browser must give up control to the browser itself. 
We need to use the requestAnimationFrame function.

2. We'll need to make sure we can account for the differences between machine speeds in our loop, 
as best we can. We'll do that with what's called a fixed-step game loop.

### RequestAnimationFrame

We'll start with the requestAnimationFrame function, which is a browser function
that "requests" a new frame draw as soon as possible. The browser then fits that in frame
draw in between handling things such as mouse clicks, operating system events, ...

Writing a game loop in Rust is a little weird because of the borrowing guarantees, 
so let's start by writing a very basic one.

You can start by adding a simple wrapper for requestAnimationFrame to browser ,
as shown in the following code:

```rust
pub fn request_animation_frame(callback: &Function) -> Result<i32> {
    window()?
        .request_animation_frame(callback)
        .map_err(|err| anyhow!("Cannot request animation frame {:#?}", err))
} 
```

The `Function` type is a pure JavaScript type and is only available in the js-sys crate.

We don't actually have to use the Function type directly if we make a small change 
to the function signature and the implementation:

```rust
pub fn request_animation_frame(callback: &Closure<dyn FnMut(f64)>) -> Result<i32> {
    window()?
        .request_animation_frame(callback.as_ref().unchecked_ref())
        .map_err(|err| anyhow!("Cannot request animation frame {:#?}", err))
}
```

Instead of taking &Function , our request_animation_frame will take
&Closure<dyn FnMut(f64)> as its parameter. Then, it will call callback.
as_ref().unchecked_ref() when calling the web-sys version of request_
animation_frame . This converts Closure into Function , without requiring an
explicit dependency on the Function type, and it's worth thinking about when you're
creating your own versions of these functions. The makers of web-sys have to match
every single potential use case, and as such, they are going to create the widest possible
interfaces. As an application programmer, you do not need most of what's in that library.
Therefore, you can and should narrow the interface to your own use cases, making it
easier for you to work with.


In order to make things a little cleaner, we'll convert that into a type, with one small change:

```rust
pub type LoopClosure = Closure<dyn FnMut(f64)>;

pub fn request_animation_frame(callback: &LoopClosure) -> Result<i32> {
// ...
```

You might think you can now write a simple game loop, like so:

```
pub fn animate(perf: f64) {
browser::request_animation_frame(animate);
}
```

Alas we need to pass a JavaScript Closure, not a Rust fn .
Using the `Closure::once` that we used before won't work because this closure will
be called more than once, but fortunately, there's `Closure::wrap` , which will do just
that. 
We'll create a function in browser to create a Closure specific 
to the `request_animation_frame` function, called `create_raf_closure` :

```rust
pub fn create_raf_closure(f: impl FnMut(f64) + 'static) -> LoopClosure {
    closure_wrap(Box::new(f))
}
```

The function being passed in has a 'static lifetime. Any `FnMut` passed into this
function cannot have any non-static references. This wasn't a decision on my part; it's a
requirement of the Closure::wrap function we'll be calling into.




Let's wrap `Closure::wrap` in a `closure_wrap` function so that the code we just added will compile, 
which looks like the following:

```rust
pub fn closure_wrap<T: WasmClosure + ?Sized>(data: Box<T>) -> Closure<T> {
    Closure::wrap(data)
}
```

This is another one of those wrapper functions where we are just matching the same
signature as the function being wrapped – Closure::wrap . 
Because the wrap function on Closure creates a Closure that can be called multiple times, 
it needs to be wrapped in a Box and stored on the heap.


Consider this:

```rust
// not working
let animate = create_raf_closure(move |perf| {
                  request_animation_frame(animate);
              });
            
request_animation_frame(animate);
```

The Closure we pass to
create_raf_closure has to have a 'static lifetime, meaning everything that the
Closure references must be owned by the closure. That's not the case right now. The
animate variable is owned by the current scope and will be destroyed when that scope
completes. Of course, animate is itself the Closure because this is a self-referencing
data structure. The animate variable is the Closure but is also referenced inside the
Closure . This is a classic Rust problem because the borrow checker cannot allow it.

Let's have another crack at a hypothetical loop:

```rust
let f = Rc<RefCell<Option<LoopClosure>>> = Rc::new(RefCell::new(None));
let g = f.clone();
let animate = Some(create_raf_closure(move |perf: f64| {
                        request_animation_frame(f.borrow().as_ref().unwrap());
              });
            
*g.borrow_mut() = animate;
request_animation_frame(g.borrow().as_ref().unwrap());
```

What we're doing is creating two references to the same place in memory, 
using Rc struct , allowing us to both take f and g and point them 
at the same thing but also move f into animate Closure . 

The other trick is that they both point to Option so that we can move f into Closure 
before it is completely defined. 

Finally, when we assign to g the Closure with *g.borrow_mut() = animate , 
we also assign to f because they are pointing to the same place.

Let's go through the types really quickly to reiterate what we did. 
f is set to the following:

• Rc to create a reference-counted pointer
• RefCell to allow for interior mutability
• Option to allow us to assign f to None
• LoopClosure to hold a mutable Closure that matches the request_animation_frame parameter

g is then set to a clone of f so that they point to the same thing, and f is moved into
animate Closure . g is assigned to animate via the dereference * operator and borrow_mut functions.

Because f points to the same place as g , it will also contain animate Closure . 
Finally, we can call request_animation_frame , both outside and inside Closure , 
by borrowing it, converting it to a reference, and calling unwrap to actually get the real Closure . 
Yes, unwrap is back; we'll deal with one of those when we create our real function. 

Finally, g can be destroyed when it leaves scope because f is still in Closure and will keep the memory around.

[Source: wasm-bindgen request_animation_frame](https://rustwasm.github.io/docs/wasm-bindgen/examples/request-animation-frame.html)


## A game trait

To write our game loop, We'll create a start function that accepts anything that implements the Game trait. 
The Game trait will start with two functions, update and draw . We'll run that through our game loop 
to first update and then draw our scene. All of this will go into the engine module;

```rust
pub trait Game {
    fn update(&mut self);
    fn draw(&self, context: &CanvasRenderingContext2d);
}
```

Note how the draw function takes CanvasRenderingContext2d as
a parameter. Now for the rest of the loop – you can add this after the Game trait or load_image; 
it doesn't really matter as long as it's in the engine module

```rust
pub struct GameLoop;
type SharedLoopClosure = Rc<RefCell<Option<LoopClosure>>>;

impl GameLoop {
    pub async fn start(mut game: impl Game + 'static) -> Result<()> {
        let f: SharedLoopClosure = Rc::new(RefCell::new(None));

        let g = f.clone();
        *g.borrow_mut() = Some(browser::create_raf_closure(move |perf: f64| {
            game.update();
            game.draw(&browser::context().expect("Context should exist",));
            
            browser::request_animation_frame(f.borrow().as_ref().unwrap());
        }));

        browser::request_animation_frame(g.borrow().as_ref().ok_or_else(|| {
            anyhow!("GameLoop: Loop is None")
        })?)?;

        Ok(())
    }//^-- fn start

}//-- GameLoop
```


We're going to create
a GameLoop struct with no data and add a SharedLoopClosure type to simplify
the type of the f and g variables. Then, we'll add an implementation of GameLoop
with one method, start , that takes the Game trait as a parameter. Note that the trait is
'static because anything moved into the "raf " closure has to be 'static . 

We follow the snippets we used before to set up our request_animation_frame loop, 
and the key change is on the inside where we update and then draw, 
passing the draw function CanvasRenderingContext2d .

There's a problem with this kind of naive game loop. Typically, request_animation_
frame runs at 60 frames per second, but if either update or draw takes longer than
1/60th of a second, it will slow down, making the game move more slowly

Since we want a consistent experience across processor speeds, we'll take
a common approach called "fixing" the time step.






