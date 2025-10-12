# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**kiss3d** is a Keep It Simple, Stupid 3D graphics engine for Rust. The philosophy is to provide one-liner features that let developers draw simple geometric figures with minimal friction. It's explicitly NOT designed to be feature-complete or ultra-fast, but rather to be easy to use.

Key design principles:
- One-liner features from the user's perspective are preferred
- KISS (Keep It Simple, Stupid) philosophy throughout
- Support for both native and WASM targets
- Built on OpenGL/WebGL via the `glow` crate

## Building and Testing

### Native Build
```bash
cargo build
cargo build --release
```

### WASM Build
```bash
# Add the WASM target first (one-time setup)
rustup target add wasm32-unknown-unknown

# Install wasm-server-runner (one-time setup, needed to run WASM examples)
cargo install wasm-server-runner

# Build for WASM
cargo build --target wasm32-unknown-unknown
```

Note: The `.cargo/config.toml` file configures `wasm-server-runner` as the runner for WASM targets. This tool serves WASM binaries via a local HTTP server and automatically opens them in your browser.

### Running Examples
```bash
# Run a native example
cargo run --example primitives

# WASM examples require wasm-server-runner
cargo run --example cube --target wasm32-unknown-unknown

# Examples with "_wasm2d" or "_wasm3d" suffix are designed to be WASM-compatible
cargo run --example instancing_wasm2d --target wasm32-unknown-unknown
cargo run --example instancing_wasm3d --target wasm32-unknown-unknown
```

### Testing
```bash
cargo test
```

### Code Quality
```bash
# Check formatting
cargo fmt -- --check

# Fix formatting
cargo fmt

# Run clippy
cargo clippy

# Generate documentation
cargo doc
```

### CI Requirements
All CI checks must pass before merging:
- Code formatting (`cargo fmt`)
- Clippy lints (with `-D warnings`)
- Documentation builds without warnings
- Both native and WASM builds succeed
- All tests pass
- Typo checks via typos-cli

## Architecture

### Core Components

**Window (`src/window/window.rs`)**
- Central interface to the 3D engine
- Manages the render loop, event handling, and scene graph
- Supports both native (via glutin) and WASM (via web-sys) backends through the `Canvas` abstraction
- Key responsibilities:
  - Scene graph management (3D via `SceneNode`, 2D via `PlanarSceneNode`)
  - Camera management (default: `ArcBall`, also supports `FirstPerson`, custom cameras)
  - Event handling and input
  - Rendering pipeline coordination
  - Post-processing effects

**Scene Graph (`src/scene/`)**
- **SceneNode**: Hierarchical 3D scene graph with transform propagation
  - Contains local and world transforms/scales
  - Supports parent-child relationships
  - Can contain an `Object` for rendering
  - Transform invalidation cascades to children
- **PlanarSceneNode**: 2D equivalent for planar rendering
- **Object**: Wraps a mesh with material, texture, and rendering state
- Scene nodes support instancing via `set_instances()` for efficient rendering of duplicates

**Resource Management (`src/resource/`)**
- **MeshManager**: Global manager for 3D mesh geometries (cube, sphere, cone, cylinder, etc.)
- **PlanarMeshManager**: 2D mesh manager (rectangle, circle, etc.)
- **MaterialManager**: Shader programs and materials
- **TextureManager**: Texture loading and caching
- **FramebufferManager**: Offscreen render targets for post-processing
- All managers use singleton pattern via static thread-local storage
- Resources are reference-counted (`Rc<RefCell<T>>`)

**Rendering Pipeline**
1. Event handling (input, window resize)
2. Camera update
3. For each camera pass:
   - Render to framebuffer (or post-processing target)
   - Render 3D scene (`SceneNode` hierarchy)
   - Render lines and points
4. Render 2D planar scene (`PlanarSceneNode` hierarchy)
5. Apply post-processing effects (if any)
6. Render text overlay
7. Render Conrod UI (if feature enabled)
8. Swap buffers

**Built-in Shaders (`src/builtin/`)**
- `object_material.rs`: Default 3D object shader (supports lighting, textures, colors)
- `planar_object_material.rs`: Default 2D shader
- `normals_material.rs`, `uvs_material.rs`: Debug visualization materials
- Shaders are embedded as `*.vert` and `*.frag` files using `include_str!`

**Camera System (`src/camera/`)**
- `Camera` trait: Defines camera interface (view/projection matrices, event handling)
- `ArcBall`: Default orbital camera (mouse drag to rotate, scroll to zoom)
- `FirstPerson`: FPS-style camera
- `FixedView`: Static camera
- `FirstPersonStereo`: Stereo rendering support
- All cameras implement event handling for user interaction

**Platform Abstraction (`src/window/canvas.rs`)**
- `Canvas` trait abstracts native vs WASM windowing
- `GlCanvas`: Native implementation via glutin
- `WebGlCanvas`: WASM implementation via web-sys
- Handles context creation, event loop, and platform-specific APIs

