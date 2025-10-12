//! The window, and things to handle the rendering loop and events.

mod canvas;
#[cfg(not(target_arch = "wasm32"))]
mod wgpu_canvas;
mod state;
#[cfg(target_arch = "wasm32")]
mod wgpu_wasm_canvas;
mod window;
mod window_cache;

pub(crate) use canvas::AbstractCanvas;
pub use canvas::{Canvas, CanvasSetup, NumSamples};
#[cfg(not(target_arch = "wasm32"))]
pub use wgpu_canvas::WgpuCanvas;
pub use state::State;
#[cfg(target_arch = "wasm32")]
pub use wgpu_wasm_canvas::WgpuWasmCanvas;
pub use window::Window;
pub(crate) use window_cache::WINDOW_CACHE;
