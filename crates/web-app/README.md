Install `wasm-pack` with `cargo install wasm-pack`.

Build with `wasm-pack build --target web --release --out-dir assets/lib --no-pack --no-typescript`

Serve the `assets` directory, for example with `python -m http.server 8080` and open `http://localhost:8080` in your browser.
