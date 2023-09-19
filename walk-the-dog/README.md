
## Install wasm-pack

On Linux and macOS systems wasm-pack is installed with a simple cURL script:

```bash 
curl https://rustwasm.github.io/wasm-pack/installer/init.
sh -sSf | sh
```
Windows users have a separate installer that can be found at 
[https://rustwasm.github.io](https://rustwasm.github.io) .

## The Rust side 

### Initialize the project


```bash
cargo new --lib walk-the-dog
```

### Update Cargo.toml 

```
...
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

...
```
and add the dependencies

```
[dependencies]
wasm-bindgen = "0.2.87"
console_error_panic_hook = "0.1.7"
web-sys= "0.3.64"

[dev-dependencies]
wasm-bindgen-test = "0.3.37"
futures = "0.3.28"
js-sys = "0.3.64"
wasm-bindgen-futures = "0.4.37"
```

### Hello world

```rust
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    console::log_1(&JsValue::from_str("Hello world!"));

    Ok(())
}
```


## The web side

#### Create the file structure

```bash
mkdir -p www/js www/css www/html www/resources/pix www/resources/sound
```

The result should look like this:

```
walk-the-dog/
├── Cargo.toml
├── README.md
├── src
│   └── lib.rs
└── www
    ├── css
    ├── html
    ├── js
    └── resources
        ├── pix
        └── sound
```

### put dummy files for now so that git picks it up

```
touch web/js/index.js web/css/style.css web/html/index.html web/resources/pix/dummy web/resources/sound/dummy
```

### Specify we are using a javascript module  

Because we are not using bundlers, we need to specify `type="module"`
like this:

```html
...
<body>
...

<script type="module" src="../js/index.js"></script>
</body>
</html>
```

### Skeleton module
Put this in the `index.js` file. Now we are good to go.

```js
import init from "../pkg/walk_the_dog.js";

async function run() {
    const wasm = await init();
    const memory = wasm.memory;

}//^--run

//-------------------
run();
```


### The build script to simplify development

We are using wasm-pack to build as a no bundle `--target web` 
and ask with `--out-dir www/pkg` for wasm-pack to put its generated `pkg` in `www` where we have all our website related things 

> wasm-pack build --target web --out-dir www/pkg

```bash
#!/bin/sh

## pre-req a web server
# cargo install http

## exit on error and  prints each executed command
set -ex

## compile for plain vanilla no javascript framework 
wasm-pack build --target web --out-dir www/pkg

## display link for easy access
echo "Serving at: http://127.0.0.1:8080/html/"

## run the web server
http -a 127.0.0.1 -p 8080 www
```


Call it `run.sh` and give it `exec` permission

```bash
chmod +x run.sh
```
## Recap Rust Side

## lib.rs

```rust
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    console::log_1(&JsValue::from_str("Hello world!"));

    Ok(())
}
```
more on [wasm_bindgen(start)](https://rustwasm.github.io/wasm-bindgen/reference/attributes/on-rust-exports/start.html)

more on [console log](https://rustwasm.github.io/docs/book/reference/debugging.html#logging-with-the-console-apis)


NOTE: 
Use `#[wasm_bindgen(start, catch)]` as in 

```rust
#[wasm_bindgen(start, catch)]
pub fn main_js() -> Result<(), JsValue> {

    ...
    Ok(())
}
```

instead of `#[wasm_bindgen(start)]`

if you plan to use catch in your Js script like we do here

```js
import init from "../pkg/walk_the_dog.js";

async function run() {
    const wasm = await init().catch(console.error);
    const memory = wasm.memory;

}//^--run

//-------------------
run();
```

SOURCE: wasm-bindgen [Result Type](https://rustwasm.github.io/docs/wasm-bindgen/reference/types/result.html)

| Note that if you import a JS function with Result you need 
| #[wasm_bindgen(catch)] to be annotated on the import 
| (unlike exported functions, which require no extra annotation). 
| This may not be necessary in the future though and it may work "as is"!.
|


also see: wasm-bindgen [catch](https://rustwasm.github.io/wasm-bindgen/reference/attributes/on-js-imports/catch.html) 

## Cargo.toml

```toml
[package]
name = "walk-the-dog"
version = "0.1.0"
categories = ["wasm"]
readme = "README.md"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
wasm-bindgen = "0.2.87"
console_error_panic_hook = "0.1.7"
#web-sys= "0.3.64"
rand = "0.8.5"
getrandom = { version = "0.2.10", features = ["js"] }


# The `web-sys` crate allows you to interact with the various browser APIs,
# like the DOM.
[dependencies.web-sys]
version = "0.3.64"
features = ["console",
           "Window",
           "Document",
           "HtmlCanvasElement",
           "CanvasRenderingContext2d",
           "Element"]

# These crates are used for running unit tests.
[dev-dependencies]
wasm-bindgen-test = "0.3.37"
futures = "0.3.28"
js-sys = "0.3.64"
wasm-bindgen-futures = "0.4.37"
```
