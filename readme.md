A clone of the excellent [https://github.com/fogleman/Quads](https://github.com/fogleman/Quads) written
in Rust and compiled to WebAssembly.


## Usage

    # building the server-side cli
    cargo build --release --bin server --features="build-binary"
    
    # sample usage
    ./target/release/server data/220_supertramp.png 10000

    # building as wasm
    wasm-pack build --target web --out-dir wasm
    
    # once wasm has been built the 'app' can be served with a http server
    python -m http.server --bind 127.0.0.1 8000