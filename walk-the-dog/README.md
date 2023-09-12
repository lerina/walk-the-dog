

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
mkdir -p web/js web/css web/html web/resources/pix web/resources/sound
```

The result should look like this:

```
walk-the-dog/
├── Cargo.toml
├── README.md
├── src
│   └── lib.rs
└── web
    ├── css
    ├── html
    ├── js
    └── resources
        ├── pix
        └── sound
```
