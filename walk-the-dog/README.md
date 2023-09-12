

## Initialize the project

### The Rust side 

```bash
cargo new --lib walk-the-dog
```

## Install wasm-pack

On Linux and macOS systems wasm-pack is installed with a simple cURL script:

```bash 
curl https://rustwasm.github.io/wasm-pack/installer/init.
sh -sSf | sh
```
Windows users have a separate installer that can be found at 
[https://rustwasm.github.io](https://rustwasm.github.io) .

### The web side

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

### The script to simplify development

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

### Update Cargo.toml 

```
...
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

...
```
