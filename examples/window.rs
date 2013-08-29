extern mod kiss3d;

use kiss3d::window::Window;

#[start]
fn start(argc: int, argv: **u8, crate_map: *u8) -> int {
    std::rt::start_on_main_thread(argc, argv, crate_map, main)
}

fn main() {
    do Window::spawn("Kiss3d: empty window") |window| {
        window.set_background_color(0.0, 0.0, 0.3)
    };
}
