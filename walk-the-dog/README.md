
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
    const wasm = await init().catch(console.error);
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


