## Building

```sh
cargo build --target wasm32-unknown-unknown
```

Then run a local web server by your favourite method, e.g.:

```sh
cd htdocs && php -S localhost:8000
```

Note that this relies on symlinks for the `.wasm` and `.d` files. You can use directly copy those files from `target/wasm32-unknown-unknown/debug/` to `htdocs/` instead, if needed.

## Testing

```shell
cargo test
```

The tests are run as native code, not as WebAssembly.