### Key Patterns

**WASM Compatibility**
- WASM apps must use `window.render_loop(state)` pattern with a `State` implementation
- Native apps can use simple `while window.render() { }` loop
- The `State` trait has a `step(&mut Window)` method called each frame

**Converting Examples to WASM**

To convert a native example to be WASM-compatible:

1. **Create a State struct** containing all application state:
```rust
struct AppState {
    scene_node: SceneNode,
    rotation: UnitQuaternion<f32>,
    // ... other state
}
```

2. **Implement the State trait**:
```rust
impl State for AppState {
    // Called every frame for updates
    fn step(&mut self, _window: &mut Window) {
        self.scene_node.prepend_to_local_rotation(&self.rotation);
    }

    // Optional: provide custom cameras/renderers
    fn cameras_and_effect_and_renderer(&mut self) -> (
        Option<&mut dyn Camera>,
        Option<&mut dyn PlanarCamera>,
        Option<&mut dyn Renderer>,
        Option<&mut dyn PostProcessingEffect>,
    ) {
        (None, None, None, None)  // Use defaults
    }
}
```

3. **Replace the render loop**:
```rust
// Before (native-only):
while window.render() {
    cube.prepend_to_local_rotation(&rot);
}

// After (WASM-compatible):
let state = AppState { scene_node: cube, rotation: rot };
window.render_loop(state);
```

See examples with the `_wasm` suffix (e.g., `examples/instancing_wasm2d.rs`, `examples/instancing_wasm3d.rs`) for complete examples of this pattern.

**WASM Canvas Creation**
- The WebGL canvas (`src/window/webgl_canvas.rs`) automatically creates a canvas element if one with id "canvas" doesn't exist
- Auto-created canvas fills the viewport (100vw × 100vh, display: block)
- Canvas is appended to document body
- This means WASM apps work without requiring a pre-defined `<canvas id="canvas">` in the HTML

**Transform Hierarchy**
- Scene nodes maintain both local and world transforms
- Invalidation pattern: when a node's local transform changes, it invalidates itself and all descendants
- World transforms are lazily recomputed during rendering or when explicitly requested

**Resource Sharing**
- Heavy use of `Rc<RefCell<T>>` for shared ownership of GPU resources
- Global managers use thread-local static storage (`TLS_MANAGER_NAME`)
- Access via `Manager::get_global_manager(|manager| { ... })` pattern

**OpenGL Context Abstraction**
- `Context` trait (`src/context/`) abstracts OpenGL calls
- Allows supporting both native OpenGL and WebGL
- All GL calls go through `Context::get()` singleton

## Dependencies

- **nalgebra 0.30**: Linear algebra (vectors, matrices, transforms)
- **ncollide3d 0.33**: Procedural mesh generation
- **glow 0.12**: OpenGL/WebGL bindings
- **glutin 0.26**: Native windowing (not on WASM)
- **web-sys**: WASM browser APIs
- **rusttype**: Text rendering
- **image 0.24**: Texture loading
- **conrod** (optional feature): Immediate-mode UI

## Important Notes

- **Material/Mesh Lifetime**: Scene nodes hold `Rc` references to meshes/materials, which are also cached in global managers. Be careful with manual resource cleanup.
- **WASM Support**: Full WASM support with automatic canvas creation. Apps using the `State` trait pattern work on both native and WASM without changes.
- **WASM Limitations**: Some features behave differently on WASM (e.g., cursor grab, window icons, title changes have no effect)
- **Vertex Index Type**: The `vertex_index_u32` feature switches from u16 to u32 indices for large meshes
- **Main Branch**: The main branch is `master`, not `main`
- **Version**: Current version is 0.36.0

## Common Pitfalls

- Don't forget to call `window.render()` or `window.render_loop(state)` - nothing appears without it
- Scene nodes must be added to the window's scene or a child of it to be rendered
- Post-processing effects require rendering to an offscreen buffer first
- Transform changes don't take effect until the next render pass
- Global managers persist across window instances - be careful with tests

### WASM-Specific Pitfalls

- **State ownership**: Once you pass state to `window.render_loop(state)`, you can't access it anymore (ownership is moved). All application state must be in the `State` struct.
- **Blocking operations**: The WASM render loop uses `requestAnimationFrame`, so blocking operations will freeze the browser. Keep `step()` fast.
- **Canvas requirement**: While the canvas is now auto-created, if you need custom canvas attributes (e.g., specific size, position), you should still define it in HTML.
- **Event handling**: Some native window events (cursor grab, window icons, etc.) are no-ops on WASM.
- **Default cameras**: If you don't override `cameras_and_effect_and_renderer()`, the default `ArcBall` 3D camera and `PlanarFixedView` 2D camera are used.
