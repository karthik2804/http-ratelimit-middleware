# http-ratelimit-middleware

This repo is an example of how to compose a middleware component with a business logic component.

## Repo structure

The `ratelimit/` directory contains an middleware that ratelimits requests using the Spin KV store.

The `example/` directory contains a Spin application which consists of one http handler which returns an HTTP response contains `Business logic executed!` in the body. In the `spin.toml` file, the component build instructions point to a `build.sh` script which builds the example component and composes it with the ratelimit component.


## Demo instructions

### Pre-requisites

- Install [cargo component v0.4.0](https://github.com/bytecodealliance/cargo-component):

```bash
cargo install --git https://github.com/bytecodealliance/cargo-component --tag v0.4.0 cargo-component --locked
```

- Install [wasm-tools](https://github.com/bytecodealliance/wasm-tools): 

```bash
cargo install --git https://github.com/bytecodealliance/wasm-tools wasm-tools --locked
```

- Install latest [Spin](https://github.com/fermyon/spin)

### Build the components and run the demo

```bash

# Build the middleware
cargo component build --manifest-path github-oauth/Cargo.toml --release

# Build and run the example
spin up --build

# Open http://127.0.0.1:3000/ in a browser
```

### Connfiguring the ratelimit component

In the `spin.toml`, there are 3 variables defines

- rate_limit_period - The period in seconds in which to enforce the count of requests
- rate_limit_count - The count per period
- header_key = The header to use as a key to ratelimit clients. Using `"spin-client-addr"` has the special effect of stripping out the port number