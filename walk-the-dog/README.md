
## using catch

If you import a JS function with Result you need 
`#[wasm_bindgen(catch)]` to be annotated on the import 
(unlike exported functions, which require no extra annotation). 

This may not be necessary in the future though and it may work "as is"!.
see: wasm-bindgen [Result Type](https://rustwasm.github.io/docs/wasm-bindgen/reference/types/result.html)  
[similar issue that gave the solution](https://github.com/rustwasm/wasm-bindgen/issues/2919)  
also see: wasm-bindgen [catch](https://rustwasm.github.io/wasm-bindgen/reference/attributes/on-js-imports/catch.html)  

Because we use catch on our Js side 
```js
 const wasm = await init().catch(console.error);
```

instead of
```rust
#[wasm_bindgen(start)]
pub fn main_js() {
    ...

}
```

use  
 
```rust
#[wasm_bindgen(start, catch)]
pub fn main_js() -> Result<(), JsValue> {
    ...

    Ok(())
}
```


## Using rand with Wasm 

### in Cargo.toml

```toml
[dependencies]
...
rand = "0.8.5"
getrandom = { version = "0.2.10", features = ["js"] }
```

### rand usage

Check [The Rust Rand Book](https://rust-random.github.io/book/)

```rust
use rand::prelude::*;

fn sierpinski( ...

...
        if depth > 0 {
            let mut rng = thread_rng();

            let next_color = (
                rng.gen_range(0..255),
                rng.gen_range(0..255),
                rng.gen_range(0..255),
            );

...
```


## Rust String to DOMString in JavaScript

context.fillStyle property takes DOMString in JavaScript, 
so we'll need to do a conversion.

```rust
    let color_str = format!("rgb({}, {}, {})", color.0, color.1, color.2);
    context.set_fill_style(&wasm_bindgen::JsValue::from_str(&color_str));
```
since we already `use wasm_bindgen::prelude::*;` we can access JsValue  direclty

```rust
context.set_fill_style(&JsValue::from_str(&color_str));
```

NOTE: 
Generally, JavaScript properties are just public and you set them, 
but web-sys generates getter and setter functions. 

These generated functions frequently take JsValue objects, 
which represent an object owned by JavaScript. 

Fortunately, wasm_bindgen has factory functions for these, 
so we can create them easily and use the compiler as our guide.


## web_sys console_log

In Javascript the console.log() method outputs a message to the web console. 
The message may be a single string (with optional substitution values), 
or it may be any one or more JavaScript objects.

Rust however does not allow such polymorphism.
So we have 

web_sys::console::log_1   for pub fn log_1(data_1: &JsValue)  
web_sys::console::log_2   for pub fn log_2(data_1: &JsValue, data_2: &JsValue)  
etc ...
 
[debugging](https://github.com/rustwasm/book/blob/master/src/reference/debugging.md)

usage:

``rust
use web_sys::console;
    
    // 
    console::log_1(&"Hello using web-sys".into());

    let js: JsValue = 4.into();
    console::log_2(&"Logging arbitrary values looks like".into(), &js);

```
