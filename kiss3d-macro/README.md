# kiss3d-macro

Procedural macros for the [kiss3d](https://crates.io/crates/kiss3d) crate.

## The `#[kiss3d::main]` macro

This macro simplifies writing cross-platform kiss3d applications that work on both native platforms and WebAssembly.

### Problem

When writing kiss3d applications that target both native and WASM, you previously had to write boilerplate code like this:

```rust
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    pollster::block_on(run())
}

#[cfg(target_arch = "wasm32")]
fn main() {
    wasm_bindgen_futures::spawn_local(run())
}

async fn run() {
    let mut window = Window::new("My App");
    while window.render_async().await {
        // Your render loop
    }
}
```

### Solution

With the `#[kiss3d::main]` macro, you can simplify this to:

```rust
#[kiss3d::main]
async fn main() {
    let mut window = Window::new("My App");
    while window.render_async().await {
        // Your render loop
    }
}
```

The macro automatically generates the appropriate platform-specific entry points:
- On native platforms: uses `pollster::block_on` to run the async function
- On WASM: uses `wasm_bindgen_futures::spawn_local` to spawn the async function

### Requirements

- The function must be named `main`
- The function must be `async`
- The function cannot have parameters
- The function should use the async rendering methods (`render_async()`, `render_with_camera_async()`, etc.)

### Example

See `examples/instancing3d.rs` and `examples/macro_test.rs` for complete examples.
