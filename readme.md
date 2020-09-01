[![Netlify Status](https://api.netlify.com/api/v1/badges/866b604d-41ee-4ca9-baa0-da68c2b6eb24/deploy-status)](https://app.netlify.com/sites/wonderful-jones-6a4b3c/deploys)

A clone of the excellent [https://github.com/fogleman/Quads](https://github.com/fogleman/Quads) written
in Rust and compiled to WebAssembly.

[Play with it](https://quadtree.ldirer.com)


## Usage

    # building the server-side cli
    cargo build --release --bin server --features="build-binary"
    
    # sample usage
    ./target/release/server data/220_supertramp.png 10000

    # building as wasm
    wasm-pack build --target web --out-dir wasm
    
    # once wasm has been built the 'app' can be served with a http server
    python -m http.server --bind 127.0.0.1 8000


## Deploy

Note for self: Unfortunately, I did not find a **simple** way to automate the build with netlify/github actions.   
So I'm checking in the results of the wasm-pack build process, which means netlify directly deploys the static files in this repo.
