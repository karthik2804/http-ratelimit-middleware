cargo component build --release
wasm-tools compose ../ratelimit/target/wasm32-wasi/release/ratelimit.wasm -d target/wasm32-wasi/release/example.wasm -o service.wasm